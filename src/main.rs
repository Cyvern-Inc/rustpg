mod actions;
mod combat;
mod cooking;
mod dialogue;
mod enemy;
mod fishing;
mod gathering;
mod inventory;
mod items;
mod map;
mod npc;
mod player;
mod quest;
mod skill;
mod utils;
mod woodcutting;

use crate::actions::ActionEntry;
use crate::combat::{handle_combat, CombatOutcome, FafAttackStyle, AUTO_COMBAT_STOPPED};
use crate::dialogue::run_dialogue;
use crate::npc::{resolve_interaction, InteractionAction};
use crate::gathering::{bfs_toward_nearest_reachable, find_adjacent_tile, find_nearest_tile, weights_toward};
use crate::inventory::display_and_handle_inventory;
use crate::items::get_loot_tables;
use crate::map::Tile;
use crate::player::Player;
use crate::quest::starting_quest;
use crate::utils::{
    check_for_input, render_mode_frame, weighted_random_direction, wrap_text,
    MovementWeights,
};

const MAX_PLAYER_NAME_LEN: usize = 32;
const MAX_RECENT_ACTIONS: usize = 50;
const ACTIONS_FEED_SEPARATOR: &str = "----------";
use chrono::{DateTime, Local};
use enemy::{basic_enemies, Enemy};
use map::{Direction, Map};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;
use skill::initialize_skills;
use std::collections::VecDeque;
use crossterm::event::{self as xterm_event, Event, KeyCode};
use crossterm::terminal as xterm_terminal;
use std::fs::{self, create_dir_all};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use term_size;

#[derive(Serialize, Deserialize, Clone)]
struct CharacterSave {
    player: Player,
    game_map: Map,
    character_name: String,
    current_map: String,
}

// ====================//
// Game Initialization //
// ====================//

/// Clear the screen, print the build line top-left, then draw a centered
/// titled box with a separator under the title.
///
/// `title`  — displayed centered in the title row (use spaced caps style)
/// `rows`   — content lines inside the box; empty string = blank row
///
/// Returns the left-indent string so the caller can align its input prompt.
fn draw_titled_box(version: &str, build_number: &str, title: &str, rows: &[String]) -> String {
    let (tw, th) = term_size::dimensions().unwrap_or((80, 24));

    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
    println!("rustpg v{} build {}", version, build_number);

    // Inner width: enough for the widest content row (with 4-char margin) or the
    // title (with 4-char margin), minimum 32.
    let content_max = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let inner = (content_max + 4).max(title.len() + 4).max(32);

    let title_lpad = inner.saturating_sub(title.len()) / 2;
    let title_line = format!("{}{}", " ".repeat(title_lpad), title);

    // box_h = top border + title row + separator + content rows + bottom border
    let box_h = rows.len() + 4;
    let overhead = 2; // version line + implicit newline
    let underhead = 2; // blank + prompt
    let vert_free = th.saturating_sub(box_h + overhead + underhead);
    let top_blank = vert_free / 2;

    for _ in 0..top_blank {
        println!();
    }

    let pad = " ".repeat(tw.saturating_sub(inner + 2) / 2);
    println!("{}╔{}╗", pad, "═".repeat(inner));
    println!("{}║{:<width$}║", pad, title_line, width = inner);
    println!("{}╠{}╣", pad, "═".repeat(inner));
    for row in rows {
        println!("{}║{:<width$}║", pad, row, width = inner);
    }
    println!("{}╚{}╝", pad, "═".repeat(inner));

    pad
}

fn main() {
    let version = option_env!("VERSION").unwrap_or("unknown version");
    let build_number = option_env!("BUILD_NUMBER").unwrap_or("unknown build");
    let saves_path = Path::new("Saves");
    if !saves_path.exists() {
        fs::create_dir(saves_path).expect("Failed to create Saves folder");
    }

    loop {
        let rows = vec![
            String::new(),
            "    1.  New Game".to_string(),
            "    2.  Continue".to_string(),
            "    3.  Load Save".to_string(),
            String::new(),
            "    (q) Quit".to_string(),
            String::new(),
        ];
        let pad = draw_titled_box(version, build_number, "R U S T Y   S W O R D", &rows);

        print!("\n{}> ", pad);
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "1" => new_game(version, build_number),
            "2" => {
                if let Some(recent_save) = get_recent_save() {
                    load_game(&recent_save);
                } else {
                    let rows = vec![
                        String::new(),
                        "  No recent save found.".to_string(),
                        String::new(),
                        "  Press Enter to return to the menu.".to_string(),
                        String::new(),
                    ];
                    draw_titled_box(version, build_number, "C O N T I N U E", &rows);
                    let _ = io::stdin().read_line(&mut String::new());
                }
            }
            "3" => {
                if let Some(save) = load_save_menu(version, build_number) {
                    load_game(&save);
                }
            }
            "q" => std::process::exit(0),
            _ => {} // just redraw — invalid input silently ignored
        }
    }
}

fn new_game(version: &str, build_number: &str) {
    let mut error: Option<String> = None;
    loop {
        let mut rows: Vec<String> = vec![
            String::new(),
            "  Enter a name for your character:".to_string(),
            "  Max 32 chars, letters and numbers only".to_string(),
            String::new(),
        ];
        if let Some(ref msg) = error {
            rows.push(format!("  ! {}", msg));
            rows.push(String::new());
        }
        rows.push("  (b) Back  |  (q) Quit".to_string());
        rows.push(String::new());

        let pad = draw_titled_box(version, build_number, "N E W   G A M E", &rows);
        print!("\n{}> ", pad);
        io::stdout().flush().unwrap();

        let mut character_name = String::new();
        io::stdin()
            .read_line(&mut character_name)
            .expect("Failed to read line");
        let character_name = character_name.trim();

        if character_name == "q" {
            std::process::exit(0);
        } else if character_name == "b" {
            break;
        }
        if character_name.is_empty()
            || character_name.len() > MAX_PLAYER_NAME_LEN
            || !character_name
                .chars()
                .all(|c| c.is_alphanumeric() || c.is_whitespace())
        {
            error = Some("Invalid name. Please try again.".to_string());
            continue;
        }
        let sanitized_name = if character_name.starts_with('*') {
            character_name[1..].to_string()
        } else {
            character_name.to_string()
        };
        // Proceed with creating the game
        let save_folder = Path::new("Saves").join(&sanitized_name);
        create_dir_all(&save_folder).expect("Failed to create save directory");
        let mut player = Player::new();
        let mut game_map = Map::new(300, 300);
        player.skills = initialize_skills();
        let quest = starting_quest();
        player.add_quest(quest.clone());
        game_map.campfire_x = game_map.player_x;
        game_map.campfire_y = game_map.player_y + 1;
        game_map.set_tile(game_map.campfire_x, game_map.campfire_y, Tile::Campfire);
        save_game(&player, &game_map, &save_folder, &sanitized_name);
        game_loop(
            player,
            game_map,
            save_folder.to_path_buf(),
            sanitized_name,
        );
        break;
    }
}

fn sanitize_character_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn get_recent_save() -> Option<PathBuf> {
    let saves_path = Path::new("Saves");
    let mut save_dirs: Vec<_> = fs::read_dir(saves_path)
        .expect("Failed to read Saves directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect();

    save_dirs.sort_by_key(|entry| fs::metadata(entry.path()).unwrap().modified().unwrap());
    save_dirs.last().map(|entry| entry.path())
}

fn load_save_menu(version: &str, build_number: &str) -> Option<PathBuf> {
    let saves_path = Path::new("Saves");
    let mut save_dirs: Vec<_> = fs::read_dir(saves_path)
        .expect("Failed to read Saves directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect();

    save_dirs.sort_by_key(|entry| fs::metadata(entry.path()).unwrap().modified().unwrap());
    save_dirs.reverse();

    // Build content rows
    let mut rows: Vec<String> = vec![String::new()];

    if save_dirs.is_empty() {
        rows.push("  No saves found.".to_string());
        rows.push(String::new());
        rows.push("  (b) Back  |  (q) Quit".to_string());
        rows.push(String::new());
        let pad = draw_titled_box(version, build_number, "L O A D   S A V E", &rows);
        print!("\n{}> ", pad);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        match input.trim() {
            "q" => std::process::exit(0),
            _ => return None,
        }
    }

    for (i, entry) in save_dirs.iter().enumerate() {
        let raw_name = entry.file_name().into_string().unwrap();
        // Truncate display name to keep rows a consistent width
        let name_display = if raw_name.len() > 20 {
            format!("{}...", &raw_name[..17])
        } else {
            format!("{:<20}", raw_name)
        };
        let metadata = fs::metadata(entry.path()).unwrap();
        let modified: DateTime<Local> = DateTime::from(metadata.modified().unwrap());
        let level = get_player_level(&entry.path()).unwrap_or(1);
        rows.push(format!(
            "  {:>2}.  {}  | lvl {:>2} | {}",
            i + 1,
            name_display,
            level,
            modified.format("%b, %d %Y")
        ));
    }

    rows.push(String::new());
    rows.push("  (dup NAME) Duplicate  |  (del NAME) Delete".to_string());
    rows.push("  (b) Back  |  (q) Quit".to_string());
    rows.push(String::new());

    let pad = draw_titled_box(version, build_number, "L O A D   S A V E", &rows);
    print!("\n{}> ", pad);
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");
    let choice = choice.trim().to_string();

    match choice.as_str() {
        "q" => std::process::exit(0),
        "b" => return None,
        _ => {}
    }

    // --- del NAME ---
    if let Some(name) = choice.strip_prefix("del ") {
        let target_name = name.trim();
        if let Some(target_save) = save_dirs.iter().find(|e| {
            e.file_name().to_string_lossy().eq_ignore_ascii_case(target_name)
        }) {
            let confirm_rows = vec![
                String::new(),
                format!("  Delete save for '{}'?", target_name),
                "  This cannot be undone.".to_string(),
                String::new(),
                "  Type 'yes' to confirm, or Enter to cancel.".to_string(),
                String::new(),
            ];
            let pad2 = draw_titled_box(version, build_number, "D E L E T E   S A V E", &confirm_rows);
            print!("\n{}> ", pad2);
            io::stdout().flush().unwrap();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).expect("Failed to read line");
            if confirm.trim().eq_ignore_ascii_case("yes") {
                let result_rows = match fs::remove_dir_all(target_save.path()) {
                    Ok(_) => vec![
                        String::new(),
                        format!("  '{}' deleted successfully.", target_name),
                        String::new(),
                        "  Press Enter to continue.".to_string(),
                        String::new(),
                    ],
                    Err(e) => vec![
                        String::new(),
                        format!("  Error: {}", e),
                        String::new(),
                        "  Press Enter to continue.".to_string(),
                        String::new(),
                    ],
                };
                draw_titled_box(version, build_number, "D E L E T E   S A V E", &result_rows);
            } else {
                let cancel_rows = vec![
                    String::new(),
                    "  Delete cancelled.".to_string(),
                    String::new(),
                    "  Press Enter to continue.".to_string(),
                    String::new(),
                ];
                draw_titled_box(version, build_number, "D E L E T E   S A V E", &cancel_rows);
            }
            let _ = io::stdin().read_line(&mut String::new());
        } else {
            let notfound_rows = vec![
                String::new(),
                format!("  Save '{}' not found.", target_name),
                String::new(),
                "  Press Enter to continue.".to_string(),
                String::new(),
            ];
            draw_titled_box(version, build_number, "D E L E T E   S A V E", &notfound_rows);
            let _ = io::stdin().read_line(&mut String::new());
        }
        return load_save_menu(version, build_number);
    }

    // --- dup NAME ---
    if let Some(name) = choice.strip_prefix("dup ") {
        let target_name = name.trim();
        if let Some(old_save) = save_dirs.iter().find(|e| {
            e.file_name().to_string_lossy().eq_ignore_ascii_case(target_name)
        }) {
            let dup_rows = vec![
                String::new(),
                format!("  Duplicating save for '{}'.", target_name),
                String::new(),
                "  Enter a name for the new save:".to_string(),
                String::new(),
            ];
            let pad2 = draw_titled_box(version, build_number, "D U P L I C A T E   S A V E", &dup_rows);
            print!("\n{}> ", pad2);
            io::stdout().flush().unwrap();
            let mut new_name_raw = String::new();
            io::stdin().read_line(&mut new_name_raw).expect("Failed to read line");
            let new_name = sanitize_character_name(&new_name_raw);
            let new_path = saves_path.join(&new_name);
            let result_rows = if new_path.exists() {
                vec![
                    String::new(),
                    format!("  '{}' already exists.", new_name),
                    "  Please choose a different name.".to_string(),
                    String::new(),
                    "  Press Enter to continue.".to_string(),
                    String::new(),
                ]
            } else if let Err(e) = create_dir_all(&new_path) {
                vec![
                    String::new(),
                    format!("  Error: {}", e),
                    String::new(),
                    "  Press Enter to continue.".to_string(),
                    String::new(),
                ]
            } else if let Err(e) = copy_save_folder(&old_save.path(), &new_path) {
                vec![
                    String::new(),
                    format!("  Error: {}", e),
                    String::new(),
                    "  Press Enter to continue.".to_string(),
                    String::new(),
                ]
            } else {
                vec![
                    String::new(),
                    format!("  '{}' duplicated to '{}'.", target_name, new_name),
                    String::new(),
                    "  Press Enter to continue.".to_string(),
                    String::new(),
                ]
            };
            draw_titled_box(version, build_number, "D U P L I C A T E   S A V E", &result_rows);
            let _ = io::stdin().read_line(&mut String::new());
        } else {
            let notfound_rows = vec![
                String::new(),
                format!("  Save '{}' not found.", target_name),
                String::new(),
                "  Press Enter to continue.".to_string(),
                String::new(),
            ];
            draw_titled_box(version, build_number, "D U P L I C A T E   S A V E", &notfound_rows);
            let _ = io::stdin().read_line(&mut String::new());
        }
        return load_save_menu(version, build_number);
    }

    // --- numeric selection ---
    if let Ok(index) = choice.parse::<usize>() {
        if index > 0 && index <= save_dirs.len() {
            return Some(save_dirs[index - 1].path().to_str().unwrap().to_string().into());
        }
    }

    // Invalid — redraw silently
    load_save_menu(version, build_number)
}

fn get_player_level(save_path: &Path) -> Option<u32> {
    let content = fs::read_to_string(save_path.join("character.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    // Level lives at player.level in the current save format
    let level = json["player"]["level"].as_i64().unwrap_or(1);
    Some(level.max(1) as u32)
}

fn copy_save_folder(from: &Path, to: &Path) -> io::Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let to_path = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            create_dir_all(&to_path)?;
            copy_save_folder(&entry.path(), &to_path)?;
        } else {
            fs::copy(entry.path(), to_path)?;
        }
    }
    Ok(())
}

fn load_game(save_folder: &Path) {
    let character_file_path = save_folder.join("character.json");
    let raw = match fs::read_to_string(&character_file_path) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to read save file: {}", e);
            println!("Press Enter to return to the main menu...");
            let _ = io::stdin().read_line(&mut String::new());
            return;
        }
    };
    let character_data: CharacterSave = match serde_json::from_str(&raw) {
        Ok(d) => d,
        Err(e) => {
            println!("Save file is corrupted or incompatible: {}", e);
            println!("Press Enter to return to the main menu...");
            let _ = io::stdin().read_line(&mut String::new());
            return;
        }
    };

    let map_file_path = save_folder.join("map.txt");
    let map_data_str = match fs::read_to_string(&map_file_path) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to read map file: {}", e);
            println!("Press Enter to return to the main menu...");
            let _ = io::stdin().read_line(&mut String::new());
            return;
        }
    };

    let px = character_data.game_map.player_x;
    let py = character_data.game_map.player_y;

    let mut map_data = Map::deserialize_map(
        character_data.game_map.width,
        character_data.game_map.height,
        &map_data_str,
        px,
        py,
    );
    map_data.restore_runtime_fields(character_data.game_map);

    // Restore the saved player directly — do not create a fresh one
    let mut player = character_data.player;
    player.in_combat = false; // reset any interrupted combat state

    map_data.clear_player_positions();
    map_data.set_tile(px, py, Tile::Player);

    game_loop(
        player,
        map_data,
        save_folder.to_path_buf(),
        character_data.character_name,
    );
}

fn save_game(player: &Player, game_map: &Map, save_folder: &Path, character_name: &str) {
    let character_save = CharacterSave {
        player: player.clone(),
        game_map: game_map.clone(),
        character_name: character_name.to_string(),
        current_map: save_folder.join("map.txt").to_string_lossy().into_owned(),
    };

    let character_save_path = save_folder.join("character.json");
    fs::write(
        &character_save_path,
        serde_json::to_string(&character_save).unwrap(),
    )
    .expect("Failed to write character file");

    let map_save_path = save_folder.join("map.txt");
    let serialized_map = game_map.serialize_map();
    fs::write(&map_save_path, serialized_map).expect("Failed to write map file");
}

/// Push an entry into the recent-actions queue, merging with the previous
/// entry when appropriate: same-enemy kills accumulate XP and loot, and
/// consecutive identical generic strings increment their count.
fn push_action(queue: &mut VecDeque<ActionEntry>, entry: ActionEntry) {
    let merged = match queue.back_mut() {
        Some(ActionEntry::Kill(prev)) => {
            if let ActionEntry::Kill(new) = &entry {
                if prev.enemy_name == new.enemy_name {
                    prev.kills += new.kills;
                    prev.combat_xp += new.combat_xp;
                    for (skill, &xp) in &new.xp_by_skill {
                        *prev.xp_by_skill.entry(skill.clone()).or_insert(0.0) += xp;
                    }
                    for (&id, &qty) in &new.loot {
                        *prev.loot.entry(id).or_insert(0) += qty;
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        Some(ActionEntry::Generic(prev_text, count)) => {
            if let ActionEntry::Generic(new_text, _) = &entry {
                if *prev_text == *new_text {
                    *count += 1;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        None => false,
    };

    if !merged {
        if queue.len() >= MAX_RECENT_ACTIONS {
            queue.pop_front();
        }
        queue.push_back(entry);
    }
}


/// Run a combat encounter with a specific enemy. Sets `in_combat`, calls
/// `handle_combat`, handles defeat/respawn, and returns an `ActionEntry`.
fn run_combat(player: &mut Player, game_map: &mut Map, enemy: Enemy) -> ActionEntry {
    player.in_combat = true;
    let outcome = handle_combat(player, enemy, get_loot_tables(), None);
    player.in_combat = false;
    if player.health <= 0 {
        println!("\nYou have been defeated!");
        println!("Press Enter to respawn...");
        let _ = io::stdin().read_line(&mut String::new());
        player.respawn(game_map);
        return ActionEntry::Generic("Player has respawned.".to_string(), 1);
    }
    match outcome {
        CombatOutcome::Kill(kill) => ActionEntry::Kill(kill),
        CombatOutcome::Other(s)   => ActionEntry::Generic(s, 1),
    }
}

/// Pick a random enemy from the basic pool and run a combat encounter.
fn run_random_combat(player: &mut Player, game_map: &mut Map, rng: &mut impl Rng) -> ActionEntry {
    let enemies = basic_enemies();
    let enemy = enemies[rng.gen_range(0..enemies.len())].clone();
    run_combat(player, game_map, enemy)
}

// ---------------------------------------------------------------------------
// FAF auto-combat loop
// ---------------------------------------------------------------------------

/// Milliseconds between FAF movement ticks (separate from auto-combat ticks).
const FAF_MOVE_TICK_MS: u64 = 400;

/// Manhattan radius at which FAF stops approaching a camp and starts patrolling
/// around it instead. Keeps the player far enough from the campfire that spawned
/// enemies have room to move before being immediately encountered.
const FAF_CAMP_PATROL_RADIUS: usize = 8;

/// Endless AFK loop: wanders toward enemies, auto-fights them with the chosen
/// attack style, eats food when low, respawns on death, and loops until the
/// player presses q or x.
fn handle_faf_loop(
    player: &mut Player,
    game_map: &mut Map,
    recent_actions: &mut VecDeque<ActionEntry>,
    rng: &mut impl Rng,
    attack_style: FafAttackStyle,
) {
    let header = format!("[FAF {}]  x or q = stop", attack_style.display_name());
    let mut prev_direction = player.facing;

    xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

    'faf: loop {
        game_map.regenerate_stumps();
        game_map.regenerate_encampments(rng, true); // fast mode: shorter respawn cooldown

        // Render current state
        print!("{}", render_mode_frame(game_map, &header, recent_actions));
        io::stdout().flush().unwrap();

        // Stop key (non-blocking poll)
        if let Some(key) = check_for_input() {
            if key == "q" || key == "x" {
                break 'faf;
            }
        }

        // ---------------------------------------------------------------
        // Fight any adjacent enemy before trying to move
        // ---------------------------------------------------------------
        if let Some((ex, ey)) = find_adjacent_tile(game_map, Tile::Enemy) {
            if let Some(idx) = game_map.npcs.iter().position(|n| n.x == ex && n.y == ey) {
                let npc = game_map.npcs.remove(idx);
                game_map.tiles[npc.y][npc.x] = npc.underlying_tile;
                if let Some((cx, cy)) = npc.home_camp {
                    game_map.notify_camp_npc_removed(cx, cy);
                }
                let enemy = Enemy::new(&npc.enemy_name, npc.health, npc.attack, &npc.loot_table);

                xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                player.in_combat = true;
                let outcome = handle_combat(player, enemy, get_loot_tables(), Some(attack_style.clone()));
                player.in_combat = false;
                xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

                if let CombatOutcome::Other(ref s) = outcome {
                    if s == AUTO_COMBAT_STOPPED { break 'faf; }
                }
                if player.health <= 0 {
                    player.respawn(game_map);
                    push_action(recent_actions, ActionEntry::Generic("Defeated — respawned at camp.".to_string(), 1));
                } else {
                    push_action(recent_actions, match outcome {
                        CombatOutcome::Kill(kill) => ActionEntry::Kill(kill),
                        CombatOutcome::Other(s)   => ActionEntry::Generic(s, 1),
                    });
                }
                continue 'faf;
            }
        }

        // ---------------------------------------------------------------
        // Choose movement direction — bias toward nearest enemy / camp
        // ---------------------------------------------------------------
        let direction = if let Some(dir) = bfs_toward_nearest_reachable(game_map, Tile::Enemy) {
            // BFS-optimal path to nearest reachable enemy; skips blocked ones
            dir
        } else if let Some((tx, ty)) = find_nearest_tile(game_map, Tile::EnemyCampfire) {
            let dist = (tx as isize - game_map.player_x as isize).unsigned_abs() as usize
                + (ty as isize - game_map.player_y as isize).unsigned_abs() as usize;
            if dist > FAF_CAMP_PATROL_RADIUS {
                // Far from camp — walk toward it
                bfs_toward_nearest_reachable(game_map, Tile::EnemyCampfire)
                    .unwrap_or_else(|| {
                        let w = weights_toward(tx, ty, game_map.player_x, game_map.player_y);
                        weighted_random_direction(rng, &w, prev_direction, game_map)
                    })
            } else {
                // Already within patrol radius — wander so spawned enemies have
                // room to move before the player walks into them
                let w = MovementWeights { same_direction: 48, up: 64, down: 64, left: 64, right: 64 };
                weighted_random_direction(rng, &w, prev_direction, game_map)
            }
        } else {
            let w = MovementWeights { same_direction: 128, up: 64, down: 64, left: 64, right: 64 };
            weighted_random_direction(rng, &w, prev_direction, game_map)
        };

        // Check if the target tile holds an enemy before moving
        let (target_x, target_y) = match direction {
            Direction::Up    => (game_map.player_x, game_map.player_y.saturating_sub(1)),
            Direction::Down  => (game_map.player_x, (game_map.player_y + 1).min(game_map.height - 1)),
            Direction::Left  => (game_map.player_x.saturating_sub(1), game_map.player_y),
            Direction::Right => ((game_map.player_x + 1).min(game_map.width - 1), game_map.player_y),
        };

        if game_map.tiles[target_y][target_x] == Tile::Enemy {
            if let Some(idx) = game_map.npcs.iter().position(|n| n.x == target_x && n.y == target_y) {
                let npc = game_map.npcs.remove(idx);
                game_map.tiles[npc.y][npc.x] = npc.underlying_tile;
                if let Some((cx, cy)) = npc.home_camp {
                    game_map.notify_camp_npc_removed(cx, cy);
                }
                let enemy = Enemy::new(&npc.enemy_name, npc.health, npc.attack, &npc.loot_table);

                xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                player.in_combat = true;
                let outcome = handle_combat(player, enemy, get_loot_tables(), Some(attack_style.clone()));
                player.in_combat = false;
                xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

                if let CombatOutcome::Other(ref s) = outcome {
                    if s == AUTO_COMBAT_STOPPED { break 'faf; }
                }
                if player.health <= 0 {
                    player.respawn(game_map);
                    push_action(recent_actions, ActionEntry::Generic("Defeated — respawned at camp.".to_string(), 1));
                } else {
                    push_action(recent_actions, match outcome {
                        CombatOutcome::Kill(kill) => ActionEntry::Kill(kill),
                        CombatOutcome::Other(s)   => ActionEntry::Generic(s, 1),
                    });
                }
                continue 'faf;
            }
        }

        // ---------------------------------------------------------------
        // Normal move, then tick NPCs
        // ---------------------------------------------------------------
        let prev_count = game_map.move_count;
        game_map.move_player(&direction);
        player.facing = direction;
        prev_direction = direction;

        // Tick NPCs every 2 player steps — check if one walks into the player
        if game_map.move_count > prev_count && game_map.move_count % 2 == 0 {
            if let Some(npc_idx) = game_map.update_npcs(rng) {
                let npc = game_map.npcs.remove(npc_idx);
                game_map.tiles[npc.y][npc.x] = npc.underlying_tile;
                if let Some((cx, cy)) = npc.home_camp {
                    game_map.notify_camp_npc_removed(cx, cy);
                }
                let enemy = Enemy::new(&npc.enemy_name, npc.health, npc.attack, &npc.loot_table);

                xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                player.in_combat = true;
                let outcome = handle_combat(player, enemy, get_loot_tables(), Some(attack_style.clone()));
                player.in_combat = false;
                xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

                if let CombatOutcome::Other(ref s) = outcome {
                    if s == AUTO_COMBAT_STOPPED { break 'faf; }
                }
                if player.health <= 0 {
                    player.respawn(game_map);
                    push_action(recent_actions, ActionEntry::Generic("Defeated — respawned at camp.".to_string(), 1));
                } else {
                    push_action(recent_actions, match outcome {
                        CombatOutcome::Kill(kill) => ActionEntry::Kill(kill),
                        CombatOutcome::Other(s)   => ActionEntry::Generic(s, 1),
                    });
                }
                continue 'faf;
            }
        }

        thread::sleep(Duration::from_millis(FAF_MOVE_TICK_MS));
    }

    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
    push_action(recent_actions, ActionEntry::Generic("FAF mode stopped.".to_string(), 1));
}

/// Attempt one movement step. Handles both player-initiated combat (walking
/// into an enemy tile) and NPC-initiated combat (an NPC ticks into the player).
///
/// If `in_raw_mode` is true when combat is triggered the function disables raw
/// mode first so that `handle_combat`'s output renders correctly, then sets
/// `*in_raw_mode = false` before returning.
///
/// Returns `(action_message, stop_walking)`.  `stop_walking` is true whenever
/// combat occurred — the caller should exit walk mode.
/// Move the player one step in `direction`. Always called while raw mode is active.
/// Returns `(Option<ActionEntry>, raw_was_disabled)`. `None` means no action to
/// push (Flee/Ignore). When `raw_was_disabled` is true the caller must re-enable
/// raw mode before the next frame.
fn try_move_player(
    direction: Direction,
    player: &mut Player,
    game_map: &mut Map,
    rng: &mut impl Rng,
) -> (Option<ActionEntry>, bool) {
    player.facing = direction;

    let (target_x, target_y) = match direction {
        Direction::Up    => (game_map.player_x, game_map.player_y.saturating_sub(1)),
        Direction::Down  => (game_map.player_x, (game_map.player_y + 1).min(game_map.height - 1)),
        Direction::Left  => (game_map.player_x.saturating_sub(1), game_map.player_y),
        Direction::Right => ((game_map.player_x + 1).min(game_map.width - 1), game_map.player_y),
    };

    // Player walked into an NPC tile — resolve interaction before moving
    let target_tile = game_map.tiles[target_y][target_x];
    if matches!(target_tile, Tile::Enemy | Tile::Npc) {
        if let Some(idx) = game_map.npcs.iter().position(|n| n.x == target_x && n.y == target_y) {
            let action = resolve_interaction(&game_map.npcs[idx], player);
            match action {
                InteractionAction::Combat => {
                    let npc = game_map.npcs.remove(idx);
                    game_map.tiles[npc.y][npc.x] = npc.underlying_tile;
                    if let Some((cx, cy)) = npc.home_camp {
                        game_map.notify_camp_npc_removed(cx, cy);
                    }
                    let enemy = Enemy::new(&npc.enemy_name, npc.health, npc.attack, &npc.loot_table);
                    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                    let entry = run_combat(player, game_map, enemy);
                    return (Some(entry), true);
                }
                InteractionAction::OpenDialogue(root_id) => {
                    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                    let msg = run_dialogue(player, &root_id);
                    return (Some(ActionEntry::Generic(msg, 1)), true);
                }
                InteractionAction::Flee | InteractionAction::Ignore => {
                    return (None, false);
                }
            }
        }
    }

    // Normal movement
    let prev_count = game_map.move_count;
    game_map.move_player(&direction);
    let action = ActionEntry::Generic(format!("Moved {:?}", direction), 1);

    // Tick NPCs every 2 successful player steps
    if game_map.move_count > prev_count && game_map.move_count % 2 == 0 {
        if let Some(npc_idx) = game_map.update_npcs(rng) {
            let npc = game_map.npcs.remove(npc_idx);
            game_map.tiles[npc.y][npc.x] = npc.underlying_tile;
            if let Some((cx, cy)) = npc.home_camp {
                game_map.notify_camp_npc_removed(cx, cy);
            }
            let enemy = Enemy::new(&npc.enemy_name, npc.health, npc.attack, &npc.loot_table);
            xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
            let entry = run_combat(player, game_map, enemy);
            return (Some(entry), true);
        }
    }

    (Some(action), false)
}

/// Build the full walk-mode screen string (uses \r\n for raw mode).
fn build_walk_frame(
    game_map: &Map,
    recent_actions: &VecDeque<ActionEntry>,
    command_history: &VecDeque<String>,
    input_buffer: &str,
    in_input_mode: bool,
) -> String {
    let mut out = String::new();
    out.push_str("\x1B[2J\x1B[1;1H");

    let (term_w, term_h) = term_size::dimensions().unwrap_or((80, 24));

    // Overhead: 1 header + 1 blank + 4 box lines = 6; add 1 spare = 7
    let h_radius = (term_w.saturating_sub(52) / 4).clamp(5, 40);
    let v_radius = (term_h.saturating_sub(7) / 2).clamp(5, 40);

    out.push_str("[WALK MODE]  wasd = move  |  Enter = command\r\n\r\n");

    // Map + sidebar
    let map_str = game_map.render_viewport(h_radius, v_radius);
    let map_lines: Vec<&str> = map_str.lines().collect();
    let map_height = map_lines.len();
    let max_recent = if map_height > 1 { map_height - 1 } else { 0 };

    let sidebar_w = term_w
        .saturating_sub((2 * h_radius + 1) * 2 + 4)
        .max(10);
    let mut all_lines: Vec<String> = Vec::new();
    for entry in recent_actions {
        for line in entry.format_for_sidebar() {
            all_lines.extend(wrap_text(&line, sidebar_w));
        }
    }
    let mut info_lines: Vec<String> = vec!["Recent Actions:".to_string()];
    let skip = all_lines.len().saturating_sub(max_recent);
    for line in &all_lines[skip..] {
        info_lines.push(line.clone());
    }
    if max_recent > 0 {
        while info_lines.len() <= max_recent {
            info_lines.push(ACTIONS_FEED_SEPARATOR.to_string());
        }
    }

    let map_width = map_lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let max_rows = map_lines.len().max(info_lines.len());
    for i in 0..max_rows {
        let map_part  = if i < map_lines.len()  { map_lines[i] }           else { "" };
        let info_part = if i < info_lines.len() { info_lines[i].as_str() } else { "" };
        out.push_str(&format!("{:<width$}    {}\r\n", map_part, info_part, width = map_width));
    }

    // Command box — always 4 lines so the viewport height never jumps
    let inner = term_w.saturating_sub(2).max(4);
    let content_w = inner.saturating_sub(3).max(1); // space inside "║  " prefix

    if in_input_mode {
        let title = " Command ";
        let right = inner.saturating_sub(2 + title.len());
        out.push_str(&format!("╔{}{}{}╗\r\n", "═".repeat(2), title, "═".repeat(right)));

        // One history line (most recent previous command)
        if let Some(prev) = command_history.iter().next_back() {
            let s = truncate_to(prev, content_w);
            out.push_str(&format!("║  {:<w$}║\r\n", s, w = content_w));
        } else {
            out.push_str(&format!("║{:<w$}║\r\n", "", w = inner));
        }

        // Active input line with cursor
        let cursor = format!("> {}_", input_buffer);
        let cursor_display = truncate_start_to(&cursor, content_w);
        out.push_str(&format!("║  {:<w$}║\r\n", cursor_display, w = content_w));

        out.push_str(&format!("╚{}╝\r\n", "═".repeat(inner)));
    } else {
        out.push_str(&format!("╔{}╗\r\n", "═".repeat(inner)));
        let hint = "  Press Enter to open the command input.";
        out.push_str(&format!("║{:<w$}║\r\n", hint, w = inner));
        out.push_str(&format!("║{:<w$}║\r\n", "", w = inner));
        out.push_str(&format!("╚{}╝\r\n", "═".repeat(inner)));
    }

    out
}

fn truncate_to(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}

fn truncate_start_to(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[s.len() - max..] }
}

/// Execute a typed command. Called in non-raw mode.
/// Returns the action string to push to recent_actions (empty = push nothing).
fn execute_command(
    input: &str,
    player: &mut Player,
    game_map: &mut Map,
    rng: &mut impl Rng,
    recent_actions: &mut VecDeque<ActionEntry>,
) -> String {
    // FAF — manages its own raw mode and pushes its own action
    if (input == "faf" || input.starts_with("faf ")) && !player.in_combat {
        let style = match input.trim_start_matches("faf").trim() {
            "spell"   => FafAttackStyle::Spell,
            "charged" => FafAttackStyle::Charged,
            _         => FafAttackStyle::Main,
        };
        handle_faf_loop(player, game_map, recent_actions, rng, style);
        return String::new();
    }

    match input {
        "status" => {
            player.display_status();
            "Viewed player status.".to_string()
        }
        "quests" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().unwrap();
            println!("[Quests]\n");
            if player.quests.is_empty() {
                println!("You have no active quests.");
            } else {
                let mut sorted = player.quests.clone();
                sorted.sort_by_key(|q| (q.is_completed(), q.id));
                for quest in &sorted {
                    let status = if quest.is_completed() { "DONE  " } else { "Active" };
                    println!("[{}] {}", status, quest.name);
                    println!("         {}", quest.description);
                    for obj in &quest.objectives {
                        let check = if obj.is_complete() { "x" } else { " " };
                        println!("         [{}] {} ({})", check, obj.description, obj.progress_str());
                    }
                    println!();
                }
            }
            println!("Press Enter to continue...");
            let _ = io::stdin().read_line(&mut String::new());
            "Viewed quests.".to_string()
        }
        "i" | "inventory" => {
            display_and_handle_inventory(player, None, Some(game_map));
            "Viewed inventory.".to_string()
        }
        "cut" => woodcutting::handle_woodcutting(player, game_map, recent_actions),
        "gather wood" => woodcutting::handle_gather_wood(player, game_map, recent_actions),
        "fish" => fishing::handle_fishing(player, game_map, recent_actions),
        "gather fish" => fishing::handle_gather_fish(player, game_map, recent_actions),
        "talk" => {
            let adjacent = find_adjacent_tile(game_map, Tile::Npc)
                .or_else(|| find_adjacent_tile(game_map, Tile::Enemy));
            if let Some((nx, ny)) = adjacent {
                if let Some(npc) = game_map.npcs.iter().find(|n| n.x == nx && n.y == ny) {
                    let action = resolve_interaction(npc, player);
                    if let InteractionAction::OpenDialogue(root_id) = action {
                        return run_dialogue(player, &root_id);
                    } else {
                        return "That creature doesn't want to talk.".to_string();
                    }
                }
            }
            "There's no one nearby to talk to.".to_string()
        }
        _ => format!("Unknown command: '{}'", input),
    }
}

fn game_loop(
    mut player: Player,
    mut game_map: Map,
    save_folder: PathBuf,
    character_name: String,
) {
    let mut rng = rand::thread_rng();
    let mut recent_actions: VecDeque<ActionEntry> = VecDeque::new();
    let mut command_history: VecDeque<String> = VecDeque::new();
    let mut input_buffer = String::new();
    let mut in_input_mode = false;

    xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

    'main: loop {
        game_map.regenerate_stumps();
        game_map.regenerate_encampments(&mut rng, false);

        let frame = build_walk_frame(
            &game_map,
            &recent_actions,
            &command_history,
            &input_buffer,
            in_input_mode,
        );
        print!("{}", frame);
        io::stdout().flush().unwrap();

        match xterm_event::read().expect("Failed to read input") {
            Event::Key(ke) => match ke.code {
                // Movement — only active outside input mode
                KeyCode::Char('w') if !in_input_mode => {
                    let (maybe_action, raw_disabled) =
                        try_move_player(Direction::Up, &mut player, &mut game_map, &mut rng);
                    if let Some(action) = maybe_action { push_action(&mut recent_actions, action); }
                    if raw_disabled {
                        xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");
                    }
                }
                KeyCode::Char('s') if !in_input_mode => {
                    let (maybe_action, raw_disabled) =
                        try_move_player(Direction::Down, &mut player, &mut game_map, &mut rng);
                    if let Some(action) = maybe_action { push_action(&mut recent_actions, action); }
                    if raw_disabled {
                        xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");
                    }
                }
                KeyCode::Char('a') if !in_input_mode => {
                    let (maybe_action, raw_disabled) =
                        try_move_player(Direction::Left, &mut player, &mut game_map, &mut rng);
                    if let Some(action) = maybe_action { push_action(&mut recent_actions, action); }
                    if raw_disabled {
                        xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");
                    }
                }
                KeyCode::Char('d') if !in_input_mode => {
                    let (maybe_action, raw_disabled) =
                        try_move_player(Direction::Right, &mut player, &mut game_map, &mut rng);
                    if let Some(action) = maybe_action { push_action(&mut recent_actions, action); }
                    if raw_disabled {
                        xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");
                    }
                }

                // Enter: open input mode, or submit command
                KeyCode::Enter => {
                    if !in_input_mode {
                        in_input_mode = true;
                    } else {
                        let cmd = input_buffer.trim().to_lowercase().to_string();
                        input_buffer.clear();
                        in_input_mode = false;

                        if cmd.is_empty() {
                            continue 'main;
                        }
                        if cmd == "q" || cmd == "quit" {
                            xterm_terminal::disable_raw_mode()
                                .expect("Failed to disable raw mode");
                            save_game(&player, &game_map, &save_folder, &character_name);
                            break 'main;
                        }

                        command_history.push_back(cmd.clone());

                        // Run command in non-raw mode (FAF manages its own raw mode internally)
                        xterm_terminal::disable_raw_mode()
                            .expect("Failed to disable raw mode");
                        print!("\x1B[2J\x1B[1;1H");
                        io::stdout().flush().unwrap();

                        let action = execute_command(
                            &cmd,
                            &mut player,
                            &mut game_map,
                            &mut rng,
                            &mut recent_actions,
                        );

                        xterm_terminal::enable_raw_mode()
                            .expect("Failed to enable raw mode");

                        if !action.is_empty() {
                            push_action(&mut recent_actions, ActionEntry::Generic(action, 1));
                        }
                    }
                }

                // Esc: close input mode and clear the buffer
                KeyCode::Esc => {
                    in_input_mode = false;
                    input_buffer.clear();
                }

                // Backspace: remove last character while typing
                KeyCode::Backspace if in_input_mode => {
                    input_buffer.pop();
                }

                // Regular character while typing
                KeyCode::Char(c) if in_input_mode => {
                    input_buffer.push(c);
                }

                _ => {}
            },
            _ => {}
        }
    }
}

