use std::collections::HashMap;
use crate::skill::{Skill, initialize_skills}; // Proper import for Skill and initialize_skills
use crate::items::create_items; // Access item information
use crate::items::{Item, ItemType};

pub struct Player {
    pub health: i32,
    pub attack: i32,
    pub level: i32,
    pub experience: i32,
    pub inventory: HashMap<u32, i32>, // Item ID to Quantity
    pub skills: HashMap<String, Skill>,
}

impl Player {
    pub fn new() -> Player {
        Player {
            health: 100,
            attack: 10,
            level: 1,
            experience: 0,
            inventory: HashMap::new(),
            skills: initialize_skills(),
        }
    }
    pub fn add_item_to_inventory(&mut self, item_id: u32, quantity: i32) {
        *self.inventory.entry(item_id).or_insert(0) += quantity;
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
        println!("Player leveled up to level {}!", self.level);

        // Leveling up skills as well
        for skill in self.skills.values_mut() {
            skill.level_up();
        }
    }

    pub fn display_status(&self) -> String {
        let mut status = format!(
            "Health: {}\nLevel: {}\nExperience: {}\nInventory:\n",
            self.health, self.level, self.experience
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
}
