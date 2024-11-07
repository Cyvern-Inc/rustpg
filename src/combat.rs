use crate::player::Player;
use crate::enemy::Enemy;
use crate::items::{create_items, ItemType};
use crate::items::{LootTable, calculate_loot};
use std::collections::HashMap;
use log::{debug, info, error}; // Import logging macros
use std::io::{self, Write};
use rand::Rng;

pub fn handle_combat(player: &mut Player, mut enemy: Enemy, loot_tables: &HashMap<String, LootTable>) -> String {
    info!("Entering combat with {}", enemy.name);
    let mut rng = rand::thread_rng();
    let mut charging = false;
    let mut charge_damage = 0;
    let mut combat_action_message = String::new();

    loop {
        // Clear terminal for better user experience
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the top menu
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit\n");

        // Display combat status and previous round actions
        if combat_action_message.is_empty() {
            // If combat just started, display encounter text
            println!("\nYou've encountered a {}!", enemy.name);
        } else {
            // Display action messages from the previous round
            println!("\n{}", combat_action_message);
        }

        // Print a gap for clarity
        println!();

        // Display enemy and player health
        println!("Enemy: {} (Health: {})", enemy.name, enemy.health);
        println!("Your health: {}\n", player.health);

        // If charging, execute the charged attack in this round
        if charging {
            println!("Press Enter to continue combat...");
            io::stdin().read_line(&mut String::new()).unwrap();

            charged_attack(player, &mut enemy, charge_damage);
            charging = false; // Reset charging flag
            charge_damage = 0; // Reset charge damage

            if enemy.is_defeated() {
                info!("Enemy {} has been defeated", enemy.name);
                return handle_enemy_defeat(player, &enemy, loot_tables);
            }

            // Enemy attacks after player’s charged attack
            enemy.attack_player(&mut player.health);
            debug!("{} hit player for {} damage", enemy.name, enemy.attack);
            combat_action_message = format!(
                "You performed a charged attack for {} damage!\nThe {} hits you for {} damage!",
                10 * 3, enemy.name, enemy.attack
            );

            // Check if the player has been defeated
            if player.health <= 0 {
                info!("Player has been defeated by {}", enemy.name);
                return handle_player_defeat(player, &enemy);
            }
        } else {
            println!("Choose (m)ain, (c)harged, (s)pell, (i)tems, or (r)un?");
            let mut action = String::new();
            io::stdin().read_line(&mut action).expect("Failed to read line");
            let action = action.trim();

            match action {
                "m" => {
                    main_attack(player, &mut enemy);
                    if enemy.is_defeated() {
                        return handle_enemy_defeat(player, &enemy, loot_tables);
                    }
                    combat_action_message = format!(
                        "You hit the {} for {} damage!\nThe {} hits you for {} damage!",
                        enemy.name, 10, enemy.name, enemy.attack
                    );
                },
                "c" => {
                    charging = true;
                    charge_damage = 10 * 3; // Define charged attack damage
                    info!("Player is preparing a charged attack.");
                    combat_action_message = format!(
                        "You are preparing a charged attack...\nThe {} hits you for {} damage!",
                        enemy.name, enemy.attack
                    );

                    // Let the enemy attack while the player charges
                    enemy.attack_player(&mut player.health);
                    debug!("{} hit player for {} damage", enemy.name, enemy.attack);

                    // Check if the player has been defeated
                    if player.health <= 0 {
                        info!("Player has been defeated by {}", enemy.name);
                        return handle_player_defeat(player, &enemy);
                    }

                    continue; // Continue to the next iteration after updating the message
                },
                "s" => {
                    spell_attack(player, &mut enemy);
                    if enemy.is_defeated() {
                        return handle_enemy_defeat(player, &enemy, loot_tables);
                    }
                    combat_action_message = format!(
                        "You cast a spell on the {} for {} damage!\nThe {} hits you for {} damage!",
                        enemy.name, 15, enemy.name, enemy.attack
                    );
                },
                "i" => {
                    // Show inventory and manage usage as a separate view
                    handle_inventory_in_combat(player);

                    // Re-render combat screen after inventory closes
                    continue; // Do not advance combat, re-render combat screen
                },
                "r" => {
                    if rng.gen_bool(0.5) {
                        info!("Player successfully ran away from combat.");
                        println!("\nYou successfully ran away!");
                        return "Ran away from combat.".to_string();
                    } else {
                        combat_action_message = format!(
                            "You attempted to run away but failed!\nThe {} hits you for {} damage!",
                            enemy.name, enemy.attack
                        );

                        // Let the enemy attack if running away fails
                        enemy.attack_player(&mut player.health);
                        debug!("{} hit player for {} damage", enemy.name, enemy.attack);

                        // Check if the player has been defeated
                        if player.health <= 0 {
                            info!("Player has been defeated by {}", enemy.name);
                            return handle_player_defeat(player, &enemy);
                        }

                        continue; // Update message and continue to next round
                    }
                },
                _ => {
                    println!("\nInvalid command. Please enter 'm', 'c', 's', 'i', or 'r'.");
                    continue; // Invalid command, ask again
                }
            }

            // Enemy attacks if not defeated
            enemy.attack_player(&mut player.health);
            debug!("{} hit player for {} damage", enemy.name, enemy.attack);
        }

        // Check if player is defeated
        if player.health <= 0 {
            info!("Player has been defeated by {}", enemy.name);
            return handle_player_defeat(player, &enemy);
        }
    }
}

fn main_attack(player: &mut Player, enemy: &mut Enemy) {
    let damage = 10; // Example main attack damage
    enemy.take_damage(damage);
    println!("\nYou hit the {} for {} damage!", enemy.name, damage);
}

fn spell_attack(player: &mut Player, enemy: &mut Enemy) {
    if player.skills.get("Magic").is_some() {
        let damage = 15; // Example magic attack damage
        enemy.take_damage(damage);
        println!("\nYou cast a spell on the {} for {} damage!", enemy.name, damage);
    } else {
        println!("\nYou don't have enough magic ability to cast a spell!");
    }
}

fn charged_attack(player: &mut Player, enemy: &mut Enemy, charge_damage: i32) {
    enemy.take_damage(charge_damage);
    debug!("Player performed a charged attack for {} damage!", charge_damage);
    println!("\nYou performed a charged attack for {} damage!", charge_damage);
}

fn handle_inventory_in_combat(player: &mut Player) {
    loop {
        // Clear terminal and display only the inventory
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        println!("\n[Consumable Items]");
        if player.inventory.is_empty() {
            println!("You have no items to use.");
        } else {
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
                println!("You have no consumable items.");
            } else {
                for (item, quantity) in &consumables {
                    println!("- {} (Quantity: {})", item.name, quantity);
                }
                println!("\nEnter the name of the item you wish to use (or press Enter to exit):");
                let mut item_name = String::new();
                io::stdin().read_line(&mut item_name).expect("Failed to read line");
                let item_name = item_name.trim();

                if item_name.is_empty() {
                    // Exit inventory
                    break;
                }

                if let Some((item, quantity)) = consumables.iter_mut().find(|(item, _)| item.name.eq_ignore_ascii_case(item_name)) {
                    if *quantity > 0 {
                        println!("\nYou used {}!", item.name);
                        player.consume_item(item.id);
                        *quantity -= 1;
                    } else {
                        println!("\nYou don't have any {} left.", item.name);
                    }
                } else {
                    println!("\nInvalid item selection.");
                }
            }
        }

        println!("\nPress Enter to return to combat...");
        io::stdin().read_line(&mut String::new()).unwrap();
        break;
    }
}

fn handle_enemy_defeat(player: &mut Player, enemy: &Enemy, loot_tables: &HashMap<String, LootTable>) -> String {
    // Clear the terminal for better readability of combat results
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    // Display the defeat information
    println!("You have defeated the {}!", enemy.name);
    let xp_gain = 10;
    player.experience += xp_gain;

    let mut loot_message = String::new();
    if let Some(loot_table) = loot_tables.get(&enemy.loot_table) {
        let loot = calculate_loot(loot_table);
        player.add_loot(&loot);

        for (item_id, quantity) in loot {
            if let Some(item) = create_items().get(&item_id) {
                loot_message.push_str(&format!("({}) {}, ", quantity, item.name));
            }
        }

        if !loot_message.is_empty() {
            loot_message.pop();
            loot_message.pop();
        }
    }

    // Display combat results and loot
    println!("\n[Combat Results]");
    println!("+{} XP", xp_gain);
    if !loot_message.is_empty() {
        println!("Looted: {}", loot_message);
    } else {
        println!("No items were looted.");
    }

    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).unwrap();

    format!("Defeated a {} | +{} XP | Looted: {}", enemy.name, xp_gain, loot_message)
}

fn handle_player_defeat(player: &mut Player, enemy: &Enemy) -> String {
    println!("You have been defeated by the {}...", enemy.name);
    player.in_combat = false;
    "You were defeated...".to_string()
}
