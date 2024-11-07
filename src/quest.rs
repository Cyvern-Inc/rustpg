use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Quest {
    pub name: String,
    pub description: String,
    pub completed: bool,
}

impl Quest {
    pub fn new(name: &str, description: &str) -> Quest {
        Quest {
            name: name.to_string(),
            description: description.to_string(),
            completed: false,
        }
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }
}

pub fn sample_quests() -> Vec<Quest> {
    vec![
        Quest::new("Retrieve the lost sword", "Find and bring back the lost sword from the goblin camp."),
        Quest::new("Rescue the villager", "Rescue the villager from the bandits in the forest."),
    ]
}
