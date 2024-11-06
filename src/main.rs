mod player;
mod enemy;
mod map;
mod quest;
mod utils;
mod skill;

use player::Player;
use map::{Map, Direction};
use quest::{Quest, sample_quests};
use enemy::Enemy;
use skill::initialize_skills;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_json;
use std::panic;
use rand::Rng;
use enemy::sample_enemies;

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
    // Set a panic hook to ensure any panic properly prints and cleans up terminal
    panic::set_hook(Box::new(|panic_info| {
        println!("Application panicked: {}", panic_info);
    }));

    // Check if the "Saves" folder exists, create if not
    let saves_path = Path::new("Saves");
    if !saves_path.exists() {
        fs::create_dir(saves_path).expect("Failed to create Saves folder");
    }

    // Display initial menu
    loop {
        println!("1. New Game\n2. Continue\n3. Load Save\n(q) Quit");
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
                    println!("No recent save found.");
                }
            }
            "3" => {
                if let Some(save) = load_save_menu() {
                    load_game(&save);
                    break;
                }
            }
            "q" => return,
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn new_game() {
    // Prompt for character name
    println!("Enter your character's name: ");
    let mut character_name = String::new();
    io::stdin().read_line(&mut character_name).expect("Failed to read line");
    let character_name = character_name.trim().to_string();

    // Create save folder for the character
    let save_folder = format!("Saves/save1_{}", character_name);
    fs::create_dir(&save_folder).expect("Failed to create save folder");

    // Initialize player and map
    let mut player = Player::new();
    let game_map = Map::new(300, 300);  // Generate a new map
    player.skills = initialize_skills();

    // Load sample quests
    let quests = sample_quests();

    // Save character and map data
    save_game(&player, &game_map, &save_folder, &character_name);

    // Continue with the game loop
    game_loop(player, game_map, quests);
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
    println!("(Backspace) return to main menu\n(q) quit");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");
    let choice = choice.trim();

    if choice == "q" {
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

    // Load map data
    let map_file_path = format!("{}/map.json", save_folder);
    let map_data: Map = serde_json::from_str(&fs::read_to_string(&map_file_path).expect("Failed to read map file")).expect("Failed to parse map file");

    let mut player = Player::new();
    player.health = character_data.health;
    player.level = character_data.level;
    player.experience = character_data.experience;
    player.skills = initialize_skills(); // Replace with deserialization if detailed skill information needs to be restored

    // Load sample quests
    let quests = sample_quests();

    game_loop(player, map_data, quests);
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
        current_map: format!("{}/map.json", save_folder),
    };
    let character_save_path = format!("{}/character.json", save_folder);
    fs::write(&character_save_path, serde_json::to_string(&character_save).unwrap()).expect("Failed to write character file");

    // Create map save file
    let map_save_path = format!("{}/map.json", save_folder);
    fs::write(&map_save_path, serde_json::to_string(&game_map).unwrap()).expect("Failed to write map file");
}

fn handle_combat(player: &mut Player, mut enemy: Enemy) -> String {
    println!("You've encountered a {}!", enemy.name);
    loop {
        // Display combat options
        println!("Enemy: {} (Health: {})", enemy.name, enemy.health);
        println!("Your health: {}", player.health);
        println!("Do you want to (f)ight or (r)un?");
        let mut action = String::new();
        io::stdin().read_line(&mut action).expect("Failed to read line");
        let action = action.trim();

        match action {
            "f" => {
                // Player attacks enemy
                enemy.take_damage(10); // Example player attack damage
                println!("You hit the {} for 10 damage!", enemy.name);

                if enemy.is_defeated() {
                    println!("You have defeated the {}!", enemy.name);
                    return format!("Defeated a {}.", enemy.name);
                }

                // Enemy attacks player
                enemy.attack_player(&mut player.health);
                println!("The {} hits you for {} damage!", enemy.name, enemy.attack);

                if player.health <= 0 {
                    println!("You have been defeated by the {}...", enemy.name);
                    return "You were defeated...".to_string();
                }
            }
            "r" => {
                // Attempt to run away
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    println!("You successfully ran away!");
                    return "Ran away from combat.".to_string();
                } else {
                    println!("Failed to run away! The {} attacks!", enemy.name);
                    enemy.attack_player(&mut player.health);
                    if player.health <= 0 {
                        println!("You have been defeated by the {}...", enemy.name);
                        return "You were defeated...".to_string();
                    }
                }
            }
            _ => println!("Invalid command, please choose 'f' to fight or 'r' to run."),
        }
    }
}

fn game_loop(mut player: Player, mut game_map: Map, quests: Vec<Quest>) {
    let mut recent_actions: VecDeque<String> = VecDeque::with_capacity(3);
    let mut new_action: String;

    loop {
        // Clear the terminal using ANSI escape codes
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the top menu
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit");
        println!();

        // Render the current 30x30 chunk of the map
        println!("{}", game_map.render());

        // Render recent actions
        println!();
        println!("Recent Actions:");
        for action in &recent_actions {
            println!("{}", action);
        }

        // Prompt for player action
        println!("\nWhat would you like to do?...");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        match input {
            "q" => break, // Exit game
            "w" | "s" | "a" | "d" => {
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
                if rng.gen_range(0..100) < 20 {
                    let enemies = sample_enemies();
                    let enemy = enemies[rng.gen_range(0..enemies.len())].clone();
                    new_action = handle_combat(&mut player, enemy);
                }
            }
            "quests" => {
                // Display active quests
                println!("Active Quests:");
                for quest in &quests {
                    if quest.is_completed() {
                        println!("- {}: Completed", quest.name);
                    } else {
                        println!("- {}: {}", quest.name, quest.description);
                    }
                }
                new_action = "Viewed active quests.".to_string();
            }
            _ => new_action = "Invalid command.".to_string(),
        }

        // Add the new action to the recent actions queue
        if recent_actions.len() == 3 {
            recent_actions.pop_front(); // Remove the oldest action if at capacity
        }
        recent_actions.push_back(new_action);
    }
}
