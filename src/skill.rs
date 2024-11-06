
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub level: i32,
    pub experience: i32,
    pub experience_to_next_level: i32,
}

impl Skill {
    pub fn new(name: &str, starting_level: i32) -> Skill {
        Skill {
            name: name.to_string(),
            level: starting_level,
            experience: 0,
            experience_to_next_level: 83,  // Starting experience required for level 2
        }
    }

    // Function to add experience to the skill
    pub fn add_experience(&mut self, amount: f32) {
        // Convert to fixed-point (round down)
        let amount_as_int = (amount * 10.0).floor() as i32;

        // Update experience
        self.experience += amount_as_int;

        // Cap the experience at 200,000,000 as specified
        if self.experience > 200_000_000 * 10 {
            self.experience = 200_000_000 * 10;
        }

        // Check for leveling up
        while self.experience >= self.experience_to_next_level * 10 && self.level < 99 {
            self.level_up();
        }
    }

    // Method to level up the skill (now made public)
    pub fn level_up(&mut self) {
        self.level += 1;
        self.experience -= self.experience_to_next_level * 10;

        // Increase experience requirement by approximately 10% for the next level
        self.experience_to_next_level = ((self.experience_to_next_level as f32) * 1.10) as i32;

        println!("Skill leveled up: {} is now level {}", self.name, self.level);
    }

    // Display skill information
    pub fn display_skill_info(&self) {
        println!("Skill: {}, Level: {}, Experience: {}", self.name, self.level, self.experience / 10);
    }
}

// Initialize default skills for the player based on the given categories
pub fn initialize_skills() -> HashMap<String, Skill> {
    let mut skills = HashMap::new();

    // Combat Skills
    skills.insert("Hitpoints".to_string(), Skill::new("Hitpoints", 1));
    skills.insert("Attack".to_string(), Skill::new("Attack", 1));
    skills.insert("Strength".to_string(), Skill::new("Strength", 1));
    skills.insert("Defense".to_string(), Skill::new("Defense", 1));
    skills.insert("Magic".to_string(), Skill::new("Magic", 1));
    skills.insert("Ranged".to_string(), Skill::new("Ranged", 1));
    skills.insert("Slayer".to_string(), Skill::new("Slayer", 1));

    // Gathering Skills
    skills.insert("Mining".to_string(), Skill::new("Mining", 1));
    skills.insert("Fishing".to_string(), Skill::new("Fishing", 1));
    skills.insert("Woodcutting".to_string(), Skill::new("Woodcutting", 1));

    // Artisan/Production Skills
    skills.insert("Cooking".to_string(), Skill::new("Cooking", 1));
    skills.insert("Smithing".to_string(), Skill::new("Smithing", 1));
    skills.insert("Crafting".to_string(), Skill::new("Crafting", 1));
    skills.insert("Herblore".to_string(), Skill::new("Herblore", 1));
    skills.insert("Runecrafting".to_string(), Skill::new("Runecrafting", 1));

    // Utility Skills
    skills.insert("Thieving".to_string(), Skill::new("Thieving", 1));
    skills.insert("Sourceries".to_string(), Skill::new("Sourceries", 1));

    skills
}
