use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Enemy {
    pub name: String,
    pub health: i32,
    /// Accuracy modifier added to the enemy's d20 attack roll.
    pub attack: i32,
    /// Used to calculate the enemy's Max Hit: floor(1 + strength * 0.2).
    pub strength: i32,
    /// Added to the enemy's Armour Class when the player attacks.
    pub defense: i32,
    pub loot_table: String,
}

impl Enemy {
    pub fn new(name: &str, health: i32, attack: i32, strength: i32, defense: i32, loot_table: &str) -> Enemy {
        Enemy {
            name: name.to_string(),
            health,
            attack,
            strength,
            defense,
            loot_table: loot_table.to_string(),
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
