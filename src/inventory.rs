use crate::items::{Item, create_items, ItemType};
use crate::player::Player;
use std::collections::HashMap;
use std::io::{self, Write};
use term_size;

pub fn display_inventory(player: &mut Player, filter: Option<ItemType>) {
    let (width, height) = term_size::dimensions().unwrap_or((80, 25));
    let items_per_column = std::cmp::min(height / 2 - 2, 8);
    let max_columns = std::cmp::min(width / 25, 4);

    let items = create_items();

    // Filter player's inventory
    let filtered_items: Vec<_> = player
    .inventory
    .iter()
    .filter(|(_, quantity)| **quantity > 0)
    .filter_map(|(item_id, quantity)| {
        if let Some(filter_type) = &filter {
            if let Some(item) = items.get(item_id) {
                if item.item_type == *filter_type {
                    return Some((item_id.clone(), *quantity));
                }
            }
            None
        } else {
            Some((item_id.clone(), *quantity))
        }
    })
    .collect();

    // Pagination logic
    let total_items = filtered_items.len();
    let mut current_page = 0;
    let items_per_page = items_per_column * max_columns;

    loop {
        // Clear terminal
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render inventory title
        println!(
            "[Inventory - Page {}/{}]",
            current_page + 1,
            (total_items + items_per_page - 1) / items_per_page
        );

        // Determine which items to display for this page
        let start_idx = current_page * items_per_page;
        let end_idx = std::cmp::min(start_idx + items_per_page, total_items);
        let items_to_display = filtered_items[start_idx..end_idx].to_vec();

        // Display items in columns
        for row in 0..items_per_column {
            for col in 0..max_columns {
                let item_idx = row + col * items_per_column;
                if item_idx < items_to_display.len() {
                    let (item_id, quantity) = &items_to_display[item_idx];
                    if let Some(item) = items.get(item_id) {
                        print!("{:<20} x{:<5}  ", item.name, quantity);
                    }
                }
            }
            println!();
        }

        // Options to navigate pages or interact with items
        println!("\n(n)ext page | (p)revious page | (use) item by name | (q)uit inventory");
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect("Failed to read line");
        let choice = choice.trim();

        match choice {
            "n" => {
                if current_page < (total_items + items_per_page - 1) / items_per_page - 1 {
                    current_page += 1;
                }
            }
            "p" => {
                if current_page > 0 {
                    current_page -= 1;
                }
            }
            "q" => break,
            _ if choice.starts_with("use ") => {
                let item_name = choice[4..].trim();

                if let Some((item_id, _)) = items_to_display.iter().find(|&&(ref id, _)| {
                    if let Some(item) = items.get(id) {
                        item.name.eq_ignore_ascii_case(item_name)
                    } else {
                        false
                    }
                }) {
                    if let Some(item) = items.get(item_id) {
                        if let Some(qty) = player.inventory.get_mut(item_id) {
                            if *qty > 0 {
                                // Update quantity before applying effect
                                *qty -= 1;

                                // Apply item effect
                                interact_with_item(player, item);

                                // Ensure no redundant prompts after interaction
                                continue;
                            } else {
                                println!("You don't have any {} left to use.", item.name);
                            }
                        }
                    } else {
                        println!("Item '{}' not found in inventory.", item_name);
                    }
                } else {
                    println!("Item '{}' not found in inventory.", item_name);
                }
                println!("Press Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
            _ => {
                println!("Invalid command. Please try again. Press Enter to continue...");
                let _ = io::stdin().read_line(&mut String::new());
            }
        }
    }
}

pub fn interact_with_item(player: &mut Player, item: &Item) {
    match item.item_type {
        ItemType::Consumable => {
            println!("You consume the {}.", item.name);
            player.health = std::cmp::min(player.health + 10, player.max_health);
            println!("You feel refreshed! Health: {}/{}", player.health, player.max_health);
        }
        ItemType::Combat => {
            println!("The {} is equipped.", item.name);
        }
        _ => {
            println!("The {} cannot be used directly.", item.name);
        }
    }
    println!("Press Enter to continue...");
    let _ = io::stdin().read_line(&mut String::new());
}
