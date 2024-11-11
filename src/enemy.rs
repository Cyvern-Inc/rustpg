use serde::{Serialize, Deserialize};

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

// Function to create some basic enemies
pub fn basic_enemies() -> Vec<Enemy> {
    vec![
        Enemy::new("Goblin", 30, 5, "common"),       // Goblin drops from common loot table
        Enemy::new("Orc", 50, 10, "uncommon"),       // Orc drops from uncommon loot table
        Enemy::new("Bandit", 40, 8, "common_food"),       // Bandit drops from common loot table
        Enemy::new("Wolf", 35, 7, "uncommon"),    // Wolf drops from common_food loot table
        Enemy::new("Skeleton", 45, 9, "uncommon"),   // Skeleton drops from uncommon loot table
        Enemy::new("Troll", 80, 15, "rare"),         // Troll drops from rare loot table
    ]
}
