use serde::{Serialize, Deserialize};
use std::sync::OnceLock;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ObjectiveKind {
    /// Kill enemies matching `enemy_name`. Use "any" to match all enemy types.
    KillEnemy { enemy_name: String },
    /// Have at least `required` of `item_id` in the player's inventory.
    HaveItem { item_id: u32 },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub description: String,
    pub required: u32,
    pub current: u32,
}

impl Objective {
    pub fn is_complete(&self) -> bool {
        self.current >= self.required
    }

    pub fn progress_str(&self) -> String {
        format!("{}/{}", self.current, self.required)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum QuestReward {
    Experience(i32),
    Item(u32, u32), // item_id, quantity
    QuestPoints(u32),
}

/// A conditional item drop added to an enemy's loot pool while a quest is active.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QuestDrop {
    /// Enemy type that can drop this item ("Goblin", "any", etc.).
    pub enemy_name: String,
    pub item_id: u32,
    /// 1-in-N chance of dropping on each kill.
    pub chance: u32,
    /// Don't drop if the player already has this many in inventory.
    pub max_in_inventory: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub objectives: Vec<Objective>,
    pub rewards: Vec<QuestReward>,
    pub completed: bool,
    /// ID of the quest that must be completed before this one is given to the player.
    /// None means it is available from the start.
    pub prerequisite_id: Option<u32>,
    /// When false this quest is never auto-assigned — it must be given via dialogue.
    /// Defaults to true so existing quests are unaffected.
    #[serde(default = "default_true")]
    pub auto_assign: bool,
    /// Conditional drops injected into enemy loot while this quest is active.
    #[serde(default)]
    pub drops: Vec<QuestDrop>,
    /// Whether completing all objectives automatically marks the quest done.
    /// Set to false for quests that require a dialogue turn-in to complete.
    #[serde(default = "default_true")]
    pub auto_complete: bool,
}

fn default_true() -> bool { true }

impl Quest {
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn objectives_met(&self) -> bool {
        self.objectives.iter().all(|o| o.is_complete())
    }

    /// True if this quest is active (given to the player and not yet complete).
    pub fn is_active(&self) -> bool {
        !self.completed
    }
}

/// Look up a quest definition by id from the global registry.
pub fn quest_by_id(id: u32) -> Option<Quest> {
    all_quests().iter().find(|q| q.id == id).cloned()
}

// --------------------------------------------------------------------------
// Global quest definitions
// --------------------------------------------------------------------------

static ALL_QUESTS: OnceLock<Vec<Quest>> = OnceLock::new();

/// Returns the master list of every quest in the game. Initialised once.
pub fn all_quests() -> &'static Vec<Quest> {
    ALL_QUESTS.get_or_init(define_quests)
}

/// Returns the quest the player starts a new game with (auto-assigned, no prerequisite).
pub fn starting_quest() -> Quest {
    all_quests()
        .iter()
        .find(|q| q.prerequisite_id.is_none() && q.auto_assign)
        .expect("no starting quest defined")
        .clone()
}

fn define_quests() -> Vec<Quest> {
    vec![
        // ----------------------------------------------------------------
        // Quest 1 — tutorial / first kill
        // ----------------------------------------------------------------
        Quest {
            id: 1,
            name: "Starting Off".to_string(),
            description: "The world is vast and dangerous. Prove yourself by defeating an enemy.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "any".to_string() },
                    description: "Defeat any enemy".to_string(),
                    required: 1,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(50),
                QuestReward::Item(100003, 25), // 25 Copper Coins
            ],
            completed: false,
            prerequisite_id: None,
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 2 — goblin arc opener (unlocked after quest 1)
        // ----------------------------------------------------------------
        Quest {
            id: 2,
            name: "The Goblin Menace".to_string(),
            description: "Goblins have been raiding nearby settlements. Drive them back.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Goblin".to_string() },
                    description: "Defeat Goblins".to_string(),
                    required: 5,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(150),
                QuestReward::Item(100002, 10), // 10 Silver Coins
            ],
            completed: false,
            prerequisite_id: Some(1),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 3 — main storyline: the lost sword (unlocked after quest 2)
        // ----------------------------------------------------------------
        Quest {
            id: 3,
            name: "The Lost Sword".to_string(),
            description: "A goblin chief was spotted carrying a rusty old sword stolen from a local blacksmith. Cut through the goblin camp and retrieve it.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Goblin".to_string() },
                    description: "Defeat Goblins".to_string(),
                    required: 15,
                    current: 0,
                },
                Objective {
                    kind: ObjectiveKind::HaveItem { item_id: 100023 },
                    description: "Find the Rusty Sword".to_string(),
                    required: 1,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(300),
                QuestReward::Item(100001, 5), // 5 Gold Coins
            ],
            completed: false,
            prerequisite_id: Some(2),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 4 — bandit arc (unlocked after quest 2, parallel to quest 3)
        // ----------------------------------------------------------------
        Quest {
            id: 4,
            name: "Outlaw Justice".to_string(),
            description: "Bandits have set up camp on the roads. Travellers are in danger — clear them out.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Bandit".to_string() },
                    description: "Defeat Bandits".to_string(),
                    required: 5,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(200),
                QuestReward::Item(100002, 5), // 5 Silver Coins
                QuestReward::Item(100005, 3), // 3 Leather Scraps
            ],
            completed: false,
            prerequisite_id: Some(2),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 5 — bestiary quest (unlocked after quest 2, parallel to quest 3)
        // ----------------------------------------------------------------
        Quest {
            id: 5,
            name: "Know Your Enemy".to_string(),
            description: "A seasoned adventurer knows what lurks in the wilderness. Hunt down an Orc, a Wolf, and a Skeleton.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Orc".to_string() },
                    description: "Defeat an Orc".to_string(),
                    required: 1,
                    current: 0,
                },
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Wolf".to_string() },
                    description: "Defeat a Wolf".to_string(),
                    required: 1,
                    current: 0,
                },
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Skeleton".to_string() },
                    description: "Defeat a Skeleton".to_string(),
                    required: 1,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(250),
                QuestReward::Item(100002, 10), // 10 Silver Coins
            ],
            completed: false,
            prerequisite_id: Some(2),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 6 — boss quest (unlocked after completing quest 4)
        // ----------------------------------------------------------------
        Quest {
            id: 6,
            name: "Trollshead".to_string(),
            description: "A massive troll has been terrorising the wilderness. Put it down before it reaches the settlements.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::KillEnemy { enemy_name: "Troll".to_string() },
                    description: "Defeat a Troll".to_string(),
                    required: 1,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(500),
                QuestReward::Item(100001, 20), // 20 Gold Coins
                QuestReward::Item(100009, 1),  // Leather Boots
            ],
            completed: false,
            prerequisite_id: Some(4),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 7 — woodcutting intro (unlocked after quest 1)
        // ----------------------------------------------------------------
        Quest {
            id: 7,
            name: "First Lumber".to_string(),
            description: "A nearby woodsman suggests you try your hand at cutting trees. Equip your hatchet and chop down a tree to collect your first log.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::HaveItem { item_id: 100022 },
                    description: "Collect a Log".to_string(),
                    required: 1,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(25),
                QuestReward::Item(100003, 20), // 20 Copper Coins
            ],
            completed: false,
            prerequisite_id: Some(1),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 8 — woodcutting sustained (unlocked after quest 7)
        // ----------------------------------------------------------------
        Quest {
            id: 8,
            name: "The Woodsman".to_string(),
            description: "The woodsman needs a supply of logs for the coming winter. Stock up a good pile for him.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::HaveItem { item_id: 100022 },
                    description: "Collect Logs".to_string(),
                    required: 20,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::Experience(100),
                QuestReward::Item(100002, 5), // 5 Silver Coins
            ],
            completed: false,
            prerequisite_id: Some(7),
            auto_assign: true,
            auto_complete: true,
            drops: vec![],
        },

        // ----------------------------------------------------------------
        // Quest 9 — Woodsman: goblin head (dialogue-given, dialogue turn-in)
        // ----------------------------------------------------------------
        Quest {
            id: 9,
            name: "A Curious Request".to_string(),
            description: "The woodsman is studying the local goblin population and has asked you to bring him a goblin head for examination. He promises a reward.".to_string(),
            objectives: vec![
                Objective {
                    kind: ObjectiveKind::HaveItem { item_id: 100052 }, // Goblin Head
                    description: "Obtain a Goblin Head".to_string(),
                    required: 1,
                    current: 0,
                },
            ],
            rewards: vec![
                QuestReward::QuestPoints(1),
                QuestReward::Item(100053, 1),  // Goblin Mask
                QuestReward::Item(100001, 10), // 10 Gold Coins
                QuestReward::Item(100016, 20), // 20 Cooked Shrimp
            ],
            completed: false,
            prerequisite_id: None,
            auto_assign: false,   // given only through dialogue with the Woodsman
            auto_complete: false, // completed only through dialogue turn-in
            drops: vec![
                QuestDrop {
                    enemy_name: "Goblin".to_string(),
                    item_id: 100052, // Goblin Head
                    chance: 10,      // 1-in-10
                    max_in_inventory: 1,
                },
            ],
        },
    ]
}
