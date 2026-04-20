use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Enemy {
    pub name: String,
    pub health: i32,
    pub attack: i32,
    pub loot_table: String, // Added loot_table
}

impl Enemy {
    pub fn new(name: &str, health: i32, attack: i32, loot_table: &str) -> Enemy {
        Enemy {
            name: name.to_string(),
            health,
            attack,
            loot_table: loot_table.to_string(), // Initialize loot_table here
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health < 0 {
            self.health = 0;
        }
    }

    pub fn is_defeated(&self) -> bool {
        self.health <= 0
    }

    pub fn attack_player(&self, player_health: &mut i32) {
        *player_health -= self.attack;
        if *player_health < 0 {
            *player_health = 0;
        }
    }
}

pub fn basic_enemies() -> Vec<Enemy> {
    let content = fs::read_to_string("data/enemies.json")
        .expect("Could not read data/enemies.json");
    serde_json::from_str(&content).expect("Failed to parse data/enemies.json")
}
