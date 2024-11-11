use crate::items::{create_items, Item, ItemType};
use crate::player::Player;
use std::io::{self, Write};

pub fn display_inventory(player: &mut Player, filter_type: Option<ItemType>) -> Option<String> {
    loop {
        // Clear the terminal screen for better user experience
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Display inventory header
        println!("\n[Inventory - Page 1/1]");
        let items = create_items(); // Create a hashmap of all game items
        let mut found = false; // Flag to check if any items are displayed

        // Display items based on filter
        for (item_id, quantity) in &player.inventory {
            if let Some(item) = items.get(item_id) {
                // Borrow `filter_type` using `as_ref()` to prevent moving
                if filter_type.as_ref().map_or(true, |f| item.item_type == *f) && *quantity > 0 {
                    println!("{:<20} x{:<8}", item.name, quantity);
                    found = true; // Set flag if at least one item is displayed
                }
            }
        }

        if !found {
            println!("No items found.");
        }

        // Prompt User for Action
        println!("\nOptions:");
        println!("  use <item_name> - Use an item");
        println!("  eat <item_name> - Eat a consumable item");
        println!("  q - Quit inventory");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim().to_lowercase(); // Trim and convert input to lowercase

        // Handle User Input
        match input.as_str() {
            "q" => return None, // Quit inventory

            cmd if cmd.starts_with("use ") => {
                let item_name = cmd.trim_start_matches("use ").trim();
                if let Some(item) = items
                    .values()
                    .find(|i| i.name.eq_ignore_ascii_case(item_name))
                {
                    if let Some(&quantity) = player.inventory.get(&item.id) {
                        if quantity > 0 {
                            match item.item_type {
                                ItemType::Consumable => {
                                    // Use consumable item
                                    let message =
                                        crate::inventory::handle_eat_command(player, &item.name);
                                    println!("\n{}", message);
                                    println!("\nPress Enter to continue...");
                                    let _ = io::stdin().read_line(&mut String::new());
                                    continue;
                                }
                                _ => {
                                    // Handle other item types if needed
                                    println!("\nCan't use that type of item.");
                                    println!("\nPress Enter to continue...");
                                    let _ = io::stdin().read_line(&mut String::new());
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Item not found or not available
                println!(
                    "\nYou don't have any '{}' to use.",
                    input.trim_start_matches("use ").trim()
                );
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
                continue;
            }

            cmd if cmd.starts_with("eat ") => {
                let item_name = cmd.trim_start_matches("eat ").trim();
                if let Some(item) = items
                    .values()
                    .find(|i| i.name.eq_ignore_ascii_case(item_name))
                {
                    if let Some(&quantity) = player.inventory.get(&item.id) {
                        if quantity > 0 {
                            match item.item_type {
                                ItemType::Consumable => {
                                    // Eat consumable item
                                    let message =
                                        crate::inventory::handle_eat_command(player, &item.name);
                                    println!("\n{}", message);
                                    println!("\nPress Enter to continue...");
                                    let _ = io::stdin().read_line(&mut String::new());
                                    continue;
                                }
                                _ => {
                                    // Cannot eat non-consumable items
                                    println!("\nYou can't eat that.");
                                    println!("\nPress Enter to continue...");
                                    let _ = io::stdin().read_line(&mut String::new());
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Item not found or not available
                println!(
                    "\nYou don't have any '{}' to eat.",
                    input.trim_start_matches("eat ").trim()
                );
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
                continue;
            }

            _ => {
                // Invalid command
                println!("\nInvalid command.");
                println!("\nPress Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
                continue;
            }
        }
    }
}

pub fn interact_with_item(player: &mut Player, item: &Item) {
    match item.item_type {
        ItemType::Consumable => interact_with_consumable(player, item),
        ItemType::Equipment => interact_with_equipment(player, item),
        ItemType::CraftingMaterial => println!("The {} is used in crafting.", item.name),
        _ => println!("The {} cannot be used directly.", item.name),
    }
    println!("Press Enter to continue...");
    let _ = io::stdin().read_line(&mut String::new());
}

fn interact_with_equipment(player: &mut Player, item: &Item) {
    // Determine if item is a weapon or armor based on item properties
    if item.name.contains("Sword") || item.name.contains("Dagger") {
        // Handle equipping or unequipping weapon
        if let Some(current_weapon) = &player.equipped_weapon {
            if current_weapon.id == item.id {
                println!("You unequip the {}.", item.name);
                player.equipped_weapon = None;
            } else {
                println!(
                    "You unequip the {} and equip the {}.",
                    current_weapon.name, item.name
                );
                player.equipped_weapon = Some(item.clone());
            }
        } else {
            println!("You equip the {}.", item.name);
            player.equipped_weapon = Some(item.clone());
        }
    } else if item.name.contains("Armor") || item.name.contains("Shield") {
        // Handle equipping or unequipping armor
        if let Some(current_armor) = &player.equipped_armor {
            if current_armor.id == item.id {
                println!("You unequip the {}.", item.name);
                player.equipped_armor = None;
            } else {
                println!(
                    "You unequip the {} and equip the {}.",
                    current_armor.name, item.name
                );
                player.equipped_armor = Some(item.clone());
            }
        } else {
            println!("You equip the {}.", item.name);
            player.equipped_armor = Some(item.clone());
        }
    } else {
        println!("The {} cannot be equipped.", item.name);
    }
}

pub fn interact_with_consumable(player: &mut Player, item: &Item) {
    println!("You consume the {}.", item.name);
    if let Some(quantity) = player.inventory.get_mut(&item.id) {
        if *quantity > 0 {
            *quantity -= 1;
            // Apply item effect, for now just heal for a basic amount (e.g., 10 HP)
            player.health = std::cmp::min(player.health + 10, player.max_health);
            println!(
                "You feel refreshed! Health: {}/{}",
                player.health, player.max_health
            );
        } else {
            println!("You don't have any {} left to use.", item.name);
        }
    }
}

pub fn use_item(player: &mut Player, item_name: &str) -> Option<String> {
    let items = create_items();

    if let Some(item) = items
        .values()
        .find(|i| i.name.eq_ignore_ascii_case(item_name))
    {
        if let Some(quantity) = player.inventory.get_mut(&item.id) {
            if *quantity > 0 {
                match item.item_type {
                    ItemType::Consumable => {
                        *quantity -= 1;
                        if *quantity == 0 {
                            player.inventory.remove(&item.id);
                        }

                        let mut message = format!("You ate the {}!", item.name);
                        if let Some(effect) = &item.effect {
                            if effect.health_change != 0 {
                                player.health = (player.health + effect.health_change)
                                    .min(player.max_health)
                                    .max(0);
                                message.push_str(&format!(
                                    "\nHealth restored: {}. Current health: {}/{}",
                                    effect.health_change, player.health, player.max_health
                                ));
                            }
                            if effect.stamina_change != 0 {
                                message.push_str(&format!(
                                    "\nStamina change: {}",
                                    effect.stamina_change
                                ));
                            }
                        }
                        return Some(message);
                    }
                    _ => return Some(format!("Used {}", item.name)),
                }
            }
        }
    }
    None
}

pub fn consume_item(player: &mut Player, item_name: &str) -> Option<String> {
    let items = create_items();

    if let Some(item) = items
        .values()
        .find(|i| i.name.eq_ignore_ascii_case(item_name))
    {
        if item.item_type != ItemType::Consumable {
            return None;
        }

        if let Some(quantity) = player.inventory.get_mut(&item.id) {
            if *quantity > 0 {
                *quantity -= 1;
                if *quantity == 0 {
                    player.inventory.remove(&item.id);
                }

                let mut message = format!("You ate the {}!", item.name);
                if let Some(effect) = &item.effect {
                    if effect.health_change != 0 {
                        player.health = (player.health + effect.health_change)
                            .min(player.max_health)
                            .max(0);
                        message.push_str(&format!(
                            "\nHealth restored: {}. Current health: {}/{}",
                            effect.health_change, player.health, player.max_health
                        ));
                    }
                    if effect.stamina_change != 0 {
                        message.push_str(&format!("\nStamina change: {}", effect.stamina_change));
                    }
                }
                return Some(message);
            }
        }
    }
    None
}

pub fn handle_eat_command(player: &mut Player, item_name: &str) -> String {
    if let Some(result) = consume_item(player, item_name) {
        result
    } else {
        "You can't eat that!".to_string()
    }
}

pub fn display_and_handle_inventory(
    player: &mut Player,
    item_type_filter: Option<ItemType>,
) -> String {
    // Display inventory
    display_inventory(player, item_type_filter);
    // Return message
        "Viewed inventory.".to_string()
}
