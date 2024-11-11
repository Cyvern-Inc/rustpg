// Core game components
use crate::enemy::Enemy;
use crate::player::Player;
use crate::skill::{combat_xp_calculation, AttackType};

// Inventory system
use crate::inventory::display_and_handle_inventory;

// Item system
use crate::items::{calculate_loot, create_items, LootTable};

// External crates
use log::{debug, info};
use rand::Rng;
use std::collections::HashMap;
use std::io::{self, Write};

pub fn handle_combat(
    player: &mut Player,
    mut enemy: Enemy,
    loot_tables: &HashMap<String, LootTable>,
) -> String {
    info!("Entering combat with {}", enemy.name);
    let mut rng = rand::thread_rng();
    let mut charging = false;
    let mut charge_damage = 0;
    let mut combat_action_message = String::new();

    // Introduce attack_counts to keep track of attack types
    let mut attack_counts: HashMap<AttackType, usize> = HashMap::new();

    loop {
        // Clear terminal for better user experience
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the top menu
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit\n");

        // Display combat status and previous round actions
        if combat_action_message.is_empty() {
            println!("\nYou've encountered a {}!", enemy.name);
        } else {
            println!("\n{}", combat_action_message);
        }

        println!();

        // Display enemy and player health
        println!("Enemy: {} (Health: {})", enemy.name, enemy.health);
        println!("Your health: {}\n", player.health);

        // Handle the charged attack
        if charging {
            charged_attack(player, &mut enemy, charge_damage);
            *attack_counts.entry(AttackType::Charged).or_insert(0) += 1;
            charging = false;
            charge_damage = 0;

            // Enemy attacks player
            enemy.attack_player(&mut player.health);

            // Check if combat has ended
            if let Some(result) = check_combat_end(player, &mut enemy, loot_tables, &mut attack_counts) {
                return result;
            }

            combat_action_message = format!(
                "You performed a charged attack!\nThe {} hits you back!",
                enemy.name
            );
            continue;
        } else {
            println!("Choose (m)ain, (c)harged, (s)pell, (i)tems, or (r)un?");
            let mut action = String::new();
            io::stdin()
                .read_line(&mut action)
                .expect("Failed to read line");
            let action = action.trim();

            match action {
                "m" => {
                    main_attack(player, &mut enemy);
                    *attack_counts.entry(AttackType::Main).or_insert(0) += 1;

                    // Enemy attacks player
                    enemy.attack_player(&mut player.health);

                    // Check if combat has ended
                    if let Some(result) = check_combat_end(player, &mut enemy, loot_tables, &mut attack_counts) {
                        return result;
                    }

                    combat_action_message = format!(
                        "You attacked the {}!\nThe {} hits you back!",
                        enemy.name, enemy.name
                    );
                }
                "c" => {
                    charging = true;
                    charge_damage = 30; // Example charge damage
                    combat_action_message = format!(
                        "You are charging an attack...\nThe {} hits you!",
                        enemy.name
                    );

                    // Enemy attacks while charging
                    enemy.attack_player(&mut player.health);

                    if let Some(result) = check_combat_end(player, &mut enemy, loot_tables, &mut attack_counts) {
                        return result;
                    }

                    continue;
                }
                "s" => {
                    spell_attack(player, &mut enemy);
                    *attack_counts.entry(AttackType::Magic).or_insert(0) += 1;

                    // Enemy attacks player
                    enemy.attack_player(&mut player.health);

                    // Check if combat has ended
                    if let Some(result) = check_combat_end(player, &mut enemy, loot_tables, &mut attack_counts) {
                        return result;
                    }

                    combat_action_message = format!(
                        "You cast a spell on the {}!\nThe {} hits you back!",
                        enemy.name, enemy.name
                    );
                }
                "i" => {
                    display_and_handle_inventory(player, None);
                    continue;
                }
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

                        enemy.attack_player(&mut player.health);
                        debug!("{} hit player for {} damage", enemy.name, enemy.attack);

                        if player.health <= 0 {
                            info!("Player has been defeated by {}", enemy.name);
                            combat_action_message
                                .push_str("\nNo experience is gained from defeat.");
                            return handle_player_defeat(player, &enemy);
                        }

                        continue;
                    }
                }
                _ => {
                    println!("\nInvalid command. Please enter 'm', 'c', 's', 'i', or 'r'.");
                    continue;
                }
            }

            // Enemy attacks player after player's action
            enemy.attack_player(&mut player.health);
            debug!("{} hit player for {} damage", enemy.name, enemy.attack);

            if player.health <= 0 {
                info!("Player has been defeated by {}", enemy.name);
                combat_action_message.push_str("\nNo experience is gained from defeat.");
                return handle_player_defeat(player, &enemy);
            }
        }

        if charging && enemy.is_defeated() {
            // Calculate XP gains
            let xp_gains = combat_xp_calculation(&attack_counts);
            
            // Iterate by reference to avoid moving xp_gains
            for (skill_name, xp) in &xp_gains {
                if let Some(skill) = player.skills.get_mut(skill_name) {
                    skill.add_experience(*xp as f64);
                }
            }
            // Clear attack_counts after handling XP
            attack_counts.clear();

            // Pass a reference to xp_gains
            return handle_enemy_defeat(player, &enemy, loot_tables, &xp_gains);
        }

        if enemy.is_defeated() {
            let xp_gains = combat_xp_calculation(&attack_counts);
            for (skill_name, xp) in &xp_gains {
                if let Some(skill) = player.skills.get_mut(skill_name) {
                    skill.add_experience(*xp as f64);
                }
            }
            attack_counts.clear();
            return handle_enemy_defeat(player, &enemy, loot_tables, &xp_gains);
        }
    }
}

fn main_attack(player: &mut Player, enemy: &mut Enemy) {
    let mut damage = 10; // Base damage for main attack

    // Increase damage if a weapon is equipped
    if let Some(weapon) = &player.equipped_weapon {
        damage += weapon.attack_bonus.unwrap_or(0);
        println!(
            "\nYou attack with your {} for {} damage!",
            weapon.name, damage
        );
    } else {
        println!("\nYou attack with your fists for {} damage!", damage);
    }

    enemy.take_damage(damage);
}

fn spell_attack(player: &mut Player, enemy: &mut Enemy) {
    if player.skills.get("Magic").is_some() {
        let damage = 15; // Example magic attack damage
        enemy.take_damage(damage);
        println!(
            "\nYou cast a spell on the {} for {} damage!",
            enemy.name, damage
        );
    } else {
        println!("\nYou don't have enough magic ability to cast a spell!");
    }
}

fn charged_attack(_player: &mut Player, enemy: &mut Enemy, charge_damage: i32) {
    // Prefixed unused variable with underscore
    enemy.take_damage(charge_damage);
    debug!(
        "Player performed a charged attack for {} damage!",
        charge_damage
    );
    println!(
        "\nYou performed a charged attack for {} damage!",
        charge_damage
    );
}

fn handle_enemy_defeat(
    player: &mut Player,
    enemy: &Enemy,
    loot_tables: &HashMap<String, LootTable>,
    xp_gains: &HashMap<String, f32>,
) -> String {
    // Clear the terminal for better readability of combat results
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    // Display the defeat information
    println!("You have defeated the {}!", enemy.name);

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
    
    // Display XP gains to the player
    for (skill_name, xp) in xp_gains {
        println!("{} gained {} XP.", skill_name, xp);
    }

    if !loot_message.is_empty() {
        println!("Looted: {}", loot_message);
    } else {
        println!("No items were looted.");
    }

    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).unwrap();

    format!(
        "Defeated a {} | Looted: {}",
        enemy.name, loot_message
    )
}

fn handle_player_defeat(player: &mut Player, enemy: &Enemy) -> String {
    println!("You have been defeated by the {}...", enemy.name);
    player.in_combat = false;
    "You were defeated...".to_string()
}

fn check_combat_end(
    player: &mut Player,
    enemy: &mut Enemy,
    loot_tables: &HashMap<String, LootTable>,
    attack_counts: &mut HashMap<AttackType, usize>,
) -> Option<String> {
    if enemy.is_defeated() {
        // Calculate XP gains
        let xp_gains = combat_xp_calculation(attack_counts);
        // Add XP to relevant skills
        for (skill_name, xp) in &xp_gains {
            if let Some(skill) = player.skills.get_mut(skill_name) {
                skill.add_experience(*xp as f64);
            }
        }
        attack_counts.clear();
        Some(handle_enemy_defeat(player, enemy, loot_tables, &xp_gains))
    } else if player.health <= 0 {
        attack_counts.clear(); // Clear attack counts on defeat
        Some(handle_player_defeat(player, enemy))
    } else {
        None
    }
}
