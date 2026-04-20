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

const SHRIMP_ID: u32 = 100015;
const SHRIMP_XP: f64 = 10.0;

const ANCHOVY_ID: u32 = 100027;
const ANCHOVY_XP: f64 = 25.0;

/// Level at which base catch chance reaches 100%.
const CATCH_CAP_LEVEL: i32 = 10;
/// Base catch chance at level 1.
const MIN_CATCH_CHANCE: f64 = 0.50;

/// Level at which anchovies start appearing.
const ANCHOVY_MIN_LEVEL: i32 = 10;
/// Level at which anchovy chance reaches its maximum.
const ANCHOVY_CAP_LEVEL: i32 = 20;
/// Anchovy share of the catch at ANCHOVY_MIN_LEVEL.
const ANCHOVY_CHANCE_MIN: f64 = 0.10;
/// Anchovy share of the catch at ANCHOVY_CAP_LEVEL+.
const ANCHOVY_CHANCE_MAX: f64 = 0.80;

/// Milliseconds between fishing ticks.
const TICK_MS: u64 = 1000;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// One fishing attempt. Returns GatherResult::Success with the caught item,
/// GatherResult::Failed on an unsuccessful cast, or GatherResult::NothingNearby
/// if there's no water adjacent.
fn attempt_fish(player: &mut Player, map: &Map) -> GatherResult {
    if find_adjacent_tile(map, Tile::Water).is_none() {
        return GatherResult::NothingNearby;
    }

    let level = player
        .skills
        .get("Fishing")
        .map(|s| s.level)
        .unwrap_or(1)
        .max(1);

    let mut rng = rand::thread_rng();

    // Base catch chance: 50% at level 1, 100% at level 10+
    let level_progress = (level - 1).min(CATCH_CAP_LEVEL - 1) as f64 / (CATCH_CAP_LEVEL - 1) as f64;
    let catch_chance = MIN_CATCH_CHANCE + level_progress * (1.0 - MIN_CATCH_CHANCE);

    if rng.gen::<f64>() >= catch_chance {
        return GatherResult::Failed;
    }

    // Determine what was caught
    let (item_id, xp) = if level >= ANCHOVY_MIN_LEVEL {
        let anchovy_progress = (level - ANCHOVY_MIN_LEVEL)
            .min(ANCHOVY_CAP_LEVEL - ANCHOVY_MIN_LEVEL) as f64
            / (ANCHOVY_CAP_LEVEL - ANCHOVY_MIN_LEVEL) as f64;
        let anchovy_chance =
            ANCHOVY_CHANCE_MIN + anchovy_progress * (ANCHOVY_CHANCE_MAX - ANCHOVY_CHANCE_MIN);

        if rng.gen::<f64>() < anchovy_chance {
            (ANCHOVY_ID, ANCHOVY_XP)
        } else {
            (SHRIMP_ID, SHRIMP_XP)
        }
    } else {
        (SHRIMP_ID, SHRIMP_XP)
    };

    player.add_item_to_inventory(item_id, 1);
    if let Some(skill) = player.skills.get_mut("Fishing") {
        skill.add_experience(xp);
    }

    GatherResult::Success {
        item_id,
        quantity: 1,
        xp,
        resource_depleted: false,
    }
}

// ---------------------------------------------------------------------------
// Stationary fishing loop  —  "fish" command
// ---------------------------------------------------------------------------

/// Run a fishing session locked to the player's current position.
/// Returns a summary string.
pub fn handle_fishing(
    player: &mut Player,
    map: &mut Map,
    recent_actions: &VecDeque<ActionEntry>,
) -> String {
    if !has_valid_tool(player, &ToolTag::FishingNet) {
        print!(
            "{}",
            render_mode_frame(map, "[FISHING]  x or q = stop", recent_actions)
        );
        io::stdout().flush().unwrap();
        print!("You need a small net to fish here.\r\n");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1500));
        return "No fishing net.".to_string();
    }

    if find_adjacent_tile(map, Tile::Water).is_none() {
        print!(
            "{}",
            render_mode_frame(map, "[FISHING]  x or q = stop", recent_actions)
        );
        io::stdout().flush().unwrap();
        print!("No water here. Move adjacent to a river or lake to fish.\r\n");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1500));
        return "Not adjacent to water.".to_string();
    }

    let mut fish_caught: u32 = 0;
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
            format!("Fishing: {} caught  (+{:.0} XP)", fish_caught, total_xp),
            1,
        ));
        print!("{}", render_mode_frame(map, "[FISHING]  x or q = stop", &feed));
        io::stdout().flush().unwrap();

        // Stop if player wandered away from water
        if find_adjacent_tile(map, Tile::Water).is_none() {
            print!("You moved away from the water.\r\n");
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(1000));
            break;
        }

        match attempt_fish(player, map) {
            GatherResult::Success { item_id, xp, .. } => {
                fish_caught += 1;
                total_xp += xp;
                let notifications = player.on_item_gained(&[item_id]);
                for line in &notifications {
                    print!("{}\r\n", line);
                }
            }
            GatherResult::Failed => {}
            _ => break,
        }

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");

    format!(
        "Fishing: {} fish  (+{:.0} Fishing XP)",
        fish_caught, total_xp
    )
}

// ---------------------------------------------------------------------------
// Autonomous gather-fish loop  —  "gather fish" command
// ---------------------------------------------------------------------------

/// Automated fishing that wanders toward water and fishes when adjacent.
/// Returns a summary string.
pub fn handle_gather_fish(
    player: &mut Player,
    map: &mut Map,
    recent_actions: &VecDeque<ActionEntry>,
) -> String {
    if !has_valid_tool(player, &ToolTag::FishingNet) {
        print!(
            "{}",
            render_mode_frame(map, "[GATHER FISH]  x or q = stop", recent_actions)
        );
        io::stdout().flush().unwrap();
        print!("You need a small net to gather fish.\r\n");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1500));
        return "No fishing net.".to_string();
    }

    let mut prev_direction = player.facing;
    let mut rng = rand::thread_rng();
    let mut fish_caught: u32 = 0;
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
            format!("Fishing: {} caught  (+{:.0} XP)", fish_caught, total_xp),
            1,
        ));
        print!("{}", render_mode_frame(map, "[GATHER FISH]  x or q = stop", &feed));
        io::stdout().flush().unwrap();

        if find_adjacent_tile(map, Tile::Water).is_some() {
            // Fish from here
            match attempt_fish(player, map) {
                GatherResult::Success { item_id, xp, .. } => {
                    fish_caught += 1;
                    total_xp += xp;
                    let notifications = player.on_item_gained(&[item_id]);
                    for line in &notifications {
                        print!("{}\r\n", line);
                    }
                }
                GatherResult::Failed => {}
                _ => break,
            }
        } else {
            // Wander toward nearest water tile
            match find_nearest_tile(map, Tile::Water) {
                Some((tx, ty)) => {
                    let weights = weights_toward(tx, ty, map.player_x, map.player_y);
                    let dir = weighted_random_direction(&mut rng, &weights, prev_direction, map);
                    map.move_player(&dir);
                    player.facing = dir;
                    prev_direction = dir;
                }
                None => {
                    // No water on the map — wander randomly
                    let weights = MovementWeights {
                        same_direction: 128,
                        up: 64, down: 64, left: 64, right: 64,
                    };
                    let dir = weighted_random_direction(&mut rng, &weights, prev_direction, map);
                    map.move_player(&dir);
                    player.facing = dir;
                    prev_direction = dir;
                }
            }
        }

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");

    format!(
        "Gather fish: {} fish  (+{:.0} Fishing XP)",
        fish_caught, total_xp
    )
}
