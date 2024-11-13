use crate::items::create_items;
use crate::items::get_starting_items;
use crate::items::{EquipmentSlot, Item, ItemType}; // Add EquipmentSlot here
use crate::map::{Direction, Map};
use crate::quest::Quest;
use crate::skill::{initialize_skills, Skill};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};

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
    #[serde(default)]
    pub equipped_items: HashMap<EquipmentSlot, Item>, // Ensure default for missing field
}

impl Player {
    pub fn new() -> Self {
        let mut player = Player {
            health: 100,
            max_health: 100,
            attack: 10,
            level: 0, // Will be set by update_total_level
            experience: 0,
            quests: vec![],
            inventory: HashMap::new(),
            equipped_weapon: None,
            equipped_armor: None,
            skills: initialize_skills(),
            active_quest: None,
            in_combat: false,
            facing: Direction::Down, // Initially facing south
            x: 0,                    // Default position
            y: 0,
            equipped_items: HashMap::new(),
        };
        player.add_starting_items();
        player.update_total_level(); // Set initial total level
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

    pub fn display_quests(&self) -> String {
        let mut output = String::new();

        output.push_str("\n=== Active Quests ===\n");
        let active_quests: Vec<_> = self.quests.iter().filter(|q| !q.is_completed()).collect();

        if active_quests.is_empty() {
            output.push_str("No active quests\n");
        } else {
            for quest in active_quests {
                output.push_str(&format!("- {}\n", quest.name));
                output.push_str(&format!("  {}\n", quest.description));
                if let Some(progress) = quest.get_progress_text() {
                    output.push_str(&format!("  Progress: {}\n", progress));
                }
            }
        }

        output.push_str("\n=== Completed Quests ===\n");
        let completed_quests: Vec<_> = self.quests.iter().filter(|q| q.is_completed()).collect();

        if completed_quests.is_empty() {
            output.push_str("No completed quests\n");
        } else {
            for quest in completed_quests {
                output.push_str(&format!("- {} (Completed)\n", quest.name));
            }
        }

        output
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
        let mut status = String::new();
        
        // Clear the terminal and render menu (unchanged)
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        println!("(w/a/s/d) move | (status) player status | (quests) view quests");
        println!("(i) inventory | (m) menu | (q) quit");
        println!();
    
        // Left Column: Health, Level, Experience, Skills
        let left_column = vec![
            format!("Health:    {}/{}", self.health, self.max_health),
            format!("Level:     {}", self.level),
            format!("Experience: {}", self.experience),
            String::from("Skills:"),
        ];
    
        // Skills Lines
        let mut skills_lines: Vec<String> = Vec::new();
        for (skill_name, skill) in &self.skills {
            skills_lines.push(format!(
                "- {}: Level {} (XP: {})",
                skill_name, skill.level, skill.experience
            ));
        }
    
        // Combine Health, Level, Experience with Skills
        let mut left_combined = left_column.clone();
        left_combined.extend(skills_lines);
    
        // Right Column: Active Quests and Inventory
        let mut right_combined = vec![String::from("Active Quests:")];
        
        // Add active quests to right column
        let active_quests: Vec<_> = self.quests.iter()
            .filter(|q| !q.is_completed())
            .collect();
        
        if active_quests.is_empty() {
            right_combined.push(String::from("  No active quests"));
        } else {
            for quest in active_quests {
                right_combined.push(format!("- {}", quest.name));
            }
        }
        
        // Add separator between quests and inventory
        right_combined.push(String::new());
        right_combined.push(String::from("Inventory:"));
        
        // Add inventory items
        for (item_id, quantity) in &self.inventory {
            if let Some(item) = create_items().get(item_id) {
                right_combined.push(format!("- {} x{}", item.name, quantity));
            }
        }
    
        // Combine columns side by side
        let max_lines = left_combined.len().max(right_combined.len());
        for i in 0..max_lines {
            let left = if i < left_combined.len() {
                &left_combined[i]
            } else {
                ""
            };
            let right = if i < right_combined.len() {
                &right_combined[i]
            } else {
                ""
            };
            status.push_str(&format!("{:<40} {}\n", left, right));
        }
    
        status
    }

    pub fn interact(&self, map: &Map) -> Option<String> {
        map.interact(self)
    }

    // Train a skill by adding experience to it
    pub fn train_skill(&mut self, skill_name: &str, xp_gain: f32) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.add_experience(xp_gain as f64);
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
        self.skills.values().map(|skill| skill.level).sum()
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
            let old_level = skill.level;
            skill.add_experience(amount as f64);

            // If skill level changed, update total level
            if skill.level != old_level {
                self.update_total_level();
            }
        }
    }

    // Method to set position
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    pub fn equip_item(&mut self, item: &Item) -> Result<String, String> {
        if (!self.inventory.contains_key(&item.id)) {
            return Err("You don't have this item in your inventory.".to_string());
        }

        let slot = match item.equipment_slot {
            Some(slot_num) => match EquipmentSlot::from_slot_number(slot_num) {
                Some(slot) => slot,
                None => return Err("Invalid equipment slot".to_string()),
            },
            None => return Err("This item cannot be equipped".to_string()),
        };

        if let Some(old_item) = self.equipped_items.get(&slot) {
            if old_item.id == item.id {
                return Err("This item is already equipped".to_string());
            }
        }

        self.equipped_items.insert(slot, item.clone());
        Ok(format!(
            "Equipped {} in {} slot",
            item.name,
            format!("{:?}", slot).to_lowercase()
        ))
    }

    pub fn unequip_item(&mut self, item_name: &str) -> Result<String, String> {
        let slot_option = self.equipped_items.iter().find_map(|(slot, item)| {
            if item.name.eq_ignore_ascii_case(item_name) {
                Some(*slot)
            } else {
                None
            }
        });

        if let Some(slot) = slot_option {
            if let Some(item) = self.equipped_items.remove(&slot) {
                Ok(format!("Unequipped {}", item.name))
            } else {
                Err("Failed to unequip item.".to_string())
            }
        } else {
            Err("Item not equipped.".to_string())
        }
    }

    pub fn update_total_level(&mut self) {
        self.level = self.total_level();
    }
}
