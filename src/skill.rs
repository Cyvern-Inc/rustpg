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
            experience_to_next_level: Skill::calculate_experience_for_level(starting_level + 1),
        }
    }

    pub fn add_experience(&mut self, xp: i32) {
        // Cap XP at 200,000,000
        if self.experience >= 200_000_000 {
            println!("You have reached the XP cap for the {} skill.", self.name);
            return;
        }

        self.experience += xp;

        // Level up if enough XP is accumulated
        while self.experience >= self.experience_to_next_level && self.level < 99 {
            self.level_up();
        }
    }

    fn level_up(&mut self) {
        self.level += 1;
        self.experience_to_next_level = Skill::calculate_experience_for_level(self.level + 1);
        println!("Congratulations! Your {} skill is now level {}!", self.name, self.level);
    }

    fn calculate_experience_for_level(level: i32) -> i32 {
        let base_experience = 83; // Base XP for level 2
        let growth_factor = 1.10; // 10% growth per level
        let mut total_experience = base_experience as f32;

        for _ in 2..level {
            total_experience *= growth_factor;
        }

        total_experience.floor() as i32
    }
}

pub fn initialize_skills() -> HashMap<String, Skill> {
    let mut skills = HashMap::new();

    // Combat Skills
    skills.insert("Hitpoints".to_string(), Skill::new("Hitpoints", 9));
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
