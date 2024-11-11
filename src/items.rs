use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rand::Rng;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub weight: f32,
    pub durability: Option<u32>,
    pub effect: Option<Effect>,
    pub attack_bonus: Option<i32>,
    pub defense_bonus: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    Currency,
    Weapon,
    Armor,
    CraftingMaterial,
    Equipment,
    QuestItem,
    Combat,
    Consumable,
    Misc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    pub health_change: i32,
    pub stamina_change: i32,
}

// Implement the Display trait for ItemType
impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self) // Use the Debug implementation for simplicity
    }
}

// Function to create predefined items
pub fn create_items() -> HashMap<u32, Item> {
    let mut items = HashMap::new();

    // Currency
    items.insert(
        100001,
        Item {
            id: 100001,
            name: "Gold Coins".to_string(),
            item_type: ItemType::Currency,
            weight: 0.01,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100002,
        Item {
            id: 100002,
            name: "Silver Coins".to_string(),
            item_type: ItemType::Currency,
            weight: 0.01,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100003,
        Item {
            id: 100003,
            name: "Copper Coins".to_string(),
            item_type: ItemType::Currency,
            weight: 0.01,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );

    // Weapons and Armor
    items.insert(
        100004,
        Item {
            id: 100004,
            name: "Bronze Dagger".to_string(),
            item_type: ItemType::Weapon,
            weight: 1.5,
            durability: Some(100),
            effect: None,
            attack_bonus: Some(5),
            defense_bonus: None,
        },
    );
    items.insert(
        100008,
        Item {
            id: 100008,
            name: "Leather Gloves".to_string(),
            item_type: ItemType::Armor,
            weight: 0.5,
            durability: Some(50),
            effect: None,
            attack_bonus: None,
            defense_bonus: Some(2),
        },
    );
    items.insert(
        100009,
        Item {
            id: 100009,
            name: "Leather Boots".to_string(),
            item_type: ItemType::Armor,
            weight: 0.7,
            durability: Some(60),
            effect: None,
            attack_bonus: None,
            defense_bonus: Some(3),
        },
    );
    items.insert(
        100010,
        Item {
            id: 100010,
            name: "Bronze Pickaxe".to_string(),
            item_type: ItemType::Weapon,
            weight: 2.0,
            durability: Some(150),
            effect: None,
            attack_bonus: Some(7),
            defense_bonus: None,
        },
    );
    items.insert(
        100011,
        Item {
            id: 100011,
            name: "Bronze Hatchet".to_string(),
            item_type: ItemType::Weapon,
            weight: 2.2,
            durability: Some(130),
            effect: None,
            attack_bonus: Some(6),
            defense_bonus: None,
        },
    );

    // Miscellaneous
    items.insert(
        100005,
        Item {
            id: 100005,
            name: "Leather Scrap".to_string(),
            item_type: ItemType::Misc,
            weight: 0.2,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100006,
        Item {
            id: 100006,
            name: "Empty Vial".to_string(),
            item_type: ItemType::Misc,
            weight: 0.1,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100007,
        Item {
            id: 100007,
            name: "Small Bone".to_string(),
            item_type: ItemType::Misc,
            weight: 0.3,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100013,
        Item {
            id: 100013,
            name: "Fishing Rod".to_string(),
            item_type: ItemType::Misc,
            weight: 2.0,
            durability: Some(200),
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100020,
        Item {
            id: 100020,
            name: "Flint 'n Steel".to_string(),
            item_type: ItemType::Misc,
            weight: 0.5,
            durability: Some(75),
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100021,
        Item {
            id: 100021,
            name: "Fishing Bait".to_string(),
            item_type: ItemType::Misc,
            weight: 0.01,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100022,
        Item {
            id: 100022,
            name: "Log".to_string(),
            item_type: ItemType::Misc,
            weight: 5.0,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
        },
    );

    // Consumables
    items.insert(
        100015,
        Item {
            id: 100015,
            name: "Raw Shrimp".to_string(),
            item_type: ItemType::Consumable,
            weight: 0.3,
            durability: None,
            effect: Some(Effect {
                health_change: 5,
                stamina_change: 0,
            }),
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100016,
        Item {
            id: 100016,
            name: "Cooked Shrimp".to_string(),
            item_type: ItemType::Consumable,
            weight: 0.3,
            durability: None,
            effect: Some(Effect {
                health_change: 10,
                stamina_change: 5,
            }),
            attack_bonus: None,
            defense_bonus: None,
        },
    );

    // Additional Consumables
    items.insert(
        100017,
        Item {
            id: 100017,
            name: "Raw Beef".to_string(),
            item_type: ItemType::Consumable,
            weight: 0.5,
            durability: None,
            effect: Some(Effect {
                health_change: 8,
                stamina_change: 0,
            }),
            attack_bonus: None,
            defense_bonus: None,
        },
    );
    items.insert(
        100018,
        Item {
            id: 100018,
            name: "Cooked Beef".to_string(),
            item_type: ItemType::Consumable,
            weight: 0.5,
            durability: None,
            effect: Some(Effect {
                health_change: 20,
                stamina_change: 10,
            }),
            attack_bonus: None,
            defense_bonus: None,
        },
    );

    // Basic food items
    items.insert(
        100019,
        Item {
            id: 100019,
            name: "Cabbage".to_string(),
            item_type: ItemType::Consumable,
            weight: 0.2,
            durability: None,
            effect: Some(Effect {
                health_change: 4,
                stamina_change: 2,
            }),
            attack_bonus: None,
            defense_bonus: None,
        },
    );

    items
}

pub fn get_starting_items() -> HashMap<u32, u32> {
    let mut starting_items = HashMap::new();
    // Add the starting items
    starting_items.insert(100004, 1);  // 1 Bronze Dagger
    starting_items.insert(100019, 2);  // 2 Cabbage
    starting_items.insert(100015, 2);  // 2 Raw Shrimp
    starting_items.insert(100016, 8);  // 8 Cooked Shrimp
    starting_items.insert(100020, 1);  // 1 Flint 'n Steel
    starting_items.insert(100010, 1);  // 1 Bronze Pickaxe
    starting_items.insert(100011, 1);  // 1 Bronze Hatchet
    starting_items.insert(100013, 1);  // 1 Fishing Rod
    starting_items.insert(100021, 242); // 242 Fishing Bait
    starting_items.insert(100022, 1);  // 1 Log
    starting_items.insert(100001, 3);  // 3 Gold Coins
    starting_items.insert(100002, 12); // 12 Silver Coins
    starting_items.insert(100003, 1337); // 1337 Copper Coins

    // Return the starting_items HashMap
    starting_items
}

// Basic Loot Table Struct
#[derive(Debug, Clone)]
pub struct LootTable {
    pub items: Vec<(u32, Option<(u32, u32)>, f32)>, // (Item ID, Optional Quantity Range, Weight)
}

// Create basic loot tables using weight for item drop probability
pub fn create_loot_tables() -> HashMap<String, LootTable> {
    let mut loot_tables = HashMap::new();

    loot_tables.insert(
        "common".to_string(),
        LootTable {
            items: vec![
                (100001, Some((1, 5)), 50.0), // 1-5 Gold Coins [Weight: 50]
                (100004, Some((1, 1)), 10.0), // Bronze Dagger [Weight: 10]
                (100015, Some((1, 2)), 20.0), // Raw Shrimp [Weight: 20]
                (100005, Some((1, 3)), 15.0), // Leather Scrap [Weight: 15]
                (0, None, 5.0),               // Nothing [Weight: 5]
            ],
        },
    );

    loot_tables
}

// Function to calculate loot using weight-based approach
pub fn calculate_loot(loot_table: &LootTable) -> HashMap<u32, u32> {
    let mut rng = rand::thread_rng();
    let mut loot_result = HashMap::new();

    let total_weight: f32 = loot_table.items.iter().map(|(_, _, weight)| weight).sum();

    for &(item_id, quantity_range, weight) in &loot_table.items {
        let roll: f32 = rng.gen_range(0.0..total_weight);
        if roll < weight {
            let quantity = if let Some((min, max)) = quantity_range {
                if min == max {
                    min
                } else {
                    rng.gen_range(min..=max)
                }
            } else {
                1
            };
            *loot_result.entry(item_id).or_insert(0) += quantity;
        }
    }
    loot_result
}
