use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_completed: bool,
}

impl Quest {
    pub fn new(id: u32, name: &str, description: &str) -> Quest {
        Quest {
            id,
            name: name.to_string(),
            description: description.to_string(),
            is_completed: false,
        }
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }
}

pub fn starting_quest() -> Quest {
    Quest::new(1, "Starting Off", "Explore the map and defeat an enemy.")
}

pub fn sample_quests() -> Vec<Quest> {
    vec![starting_quest()]
}