use std::collections::VecDeque;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use rand::Rng;
use crossterm::terminal as xterm_terminal;

use crate::actions::ActionEntry;
use crate::gathering::{find_adjacent_tile, find_nearest_tile, has_valid_tool, weights_toward, GatherResult};
use crate::items::ToolTag;
use crate::map::{Map, Tile};
use crate::player::Player;
use crate::utils::{check_for_input, render_mode_frame, weighted_random_direction, MovementWeights};

// ---------------------------------------------------------------------------
// Tuning constants
// ---------------------------------------------------------------------------

/// Base XP awarded per log successfully obtained.
const BASE_XP: f64 = 25.0;

/// Probability (0.0–1.0) that a successfully chopped tree becomes a stump.
const STUMP_CHANCE: f64 = 0.15;

/// Minimum success-roll chance at level 1.
const MIN_SUCCESS_CHANCE: f64 = 0.60;

/// Additional success chance gained at max level (99).
const LEVEL_BONUS_CHANCE: f64 = 0.30;

/// Milliseconds between loop ticks.
const TICK_MS: u64 = 800;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Chop a specific tree tile at (tx, ty). Caller is responsible for confirming
/// the tile is currently a Tree before calling. XP is only awarded on a
/// successful chop that yields a log — not on failed swing attempts.
fn attempt_chop_at(player: &mut Player, map: &mut Map, tx: usize, ty: usize) -> GatherResult {
    if map.tiles[ty][tx] != Tile::Tree {
        return GatherResult::NothingNearby;
    }

    let wc_level = player
        .skills
        .get("Woodcutting")
        .map(|s| s.level)
        .unwrap_or(1)
        .max(1);

    let success_chance = MIN_SUCCESS_CHANCE + (wc_level as f64 / 99.0) * LEVEL_BONUS_CHANCE;
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() >= success_chance {
        return GatherResult::Failed;
    }

    let xp = BASE_XP * (1.0 + wc_level as f64 * 0.01);
    player.add_item_to_inventory(100022, 1);
    if let Some(skill) = player.skills.get_mut("Woodcutting") {
        skill.add_experience(xp);
    }

    let depleted = rng.gen::<f64>() < STUMP_CHANCE;
    if depleted {
        map.deplete_tree(tx, ty);
    }

    GatherResult::Success {
        item_id: 100022,
        quantity: 1,
        xp,
        resource_depleted: depleted,
    }
}

// ---------------------------------------------------------------------------
// Stationary cutting loop  —  "cut" command
// ---------------------------------------------------------------------------

/// Run a woodcutting session locked to one tree tile.
/// Returns a summary string.
pub fn handle_woodcutting(
    player: &mut Player,
    map: &mut Map,
    recent_actions: &VecDeque<ActionEntry>,
) -> String {
    // Tool check — fail immediately
    if !has_valid_tool(player, &ToolTag::Axe) {
        print!("{}", render_mode_frame(map, "[WOODCUTTING]  x or q = stop", recent_actions));
        io::stdout().flush().unwrap();
        print!("You need a hatchet or axe to cut trees.\r\n");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1500));
        return "You have no axe.".to_string();
    }

    // Adjacency check
    let (tx, ty) = match find_adjacent_tile(map, Tile::Tree) {
        Some(pos) => pos,
        None => {
            print!("{}", render_mode_frame(map, "[WOODCUTTING]  x or q = stop", recent_actions));
            io::stdout().flush().unwrap();
            print!("No tree to cut. Move adjacent to a tree first.\r\n");
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(1500));
            return "Not adjacent to a tree.".to_string();
        }
    };

    let mut logs_gained: u32 = 0;
    let mut total_xp: f64 = 0.0;

    xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

    loop {
        if let Some(key) = check_for_input() {
            match key.as_str() {
                "b" | "q" | "x" => break,
                _ => {}
            }
        }

        let mut feed = recent_actions.clone();
        feed.push_back(ActionEntry::Generic(
            format!("Woodcutting: {} log{}  (+{:.0} XP)", logs_gained, if logs_gained == 1 { "" } else { "s" }, total_xp),
            1,
        ));
        print!("{}", render_mode_frame(map, "[WOODCUTTING]  x or q = stop", &feed));
        io::stdout().flush().unwrap();

        match attempt_chop_at(player, map, tx, ty) {
            GatherResult::Success { quantity, xp, resource_depleted, .. } => {
                logs_gained += quantity;
                total_xp += xp;
                let notifications = player.on_item_gained(&[100022]);
                for line in &notifications {
                    print!("{}\r\n", line);
                }
                if resource_depleted {
                    print!("The tree has been depleted. Move to another tree to continue.\r\n");
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(1500));
                    break;
                }
            }
            GatherResult::Failed => {}
            GatherResult::NothingNearby => {
                print!("The tree is already depleted.\r\n");
                io::stdout().flush().unwrap();
                thread::sleep(Duration::from_millis(1000));
                break;
            }
            _ => break,
        }

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");

    format!(
        "Woodcutting: {} log{}  (+{:.0} WC XP)",
        logs_gained,
        if logs_gained == 1 { "" } else { "s" },
        total_xp,
    )
}

// ---------------------------------------------------------------------------
// Autonomous gather-wood loop  —  "gather wood" command
// ---------------------------------------------------------------------------

/// Automated woodcutting that wanders toward trees, cuts them when adjacent,
/// then moves on when they deplete.
/// Returns a summary string.
pub fn handle_gather_wood(
    player: &mut Player,
    map: &mut Map,
    recent_actions: &VecDeque<ActionEntry>,
) -> String {
    if !has_valid_tool(player, &ToolTag::Axe) {
        print!("{}", render_mode_frame(map, "[GATHER WOOD]  x or q = stop", recent_actions));
        io::stdout().flush().unwrap();
        print!("You need a hatchet or axe to gather wood.\r\n");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1500));
        return "You have no axe.".to_string();
    }

    let mut cutting_target: Option<(usize, usize)> = None;
    let mut prev_direction = player.facing;
    let mut rng = rand::thread_rng();
    let mut logs_gained: u32 = 0;
    let mut total_xp: f64 = 0.0;

    xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

    loop {
        if let Some(key) = check_for_input() {
            match key.as_str() {
                "b" | "q" | "x" => break,
                _ => {}
            }
        }

        let mut feed = recent_actions.clone();
        feed.push_back(ActionEntry::Generic(
            format!("Woodcutting: {} log{}  (+{:.0} XP)", logs_gained, if logs_gained == 1 { "" } else { "s" }, total_xp),
            1,
        ));
        print!("{}", render_mode_frame(map, "[GATHER WOOD]  x or q = stop", &feed));
        io::stdout().flush().unwrap();

        if let Some((tx, ty)) = cutting_target {
            if map.tiles[ty][tx] != Tile::Tree {
                cutting_target = None;
                thread::sleep(Duration::from_millis(TICK_MS));
                continue;
            }

            match attempt_chop_at(player, map, tx, ty) {
                GatherResult::Success { quantity, xp, resource_depleted, .. } => {
                    logs_gained += quantity;
                    total_xp += xp;
                    let notifications = player.on_item_gained(&[100022]);
                    for line in &notifications {
                        print!("{}\r\n", line);
                    }
                    if resource_depleted {
                        cutting_target = None;
                    }
                }
                GatherResult::Failed => {}
                GatherResult::NothingNearby => {
                    cutting_target = None;
                }
                _ => break,
            }
        } else {
            if let Some(adj) = find_adjacent_tile(map, Tile::Tree) {
                cutting_target = Some(adj);
                continue;
            }

            match find_nearest_tile(map, Tile::Tree) {
                Some((tx, ty)) => {
                    let weights = weights_toward(tx, ty, map.player_x, map.player_y);
                    let direction = weighted_random_direction(&mut rng, &weights, prev_direction, map);
                    map.move_player(&direction);
                    player.facing = direction;
                    prev_direction = direction;
                }
                None => {
                    let weights = MovementWeights {
                        same_direction: 128,
                        up: 64,
                        down: 64,
                        left: 64,
                        right: 64,
                    };
                    let direction = weighted_random_direction(&mut rng, &weights, prev_direction, map);
                    map.move_player(&direction);
                    player.facing = direction;
                    prev_direction = direction;
                }
            }
        }

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");

    format!(
        "Gather wood: {} log{}  (+{:.0} WC XP)",
        logs_gained,
        if logs_gained == 1 { "" } else { "s" },
        total_xp,
    )
}

/// Public single-attempt function retained for potential external use.
pub fn attempt_woodcut(player: &mut Player, map: &mut Map) -> GatherResult {
    if !has_valid_tool(player, &ToolTag::Axe) {
        return GatherResult::NoTool;
    }
    match find_adjacent_tile(map, Tile::Tree) {
        Some((tx, ty)) => attempt_chop_at(player, map, tx, ty),
        None => GatherResult::NothingNearby,
    }
}
