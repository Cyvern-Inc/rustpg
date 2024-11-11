use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub level: i32,
    pub experience: f64,
}

impl Skill {
    pub fn new(name: &str, starting_level: i32) -> Skill {
        Skill {
            name: name.to_string(),
            level: starting_level,
            experience: 0.0,
        }
    }

    pub fn add_experience(&mut self, amount: f64) {
        self.experience += amount;
        if self.experience > 200_000_000.0 {
            self.experience = 200_000_000.0;
        }
        while self.experience >= xp_for_level((self.level + 1) as u32) && self.level < 99 {
            self.level += 1;
            println!("Skill leveled up: {} is now level {}", self.name, self.level);
        }
    }

    pub fn display_skill_info(&self) {
        println!(
            "Skill: {}, Level: {}, Experience: {}",
            self.name,
            self.level,
            self.experience / 10.0 // Use 10.0 if division is necessary
        );
    }
}

fn xp_for_level(level: u32) -> f64 {
    let mut xp = 0.0;

    for i in 1..level {
        let value = (i as f64) + (300.0 * 2f64.powf(i as f64 / 7.0));
        xp += value.floor() / 4.0;
    }

    xp
}

// Initialize default skills for the player based on the given categories
pub fn initialize_skills() -> HashMap<String, Skill> {
    let mut skills = HashMap::new();
    // Combat Skills
    skills.insert("Hitpoints".to_string(), Skill::new("Hitpoints", 1));
    skills.insert("Attack".to_string(), Skill::new("Attack", 1));
    skills.insert("Strength".to_string(), Skill::new("Strength", 1));
    skills.insert("Magic".to_string(), Skill::new("Magic", 1));
    skills.insert("Slaying".to_string(), Skill::new("Slaying", 1));
    skills.insert("Adventuring".to_string(), Skill::new("Adventuring", 1));
    // Gathering Skills
    skills.insert("Woodcutting".to_string(), Skill::new("Woodcutting", 1));
    skills.insert("Mining".to_string(), Skill::new("Mining", 1));
    skills.insert("Fishing".to_string(), Skill::new("Fishing", 1));
    skills
}

pub fn combat_xp_calculation(attack_counts: &HashMap<AttackType, usize>) -> HashMap<String, f32> {
    let mut xp_gains = HashMap::new();

    for (&attack_type, &count) in attack_counts {
        match attack_type {
            AttackType::Main => {
                let xp = (count as f32) * 10.0; // Example: 10 XP per main attack
                *xp_gains.entry("Attack".to_string()).or_insert(0.0) += xp;
            }
            AttackType::Charged => {
                let xp = (count as f32) * 20.0; // Example: 20 XP per charged attack
                *xp_gains.entry("Strength".to_string()).or_insert(0.0) += xp;
            }
            AttackType::Magic => {
                let xp = (count as f32) * 15.0; // Example: 15 XP per magic attack
                *xp_gains.entry("Magic".to_string()).or_insert(0.0) += xp;
            }
        }
    }

    xp_gains
}

// Define AttackType if not already defined
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AttackType {
    Main,
    Charged,
    Magic,
}
