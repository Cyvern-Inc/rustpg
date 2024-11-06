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
    }

    pub fn is_defeated(&self) -> bool {
        self.health <= 0
    }
}
