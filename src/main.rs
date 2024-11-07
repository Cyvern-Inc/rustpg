mod player;
mod enemy;
mod map;
mod quest;
mod utils;
mod skill;
mod items;

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
use enemy::basic_enemies;
use crate::items::{ItemType, create_items, create_loot_tables};
use term_size;
use std::collections::HashMap;
use crate::items::{LootTable, calculate_loot};

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
    game_loop(player, game_map, quests, save_folder.clone(), character_name);
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

    // Load map data from serialized string
    let map_file_path = format!("{}/map.txt", save_folder);
    let map_data_str = fs::read_to_string(&map_file_path).expect("Failed to read map file");
    let mut map_data = Map::deserialize_map(300, 300, &map_data_str);

    // Restore player coordinates
    map_data.player_x = character_data.player_x;
    map_data.player_y = character_data.player_y;

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



fn handle_combat(player: &mut Player, mut enemy: Enemy, loot_tables: &HashMap<String, LootTable>) -> String {
    let mut rng = rand::thread_rng();
    let mut charging = false;
    let mut charge_damage = 0;
    let mut combat_message = format!("You've encountered a {}!", enemy.name);

    loop {
        // Clear terminal to provide a cleaner interface
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Display the top menu and combat status if not defeated
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit\n");

        if enemy.health > 0 && player.health > 0 {
            // Print player and enemy health during combat
            println!("Enemy: {} (Health: {})", enemy.name, enemy.health);
            println!("Your health: {}\n", player.health);

            // Print the combat message (which changes as combat progresses)
            println!("{}", combat_message);
            println!("\nChoose (m)ain, (c)harged, (s)pell, (i)tems, or (r)un?");
        }

        let mut action = String::new();
        io::stdin().read_line(&mut action).expect("Failed to read line");
        let action = action.trim();

        if charging {
            // Execute the charged attack on the next round
            enemy.take_damage(charge_damage);
            combat_message = format!("You performed a charged attack for {} damage!", charge_damage);
            charging = false; // Reset charging flag
            charge_damage = 0; // Reset charge damage

            // Check if the enemy is defeated
            if enemy.is_defeated() {
                return handle_enemy_defeat(player, &enemy, loot_tables);
            }
        } else {
            match action {
                "m" => {
                    // Perform a main attack
                    let damage = 10; // Example main attack damage
                    enemy.take_damage(damage);
                    combat_message = format!("You hit the {} for {} damage!", enemy.name, damage);

                    // Check if the enemy is defeated
                    if enemy.is_defeated() {
                        return handle_enemy_defeat(player, &enemy, loot_tables);
                    }
                }
                "c" => {
                    // Begin a charged attack (3x damage)
                    charging = true;
                    charge_damage = 10 * 3; // Triple the damage of a regular attack
                    combat_message = String::from("You are preparing a charged attack...");
                }
                "s" => {
                    // Perform a spell attack
                    if player.skills.get("Magic").is_some() {
                        let damage = 15; // Example magic attack damage
                        enemy.take_damage(damage);
                        combat_message = format!("You cast a spell on the {} for {} damage!", enemy.name, damage);

                        // Check if enemy is defeated
                        if enemy.is_defeated() {
                            return handle_enemy_defeat(player, &enemy, loot_tables);
                        }
                    } else {
                        combat_message = String::from("You don't have enough magic ability to cast a spell!");
                    }
                }
                "i" => {
                    // Display consumable items during combat without progressing combat
                    loop {
                        // Clear terminal for inventory view
                        print!("\x1B[2J\x1B[1;1H");
                        io::stdout().flush().unwrap();

                        // Display the top menu consistently
                        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
                        println!("(i) inventory | (m) menu | (q) quit\n");

                        // Display player and enemy health again for context
                        println!("Enemy: {} (Health: {})", enemy.name, enemy.health);
                        println!("Your health: {}\n", player.health);

                        // Render inventory view
                        let items = create_items();
                        let mut consumables = vec![];
                        for (item_id, quantity) in &player.inventory {
                            if let Some(item) = items.get(item_id) {
                                if matches!(item.item_type, ItemType::Consumable) && *quantity > 0 {
                                    consumables.push((item.clone(), *quantity));
                                }
                            }
                        }

                        if consumables.is_empty() {
                            println!("[Consumable Items]");
                            println!("You have no consumable items.");
                        } else {
                            println!("[Consumable Items]");
                            for (item, quantity) in &consumables {
                                println!("- {} (Quantity: {})", item.name, quantity);
                            }
                        }
                        println!("\nType an item name to use it, or press enter to exit inventory.");

                        let mut item_name = String::new();
                        io::stdin().read_line(&mut item_name).expect("Failed to read line");
                        let item_name = item_name.trim();

                        if item_name.is_empty() {
                            combat_message = String::from("Exited inventory without using an item.");
                            break; // Exit inventory view and return to combat
                        }

                        if let Some((item, quantity)) = consumables.iter_mut().find(|(item, _)| item.name.eq_ignore_ascii_case(item_name)) {
                            if *quantity > 0 {
                                println!("\nYou used {}!", item.name);
                                player.consume_item(item.id); // Update player's inventory
                                *quantity -= 1; // Decrement the quantity of the item
                                combat_message = format!("You used {}!", item.name);
                                break; // Exit inventory view after using an item
                            } else {
                                println!("\nYou don't have any {} left.", item.name);
                            }
                        } else {
                            println!("\nInvalid item selection. Try again or press enter to exit inventory.");
                        }
                    }

                    // After inventory view, continue without progressing combat
                    continue; // Do not allow enemy to attack in this round
                }
                "r" => {
                    // Attempt to run away
                    if rng.gen_bool(0.5) {
                        return "You successfully ran away from combat.".to_string();
                    } else {
                        combat_message = format!("Failed to run away! The {} attacks!", enemy.name);
                    }
                }
                _ => {
                    combat_message = String::from("Invalid command. Please enter 'm', 'c', 's', 'i', or 'r'.");
                    continue; // Skip enemy turn on invalid input
                }
            }
        }

        // Enemy retaliates if not defeated
        enemy.attack_player(&mut player.health);
        combat_message.push_str(&format!("\nThe {} hits you for {} damage!", enemy.name, enemy.attack));

        // Check if player is defeated
        if player.health <= 0 {
            return handle_player_defeat(player, &enemy);
        }
    }
}

fn handle_enemy_defeat(player: &mut Player, enemy: &Enemy, loot_tables: &HashMap<String, LootTable>) -> String {
    // Clear the screen before displaying combat results for a cleaner interface
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    // Display top menu
    println!("(w/a/s/d) move | (status) player status | (quests) view quests");
    println!("(i) inventory | (m) menu | (q) quit\n");

    println!("You have defeated the {}!\n", enemy.name);

    // Calculate and grant XP
    let xp_gain = 10; // Example placeholder XP value
    player.experience += xp_gain;

    // Generate and add loot to player's inventory
    let mut loot_message = String::new();
    if let Some(loot_table) = loot_tables.get(&enemy.loot_table) {
        let loot = calculate_loot(loot_table);
        player.add_loot(&loot); // Add loot to player's inventory

        for (item_id, quantity) in loot {
            if let Some(item) = create_items().get(&item_id) {
                loot_message.push_str(&format!("({}) {}, ", quantity, item.name));
            }
        }

        // Remove the last ", " from loot_message if there are any items looted
        if !loot_message.is_empty() {
            loot_message.pop();
            loot_message.pop();
        }
    }

    // Display the detailed combat results immediately
    println!("[Combat Results]");
    println!("+{} XP", xp_gain);
    if !loot_message.is_empty() {
        println!("Looted: {}", loot_message);
    } else {
        println!("No items were looted.");
    }

    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).unwrap(); // Pause to allow the user to read loot and XP

    // Return a summary message for recent actions
    format!("Defeated a {} | +{} XP | Looted: {}", enemy.name, xp_gain, loot_message)
}


fn handle_player_defeat(player: &mut Player, enemy: &Enemy) -> String {
    // Display a message indicating the player was defeated
    format!("You have been defeated by the {}...", enemy.name)
}

fn game_loop(mut player: Player, mut game_map: Map, quests: Vec<Quest>, save_folder: String, character_name: String) {
    let mut recent_actions: VecDeque<String> = VecDeque::with_capacity(3);
    let mut new_action = String::from("Welcome to the game!");

    // Create loot tables for use during combat
    let loot_tables = create_loot_tables();

    loop {
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

        // Clear the terminal using ANSI escape codes
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the top menu
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit\n");

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
                        new_action = handle_combat(&mut player, enemy, &loot_tables); // Pass loot_tables here
                        player.in_combat = false; // Set in_combat to false after combat ends
                    }
                }
            }
            // Existing options for viewing status, inventory, quests, etc.
            _ => {
                new_action = match input {
                    "status" => {
                        println!("\n[Status Information]");
                        println!("Total Level: {}", player.level);
                        println!("HP: {}/{}", player.health, player.max_health); // Show max health

                        if let Some(active_quest) = player.active_quest.as_ref() {
                            println!("Tracking: {}", active_quest.name);
                        } else {
                            println!("No active quest.");
                        }
                        println!("\nPress Enter to continue...");
                        io::stdin().read_line(&mut String::new()).unwrap();
                        "Player status viewed".to_string()
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
                        println!("\nPress Enter to continue...");
                        io::stdin().read_line(&mut String::new()).unwrap();
                        "Viewed active quests.".to_string()
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
                        println!("\nPress Enter to continue...");
                        io::stdin().read_line(&mut String::new()).unwrap();
                        "Viewed inventory.".to_string()
                    }
                    _ => "Invalid command.".to_string(),
                };
            }
        }

        // Add the new action to the recent actions queue if not in combat
        if !player.in_combat {
            if recent_actions.len() == 3 {
                recent_actions.pop_front(); // Remove the oldest action if at capacity
            }
            recent_actions.push_back(new_action.clone());
        }
    }
}
