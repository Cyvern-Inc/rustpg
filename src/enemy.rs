use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Enemy {
    pub name: String,
    pub health: i32,
    pub attack: i32,
}

impl Enemy {
    pub fn new(name: &str, health: i32, attack: i32) -> Enemy {
        Enemy {
            name: name.to_string(),
            health,
            attack,
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

// Function to create some sample enemies for testing purposes
pub fn sample_enemies() -> Vec<Enemy> {
    vec![
        Enemy::new("Goblin", 30, 5),
        Enemy::new("Orc", 50, 10),
        Enemy::new("Bandit", 40, 8),
        Enemy::new("Wolf", 35, 7),
        Enemy::new("Skeleton", 45, 9),
        Enemy::new("Troll", 80, 15),
    ]
}
