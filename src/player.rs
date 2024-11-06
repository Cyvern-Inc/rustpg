use std::collections::HashMap;
use crate::skill::{Skill, initialize_skills};

pub struct Player {
    pub health: i32,
    pub attack: i32,
    pub defense: i32,
    pub level: i32,
    pub experience: i32,
    pub inventory: Vec<String>,
    pub skills: HashMap<String, Skill>,  // Add skills to the player
}

impl Player {
    pub fn new() -> Player {
        Player {
            health: 100,
            attack: 10,
            defense: 5,
            level: 1,
            experience: 0,
            inventory: vec![],
            skills: initialize_skills(),
        }
    }

    pub fn add_item(&mut self, item: String) {
        if self.inventory.len() < 10 {
            self.inventory.push(item.clone());
            println!("You received: {}", item);
        } else {
            println!("Inventory is full! Cannot carry more items.");
        }
    }

    pub fn print_inventory(&self) {
        if self.inventory.is_empty() {
            println!("Your inventory is empty.");
        } else {
            println!("Your inventory: {:?}", self.inventory);
        }
    }

    pub fn status(&self) {
        println!("Player Status:");
        println!("Health: {}", self.health);
        println!("Attack: {}", self.attack);
        println!("Defense: {}", self.defense);
        println!("Level: {}", self.level);
        println!("Experience: {}", self.experience);

        println!("\nSkills:");
        for skill in self.skills.values() {
            println!("{}: Level {} (XP: {})", skill.name, skill.level, skill.experience);
        }
    }

    pub fn gain_skill_xp(&mut self, skill_name: &str, xp: i32) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.add_experience(xp);
        } else {
            println!("Skill not found: {}", skill_name);
        }
    }

    pub fn level_up(&mut self) {
        if self.experience >= 100 {
            self.level += 1;
            self.health += 20;
            self.attack += 5;
            self.defense += 3;
            self.experience -= 100;
            println!("You leveled up! You are now level {}", self.level);
        }
    }
}
