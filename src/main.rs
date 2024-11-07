mod player;
mod enemy;
mod map;
mod quest;
mod utils;
mod skill;
mod items;
mod combat;

use std::env;
use player::Player;
use map::{Map, Direction};
use quest::{Quest, sample_quests};
use skill::initialize_skills;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_json;
use regex::Regex;
use std::panic;
use rand::Rng;
use enemy::basic_enemies;
use crate::items::{create_items, create_loot_tables};
use crate::combat::handle_combat;
use term_size;

#[derive(Serialize, Deserialize)]
struct CharacterSave {
    name: String,
    health: i32,
    level: i32,
    experience: i32,
    skills: Vec<(String, i32, i32)>, // Skill name, level, experience
    player_x: usize,
    player_y: usize,
    current_map: String,
}

fn main() {
    // Retrieve the version and build number from environment variables
    let version = option_env!("VERSION").unwrap_or("unknown version");
    let build_number = option_env!("BUILD_NUMBER").unwrap_or("unknown build");

    // Ensure the "Saves" directory exists before starting the game
    let saves_path = Path::new("Saves");
    if !saves_path.exists() {
        fs::create_dir(saves_path).expect("Failed to create Saves folder");
    }

    // Debug information for environment variable retrieval
    println!("Debug: VERSION={} BUILD_NUMBER={}", version, build_number);

    // Display game version information at the top of the main menu
    loop {
        // Clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the game name and version information
        println!("rustpg version {} build {}\n", version, build_number);

        // Render the main menu
        println!("1. New Game");
        println!("2. Continue");
        println!("3. Load Save");
        println!("(q)uit");

        // User input and menu handling
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "1" => {
                new_game();
                break;
            }
            "2" => {
                if let Some(recent_save) = get_recent_save() {
                    load_game(&recent_save);
                    break;
                } else {
                    println!("No recent save found. Press Enter to continue...");
                    let _ = io::stdin().read_line(&mut String::new());  // Wait for user input
                }
            }
            "3" => {
                if let Some(save) = load_save_menu() {
                    load_game(&save);
                    break;
                }
            }
            "q" => return,
            _ => {
                println!("Invalid choice. Please try again. Press Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());  // Wait for user input
            }
        }
    }
}

fn new_game() {
    loop {
        // Prompt for character name
        println!("Enter your character's name (max 32 characters, no special characters):");
        let mut character_name = String::new();
        io::stdin().read_line(&mut character_name).expect("Failed to read line");
        let character_name = character_name.trim().to_string();

        // Sanitize character name
        let sanitized_name = sanitize_character_name(&character_name);

        // Check if name is too long
        if sanitized_name.len() > 32 {
            println!(
                "The name '{}' cannot be used because it contains more than 32 characters. Please use a different name.",
                character_name
            );
            continue;
        }

        // Check for invalid characters (allowing alphanumeric and spaces only)
        let valid_name_regex = Regex::new(r"^[a-zA-Z0-9 ]+$").unwrap();
        if !valid_name_regex.is_match(&sanitized_name) {
            println!(
                "The name '{}' cannot be used because it contains special characters. Please use a different name.",
                character_name
            );
            continue;
        }

        // Create save folder for the character inside the "Saves" directory
        let save_folder = format!("Saves/{}", sanitized_name);
        if Path::new(&save_folder).exists() {
            println!(
                "The name '{}' is already in use. Please use a different name.",
                sanitized_name
            );
            continue;
        }

        if let Err(err) = fs::create_dir(&save_folder) {
            eprintln!("Failed to create save folder: {}", err);
            println!("Press Enter to continue...");
            let _ = io::stdin().read_line(&mut String::new());  // Wait for user input before returning
            return;
        }

        // Initialize player and map
        let mut player = Player::new();
        let game_map = Map::new(300, 300);  // Generate a new map
        player.skills = initialize_skills();

        // Load sample quests
        let quests = sample_quests();

        // Save character and map data
        save_game(&player, &game_map, &save_folder, &sanitized_name);

        // Continue with the game loop
        game_loop(player, game_map, quests, save_folder.clone(), sanitized_name);
        break;  // Exit the loop if everything is successful
    }
}

fn sanitize_character_name(name: &str) -> String {
    // Remove leading and trailing spaces, replace consecutive spaces with a single space
    let trimmed_name = name.trim();
    let space_reduced_name = Regex::new(r"\s+").unwrap().replace_all(trimmed_name, " ");
    space_reduced_name.to_string()
}

fn get_recent_save() -> Option<String> {
    // Find the most recently modified save directory
    let saves_path = Path::new("Saves");
    let mut save_dirs: Vec<_> = fs::read_dir(saves_path)
    .expect("Failed to read Saves directory")
    .filter_map(|entry| entry.ok())
    .filter(|entry| entry.file_type().unwrap().is_dir())
    .collect();

    save_dirs.sort_by_key(|entry| fs::metadata(entry.path()).unwrap().modified().unwrap());
    save_dirs.last().map(|entry| entry.path().to_str().unwrap().to_string())
}

fn load_save_menu() -> Option<String> {
    // List all saves in the "Saves" directory
    let saves_path = Path::new("Saves");
    let save_dirs: Vec<_> = fs::read_dir(saves_path)
        .expect("Failed to read Saves directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect();

    if save_dirs.is_empty() {
        println!("No saves found.");
        return None;
    }

    println!("Select a save to load:");
    for (i, entry) in save_dirs.iter().enumerate() {
        let save_name = entry.file_name().into_string().unwrap();
        println!("{}. {}", i + 1, save_name);
    }
    println!("(b)ack\n(q)uit");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");
    let choice = choice.trim();

    if choice == "q" {
        return None;
    }

    if choice == "b" {
        return None;
    }

    if let Ok(index) = choice.parse::<usize>() {
        if index > 0 && index <= save_dirs.len() {
            return Some(save_dirs[index - 1].path().to_str().unwrap().to_string());
        }
    }

    None
}


fn load_game(save_folder: &str) {
    // Load character data
    let character_file_path = format!("{}/character.json", save_folder);
    let character_data: CharacterSave = serde_json::from_str(&fs::read_to_string(&character_file_path).expect("Failed to read character file")).expect("Failed to parse character file");

    // Load map data from serialized string
    let map_file_path = format!("{}/map.txt", save_folder);
    let map_data_str = fs::read_to_string(&map_file_path).expect("Failed to read map file");
    let map_data = Map::deserialize_map(300, 300, &map_data_str);

    let mut player = Player::new();
    player.health = character_data.health;
    player.level = character_data.level;
    player.experience = character_data.experience;
    player.skills = initialize_skills(); // Replace with deserialization if detailed skill information needs to be restored

    // Load sample quests
    let quests = sample_quests();

    game_loop(player, map_data, quests, save_folder.to_string(), character_data.name);
}

fn save_game(player: &Player, game_map: &Map, save_folder: &str, character_name: &str) {
    // Create character save file
    let character_save = CharacterSave {
        name: character_name.to_string(),
        health: player.health,
        level: player.level,
        experience: player.experience,
        skills: vec![], // Placeholder for saving skills
        player_x: game_map.player_x,
        player_y: game_map.player_y,
        current_map: format!("{}/map.txt", save_folder),
    };
    let character_save_path = format!("{}/character.json", save_folder);
    fs::write(&character_save_path, serde_json::to_string(&character_save).unwrap()).expect("Failed to write character file");

    // Create map save file using serialized RLE map representation
    let map_save_path = format!("{}/map.txt", save_folder);
    let serialized_map = game_map.serialize_map();
    fs::write(&map_save_path, serialized_map).expect("Failed to write map file");
}

fn game_loop(mut player: Player, mut game_map: Map, quests: Vec<Quest>, save_folder: String, character_name: String) {
    let mut recent_actions: VecDeque<String> = VecDeque::with_capacity(3);
    let mut new_action: String = String::new();

    loop {
        // Clear the terminal using ANSI escape codes
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Get terminal height to adjust map display size
        let view_size = if let Some((_, height)) = term_size::dimensions() {
            if height >= 44 { // Considering the entire UI (menu, map, actions)
                30 // 30x30 chunk
            } else if height >= 29 {
                20 // 20x20 chunk
            } else {
                10 // 10x10 chunk
            }
        } else {
            30 // Default to 30 if unable to detect terminal size
        };

        // Update the game map view radius based on view size
        game_map.view_radius = view_size / 2;

        // Render the top menu
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit");
        println!();

        // Render the current chunk of the map based on view size
        println!("{}", game_map.render());

        // Display recent actions if not in combat
        if !player.in_combat {
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
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        match input {
            "q" => {
                save_game(&player, &game_map, &save_folder, &character_name);
                break; // Exit game
            }
            "w" | "s" | "a" | "d" => {
                if !player.in_combat {
                    let direction = match input {
                        "w" => Direction::Up,
                        "s" => Direction::Down,
                        "a" => Direction::Left,
                        "d" => Direction::Right,
                        _ => unreachable!(),
                    };

                    // Use reference to direction for movement
                    game_map.move_player(&direction);
                    new_action = format!("Player moved {:?}", direction);

                    // Random enemy encounter logic
                    let mut rng = rand::thread_rng();
                    if rng.gen_range(0..100) < 20 {  // 20% chance to encounter an enemy
                        let enemies = basic_enemies();
                        let enemy = enemies[rng.gen_range(0..enemies.len())].clone();
                        let loot_tables = create_loot_tables();

                        player.in_combat = true; // Set in_combat to true before starting combat
                        new_action = handle_combat(&mut player, enemy, &loot_tables);
                        player.in_combat = false; // Set in_combat to false after combat ends

                        // After combat ends, clear screen to maintain clean UI
                        print!("\x1B[2J\x1B[1;1H");
                        io::stdout().flush().unwrap();
                    }
                }
            }
            // Existing options for viewing status, inventory, quests etc.
            "status" => {
                new_action = "Player status viewed".to_string();
                println!("\n[Status Information]");
                println!("Total Level: {}", player.level);
                println!("HP: {}/{}", player.health, player.max_health); // Display player health

                if let Some(active_quest) = player.active_quest.as_ref() {
                    println!("Tracking: {}", active_quest.name);
                } else {
                    println!("No active quest.");
                }
                println!("\nPress Enter to continue...");
                io::stdin().read_line(&mut String::new()).unwrap();
            }
            "quests" => {
                if quests.is_empty() {
                    println!("There are no active quests.");
                } else {
                    println!("Active Quests:");
                    for quest in &quests {
                        println!("- {}: {}", quest.name, if quest.is_completed() { "Completed" } else { &quest.description });
                    }
                }
                new_action = "Viewed active quests.".to_string();
                println!("\nPress Enter to continue...");
                io::stdin().read_line(&mut String::new()).unwrap();
            }
            "i" => {
                println!("\n[Inventory]");
                if player.inventory.is_empty() {
                    println!("Inventory is empty.");
                } else {
                    let items = create_items();
                    for (item_id, quantity) in &player.inventory {
                        if let Some(item) = items.get(item_id) {
                            println!("- {} (Quantity: {})", item.name, quantity);
                        }
                    }
                }
                new_action = "Viewed inventory.".to_string();
                println!("\nPress Enter to continue...");
                io::stdin().read_line(&mut String::new()).unwrap();
            }
            _ => {
                new_action = "Invalid command.".to_string();
                println!("Invalid command. Please try again.");
                println!("\nPress Enter to continue...");
                io::stdin().read_line(&mut String::new()).unwrap();
            }
        }

        // Add the new action to the recent actions queue if not in combat
        if !player.in_combat {
            if recent_actions.len() == 3 {
                recent_actions.pop_front(); // Remove the oldest action if at capacity
            }
            recent_actions.push_back(new_action.clone()); // Clone new_action here to keep the original value intact
        }
    }
}
