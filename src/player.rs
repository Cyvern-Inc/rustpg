use std::collections::HashMap;
use crate::skill::{Skill, initialize_skills};
use crate::items::create_items;
use crate::items::{ItemType, get_starting_items};
use crate::quest::Quest;

#[derive(Debug)]
pub struct Player {
    pub health: i32,
    pub max_health: i32,
    pub attack: i32,
    pub level: i32,
    pub experience: i32,
    pub inventory: HashMap<u32, u32>,
    pub skills: HashMap<String, Skill>,
    pub active_quest: Option<Quest>,
    pub in_combat: bool, // Field to track whether the player is in combat or not
}

impl Player {
    pub fn use_item_by_name(&mut self, item_name: &str) -> Option<()> {
        let items = create_items(); // Get the complete item list

        // Search for the item in the inventory by name
        if let Some((item_id, quantity)) = self.inventory.iter_mut()
            .find(|(item_id, &mut qty)| {
                if qty > 0 {
                    if let Some(item) = items.get(item_id) {
                        return item.name.eq_ignore_ascii_case(item_name);
                    }
                }
                false
            })
        {
            // If the item is consumable and has quantity, use it
            if *quantity > 0 {
                *quantity -= 1;
                println!("\nYou used {}!", item_name);
                return Some(());
            }
        }

        None // Return None if the item could not be used
    }
    pub fn new() -> Self {
        let mut player = Player {
            health: 100,
            max_health: 100, // Initialize max_health
            attack: 10,
            level: 1,
            experience: 0,
            inventory: HashMap::new(),
            skills: initialize_skills(),
            active_quest: None,
            in_combat: false, // Initially not in combat
        };
        player.add_starting_items();
        player
    }

    pub fn add_item_to_inventory(&mut self, item_id: u32, quantity: u32) {
        *self.inventory.entry(item_id).or_insert(0) += quantity;
    }

    pub fn add_starting_items(&mut self) {
        for (item_id, quantity) in get_starting_items() {
            self.add_item_to_inventory(item_id, quantity);
        }
    }

    pub fn display_inventory(&self) {
        println!("Inventory:");
        for (item_id, quantity) in &self.inventory {
            if let Some(item) = create_items().get(item_id) {
                println!("{} x{}", item.name, quantity);
            }
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health <= 0 {
            println!("Player has been defeated!");
            self.health = 0; // Ensure health does not go negative
        }
    }

    pub fn add_experience(&mut self, amount: i32) {
        self.experience += amount;
        if self.experience >= 100 {
            self.level_up();
        }
    }

    fn level_up(&mut self) {
        self.level += 1;
        self.experience = 0;
        self.attack += 5;
        self.max_health += 20; // Increase max health on level up
        self.health = self.max_health; // Restore health to max on level up
        println!("Player leveled up to level {}!", self.level);

        // Leveling up skills as well
        for skill in self.skills.values_mut() {
            skill.level_up();
        }
    }

    pub fn display_status(&self) -> String {
        let mut status = format!(
            "Health: {}/{}\nLevel: {}\nExperience: {}\nInventory:\n",
            self.health, self.max_health, self.level, self.experience
        );
        for (item_id, quantity) in &self.inventory {
            if let Some(item) = create_items().get(item_id) {
                status.push_str(&format!("  - {} x{}\n", item.name, quantity));
            }
        }
        status
    }

    // Train a skill by adding experience to it
    pub fn train_skill(&mut self, skill_name: &str, xp_gain: f32) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.add_experience(xp_gain);
            skill.display_skill_info();
        } else {
            println!("Skill not found: {}", skill_name);
        }
    }

    // Add loot to player's inventory
    pub fn add_loot(&mut self, loot: &HashMap<u32, u32>) {
        for (&item_id, &quantity) in loot {
            *self.inventory.entry(item_id).or_insert(0) += quantity;
        }
    }

    // Consume an item from inventory
    pub fn consume_item(&mut self, item_id: u32) {
        if let Some(quantity) = self.inventory.get_mut(&item_id) {
            if *quantity > 0 {
                *quantity -= 1;
                if *quantity == 0 {
                    self.inventory.remove(&item_id);
                }
                // Apply item effects (example for health potion)
                if let Some(item) = create_items().get(&item_id) {
                    match item.item_type {
                        ItemType::Consumable => {
                            if item.name.to_lowercase().contains("potion") {
                                self.health += 30; // Assume potion restores 30 health
                                if self.health > self.max_health {
                                    self.health = self.max_health; // Cap health at max health
                                }
                                println!("You used {} and restored health. Current health: {}/{}", item.name, self.health, self.max_health);
                            }
                        }
                        _ => println!("Item used: {}", item.name),
                    }
                }
            } else {
                println!("You don't have any {} left.", create_items().get(&item_id).unwrap().name);
            }
        } else {
            println!("Item not found in inventory.");
        }
    }

    // Method to display only consumable items
    pub fn display_consumables(&self) {
        println!("\n[Consumable Items]");
        let items = create_items();
        let mut found = false;
        for (item_id, quantity) in &self.inventory {
            if let Some(item) = items.get(item_id) {
                if matches!(item.item_type, ItemType::Consumable) && *quantity > 0 {
                    println!("- {} (Quantity: {})", item.name, quantity);
                    found = true;
                }
            }
        }
        if !found {
            println!("You have no consumable items.");
        }
    }

    // Method to handle player entering combat
    pub fn enter_combat(&mut self) {
        self.in_combat = true;
    }

    // Method to handle player exiting combat
    pub fn exit_combat(&mut self) {
        self.in_combat = false;
    }
}
