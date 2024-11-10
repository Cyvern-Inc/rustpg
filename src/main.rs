mod combat;
mod enemy;
mod inventory;
mod items;
mod map;
mod player;
mod quest;
mod skill;
mod utils;

use crate::combat::handle_combat;
use crate::inventory::{display_and_handle_inventory, display_inventory, handle_eat_command};
use crate::items::{create_items, create_loot_tables, get_starting_items};
use crate::map::Tile;
use crate::player::Player;
use crate::quest::{sample_quests, starting_quest, Quest};
use crate::skill::Skill;
use crate::utils::should_encounter_enemy;
use chrono::{DateTime, Local};
use enemy::basic_enemies;
use map::{Direction, Map};
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use skill::initialize_skills;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs::{self, copy, create_dir_all, remove_dir_all};
use std::io::{self, Write};
use std::panic;
use std::path::{Path, PathBuf};
use term_size;
use utils::faf;

#[derive(Serialize, Deserialize, Clone)]
struct CharacterSave {
    player: Player,
    game_map: Map,
    quests: Vec<Quest>,
    character_name: String,
    name: String,
    health: i32,
    level: i32,
    experience: i32,
    skills: Vec<(String, i32, f32)>,
    player_x: usize,
    player_y: usize,
    current_map: String,
    inventory: std::collections::HashMap<u32, u32>,
}

// =========================
// Game Initialization
// =========================

fn main() {
    let version = option_env!("VERSION").unwrap_or("unknown version");
    let build_number = option_env!("BUILD_NUMBER").unwrap_or("unknown build");
    let saves_path = Path::new("Saves");
    if (!saves_path.exists()) {
        fs::create_dir(saves_path).expect("Failed to create Saves folder");
    }

    loop {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        println!("rustpg version {} build {}\n", version, build_number);

        println!("1. New Game");
        println!("2. Continue");
        println!("3. Load Save");
        println!("\n(q)uit");

        print!("\nEnter your choice: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "1" => new_game(),
            "2" => {
                if let Some(recent_save) = get_recent_save() {
                    load_game(&recent_save);
                } else {
                    println!("No recent save found. Press Enter to continue...");
                    let _ = io::stdin().read_line(&mut String::new());
                }
            }
            "3" => {
                if let Some(save) = load_save_menu() {
                    load_game(&save);
                }
            }
            "q" => std::process::exit(0),
            _ => {
                println!("Invalid choice. Please try again. Press Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
        }
    }
}

fn new_game() {
    loop {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        println!(
            "rustpg version {} build {}\n",
            option_env!("VERSION").unwrap_or("unknown version"),
            option_env!("BUILD_NUMBER").unwrap_or("unknown build")
        );
        println!("Enter your character's name (max 32 characters, no special characters):");
        println!("(b)ack\n(q)uit");
        print!("\nEnter your choice: ");
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
        if character_name.is_empty() || character_name.len() > 32 || !character_name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
            println!("Invalid name. Please enter a valid name.");
            println!("\nPress Enter to continue...");
            let _ = io::stdin().read_line(&mut String::new());
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
        let quests = sample_quests();
        game_map.campfire_x = game_map.player_x;
        game_map.campfire_y = game_map.player_y + 1;
        game_map.set_tile(game_map.campfire_x, game_map.campfire_y, Tile::Campfire);
        save_game(&player, &game_map, &save_folder, &sanitized_name);
        game_loop(
            player,
            game_map,
            quests,
            save_folder.to_path_buf(),
            sanitized_name,
        );
        break;
    }
}

fn sanitize_character_name(name: &str) -> String {
    let trimmed_name = name.trim();
    let space_reduced_name = Regex::new(r"\s+").unwrap().replace_all(trimmed_name, " ");
    space_reduced_name.to_string()
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

fn load_save_menu() -> Option<PathBuf> {
    let saves_path = Path::new("Saves");
    let mut save_dirs: Vec<_> = fs::read_dir(saves_path)
        .expect("Failed to read Saves directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect();

    save_dirs.sort_by_key(|entry| fs::metadata(entry.path()).unwrap().modified().unwrap());
    save_dirs.reverse();

    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    println!(
        "rustpg version {} build {}\n",
        option_env!("VERSION").unwrap_or("unknown version"),
        option_env!("BUILD_NUMBER").unwrap_or("unknown build")
    );

    if save_dirs.is_empty() {
        println!("No saves found.");
        println!("\n(b)ack, (q)uit");
        print!("\nSelect a save to load: ");
        io::stdout().flush().unwrap();
        return None;
    }

    println!("Select a save to load:");
    for (i, entry) in save_dirs.iter().enumerate() {
        let save_name = entry.file_name().into_string().unwrap();
        let metadata = fs::metadata(entry.path()).unwrap();
        let modified: DateTime<Local> = DateTime::from(metadata.modified().unwrap());

        let level = get_player_level(&entry.path()).unwrap_or(1);

        println!(
            "{}. {} | lvl {} | {}",
            i + 1,
            save_name,
            level,
            modified.format("%b, %d %Y")
        );
    }

    println!("\n(dup)licate, (del)ete, (b)ack, (q)uit");
    print!("\nSelect a save to load: ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");
    let choice = choice.trim();

    if choice == "q" {
        std::process::exit(0);
    } else if choice == "b" {
        return None;
    }

    if let Some(name) = choice.strip_prefix("del ") {
        let target_name = name.trim();
        if let Some(target_save) = save_dirs.iter().find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(target_name)
        }) {
            println!(
                "Are you sure you want to delete the save for '{}'?\n(type 'yes' to confirm):",
                target_name
            );
            let mut confirm = String::new();
            io::stdin()
                .read_line(&mut confirm)
                .expect("Failed to read line");
            if confirm.trim().eq_ignore_ascii_case("yes") {
                if let Err(e) = fs::remove_dir_all(target_save.path()) {
                    println!("Failed to delete save: {}", e);
                } else {
                    println!("Save for '{}' has been deleted successfully.", target_name);
                }
            } else {
                println!("Delete action cancelled.");
            }
        } else {
            println!("Save for '{}' not found.", target_name);
        }
        println!("Press Enter to continue...");
        let _ = io::stdin().read_line(&mut String::new());
        return load_save_menu();
    }

    if let Some(name) = choice.strip_prefix("dup ") {
        let target_name = name.trim();
        if let Some(old_save) = save_dirs.iter().find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(target_name)
        }) {
            println!("Enter a name for the duplicated save:");
            let mut new_name = String::new();
            io::stdin()
                .read_line(&mut new_name)
                .expect("Failed to read line");
            let new_name = sanitize_character_name(&new_name);
            let new_path = saves_path.join(&new_name);
            if new_path.exists() {
                println!(
                    "A save with the name '{}' already exists. Please choose a different name.",
                    new_name
                );
            } else if let Err(e) = create_dir_all(&new_path) {
                println!("Failed to create duplicate save directory: {}", e);
            } else if let Err(e) = copy_save_folder(&old_save.path(), &new_path) {
                println!("Failed to copy save directory: {}", e);
            } else {
                println!(
                    "Save for '{}' has been duplicated successfully to '{}'.",
                    target_name, new_name
                );
            }
            println!("Press Enter to continue...");
            let _ = io::stdin().read_line(&mut String::new());
            return load_save_menu();
        } else {
            println!("Save for '{}' not found.", target_name);
            println!("Press Enter to continue...");
            let _ = io::stdin().read_line(&mut String::new());
            return load_save_menu();
        }
    }

    if let Ok(index) = choice.parse::<usize>() {
        if index > 0 && index <= save_dirs.len() {
            return Some(
                save_dirs[index - 1]
                    .path()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .into(),
            );
        }
    }

    println!("Invalid choice. Press Enter to continue...");
    let _ = io::stdin().read_line(&mut String::new());
    load_save_menu()
}

fn get_player_level(save_path: &Path) -> Option<u32> {
    // This function would load save data and return the player's level
    // Placeholder for now - real implementation needed
    Some(1) // Example: default level 1
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
    let character_data: CharacterSave = serde_json::from_str(
        &fs::read_to_string(&character_file_path).expect("Failed to read character file"),
    )
    .expect("Failed to parse character file");
    let map_file_path = save_folder.join("map.txt");
    let map_data_str = fs::read_to_string(&map_file_path).expect("Failed to read map file");
    let map_data = Map::deserialize_map(300, 300, &map_data_str);
    let mut player = Player::new();
    let quests = sample_quests();
    game_loop(
        player,
        map_data,
        quests,
        save_folder.to_path_buf(),
        character_data.name,
    );
}

fn save_game(player: &Player, game_map: &Map, save_folder: &Path, character_name: &str) {
    let character_save = CharacterSave {
        player: player.clone(),
        game_map: game_map.clone(),
        quests: player.quests.clone(),
        character_name: character_name.to_string(),
        name: character_name.to_string(),
        health: player.health,
        level: player.level,
        experience: player.experience,
        skills: player
            .skills
            .iter()
            .map(|(name, skill)| (name.clone(), skill.level, skill.experience))
            .collect(),
        player_x: game_map.player_x,
        player_y: game_map.player_y,
        current_map: save_folder.join("map.txt").to_string_lossy().into_owned(),
        inventory: player.inventory.clone(),
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

fn game_loop(
    mut player: Player,
    mut game_map: Map,
    quests: Vec<Quest>,
    save_folder: PathBuf,
    character_name: String,
) {
    let mut recent_actions: VecDeque<String> = VecDeque::with_capacity(3);
    let mut new_action: String = String::new();

    // Determine view size based on terminal height
    let view_size = if let Some((_, height)) = term_size::dimensions() {
        if (height >= 44) {
            30
        } else if (height >= 29) {
            20
        } else {
            10
        }
    } else {
        30
    };
    game_map.view_radius = view_size / 2;

    loop {
        // Clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the viewport
        println!("{}", game_map.render());

        // Render the top menu
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit");
        println!();

        // Display recent actions if not in combat
        if (!player.in_combat) {
            println!("\nRecent Actions:");
            for action in &recent_actions {
                println!("{}", action);
            }
            // Ensure exactly 3 lines are always displayed for recent actions
            for _ in recent_actions.len()..3 {
                println!("----------");
            }
            println!("\nWhat would you like to do?...");
        }

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();

        if input == "faf" && !player.in_combat {
            faf(&mut player, &mut game_map);
            continue;
        }

        match input {
            "q" => {
                save_game(&player, &game_map, &save_folder, &character_name);
                break; // Exit game
            }
            "w" | "s" | "a" | "d" => {
                if (!player.in_combat) {
                    let direction = match input {
                        "w" => Direction::Up,
                        "s" => Direction::Down,
                        "a" => Direction::Left,
                        "d" => Direction::Right,
                        _ => unreachable!(),
                    };
                    player.facing = direction; // Update facing direction
                    game_map.move_player(&direction);
                    new_action = format!("Player moved {:?}", direction);

                    // Random enemy encounter logic
                    if should_encounter_enemy(1) { // 1% chance
                        // Enemy encounter logic
                        let mut rng = rand::thread_rng();
                        let enemies = basic_enemies();
                        let enemy = enemies[rng.gen_range(0..enemies.len())].clone();
                        let loot_tables = create_loot_tables();
                        player.in_combat = true; // Set in_combat to true before starting combat

                        let combat_result = handle_combat(&mut player, enemy, &loot_tables);
                        println!("{}", combat_result);

                        player.in_combat = false; // Set in_combat to false after combat ends

                        // After combat ends, check if player is dead
                        if (player.health <= 0) {
                            println!("You have been defeated!");
                            println!("Press Enter to respawn...");
                            let _ = io::stdin().read_line(&mut String::new());
                            player.respawn(&mut game_map);
                            new_action = "Player has respawned.".to_string();
                        }
                    }
                }
            }
            "status" => {
                println!("{}", player.display_status());
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
                new_action = "Viewed player status.".to_string();
            }
            "i" => {
                player.display_inventory();
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
                new_action = "Viewed inventory.".to_string();
            }
            "m" => {
                // Handle menu
                println!("Menu is under construction.");
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
                new_action = "Opened menu.".to_string();
            }
            _ => {
                new_action = "Invalid command.".to_string();
                println!("Invalid command. Please try again.");
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
        }

        // Add the new action to the recent actions queue if not in combat
        if (!player.in_combat) {
            if (recent_actions.len() == 3) {
                recent_actions.pop_front(); // Remove the oldest action if at capacity
            }
            recent_actions.push_back(new_action.clone()); // Clone new_action here to keep the original value intact
        }

        // Check if player is dead and respawn if necessary
        if (player.health <= 0 && !player.in_combat) {
            // This block can be removed since respawn is handled after combat ends
            // Keeping it here as a fallback
            player.respawn(&mut game_map);
            new_action = "Player has respawned.".to_string();
        }
    }
}

// Ensure skills are initialized when creating a new player
fn create_new_player() -> Player {
    let mut player = Player::new();
    player.skills = initialize_skills();
    player
}
