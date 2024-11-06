
use std::collections::HashMap;
use crate::skill::{Skill, initialize_skills};

pub struct Player {
    pub health: i32,
    pub attack: i32,
    pub level: i32,
    pub experience: i32,
    pub inventory: Vec<String>, // Inventory now used in the game for status display
    pub skills: HashMap<String, Skill>,  
}

impl Player {
    pub fn new() -> Player {
        Player {
            health: 100,
            attack: 10,
            level: 1,
            experience: 0,
            inventory: vec![],
            skills: initialize_skills(),
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
