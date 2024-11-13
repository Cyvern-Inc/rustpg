use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum QuestCondition {
    KillEnemy(String), // Enemy name to kill
    // Add more conditions as needed:
    // CollectItem(u32, u32), // (item_id, amount)
    // ReachLocation(usize, usize), // (x, y)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub is_completed: bool,
    pub condition: QuestCondition,
    pub progress: Option<(u32, u32)>, // (current, total) progress
}

impl Quest {
    pub fn new(id: u32, name: &str, description: &str, condition: QuestCondition) -> Quest {
        let progress = match &condition {
            QuestCondition::KillEnemy(_) => Some((0, 10)), // Example: Kill 10 enemies
            // Add other conditions with their progress requirements
        };

        Quest {
            id,
            name: name.to_string(),
            description: description.to_string(),
            is_completed: false,
            condition,
            progress,
        }
    }

    pub fn complete(&mut self) {
        self.is_completed = true;
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    pub fn get_progress_text(&self) -> Option<String> {
        self.progress.map(|(current, total)| {
            format!("{}/{}", current, total)
        })
    }

    pub fn check_progress(&mut self, trigger: Option<&str>) -> bool {
        if self.is_completed {
            return false;
        }

        if let Some((current, total)) = self.progress {
            match (&self.condition, trigger) {
                (QuestCondition::KillEnemy(target), Some(enemy_name)) if target == enemy_name => {
                    let new_progress = current + 1;
                    self.progress = Some((new_progress, total));
                    if new_progress >= total {
                        self.complete();
                        return true;
                    }
                }
                // Add other condition matches here
                _ => {}
            }
        }
        false
    }
}

pub fn starting_quest() -> Quest {
    Quest::new(
        1,
        "Starting Off",
        "Kill a Goblin to complete your training.",
        QuestCondition::KillEnemy("Goblin".to_string())
    )
}

pub fn sample_quests() -> Vec<Quest> {
    vec![starting_quest()]
}