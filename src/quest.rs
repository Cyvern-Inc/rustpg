
pub struct Quest {
    pub name: String,
    pub description: String,  // Now used in the game for quest display
}

impl Quest {
    pub fn new(name: &str, description: &str) -> Quest {
        Quest {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}
