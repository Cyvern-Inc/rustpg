use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::skill::{Skill, initialize_skills};
use crate::items::get_starting_items;
use crate::quest::Quest;
use crate::items::{Item, ItemType};
use crate::items::create_items;
use crate::map::{Map, Direction};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub health: i32,
    pub max_health: i32,
    pub attack: i32,
    pub level: i32,
    pub experience: i32,
    pub quests: Vec<Quest>,
    pub inventory: HashMap<u32, u32>,
    pub equipped_weapon: Option<Item>,
    pub equipped_armor: Option<Item>,
    pub skills: HashMap<String, Skill>,
    pub active_quest: Option<Quest>,
    pub in_combat: bool,
    pub facing: Direction,
    pub x: usize,
    pub y: usize,
}

impl Player {
    pub fn new() -> Self {
        let mut player = Player {
            health: 100,
            max_health: 100,
            attack: 10,
            level: 1,
            experience: 0,
            quests: vec![],
            inventory: HashMap::new(),
            equipped_weapon: None,
            equipped_armor: None,
            skills: initialize_skills(),
            active_quest: None,
            in_combat: false,
            facing: Direction::Down, // Initially facing south
            x: 0, // Default position
            y: 0,
        };
        player.add_starting_items();
        player
    }

    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.push(quest);
    }

    pub fn complete_quest(&mut self, quest_id: u32) {
        if let Some(quest) = self.quests.iter_mut().find(|q| q.id == quest_id) {
            quest.complete();
            println!("Quest '{}' completed!", quest.name);
        }
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

    pub fn level_up(&mut self) {
        self.level += 1;
        self.health = self.max_health; // Restore health to max on level up
        println!("Player leveled up to level {}!", self.level);
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

    pub fn interact(&self, map: &Map) -> Option<String> {
        map.interact(self)
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

    pub fn remove_item(&mut self, item_id: u32, amount: u32) -> bool {
        if let Some(quantity) = self.inventory.get_mut(&item_id) {
            if *quantity >= amount {
                *quantity -= amount;
                if *quantity == 0 {
                    self.inventory.remove(&item_id);
                }
                return true;
            }
        }
        false
    }

    pub fn total_level(&self) -> i32 {
        self.skills.values().map(|skill| skill.level.min(99)).sum()
    }

    pub fn respawn(&mut self, map: &mut Map) {
        self.health = self.max_health;
        self.in_combat = false;
        self.facing = Direction::Down; // Reset facing direction

        map.player_x = map.campfire_x;
        
        // Safely handle player_y to prevent underflow
        if map.campfire_y > 0 {
            map.player_y = map.campfire_y - 1;
        } else {
            map.player_y = 0; // Default to top row if campfire_y is 0
        }

        // Ensure the new position is valid
        if map.player_y >= map.height {
            map.player_y = map.height - 1;
        }
    }

    pub fn add_experience_to_skill(&mut self, skill_name: &str, amount: f32) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.add_experience(amount);
            println!("{} gained {} XP.", skill_name, amount);
        } else {
            println!("Skill not found: {}", skill_name);
        }
    }

    // Method to set position
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }
}
