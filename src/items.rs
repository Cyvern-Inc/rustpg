use std::collections::HashMap;
use rand::Rng;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub weight: f32,
    pub durability: Option<u32>, // Some items may have durability, others may not
    pub effect: Option<Effect>,  // Some items may have effects (e.g., healing)
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemType {
    Currency,
    Weapon,
    Armor,
    Combat,
    Consumable,
    Misc,
}

// Implement the Display trait for ItemType
impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self) // Use the Debug implementation for simplicity
    }
}

// Effect that an item might have when used
#[derive(Clone, Debug)]
pub struct Effect {
    pub health_change: i32,
    pub stamina_change: i32,
}

impl Item {
    // Create a new item with the given properties
    pub fn new(
        id: u32,
        name: &str,
        item_type: ItemType,
        weight: f32,
        durability: Option<u32>,
        effect: Option<Effect>,
    ) -> Self {
        Item {
            id,
            name: name.to_string(),
            item_type,
            weight,
            durability,
            effect,
        }
    }

    // Example function to use an item
    pub fn use_item(&self) {
        if let Some(effect) = &self.effect {
            println!(
                "{} used, Health Change: {}, Stamina Change: {}",
                self.name, effect.health_change, effect.stamina_change
            );
            // Here we would modify the player's health or stamina accordingly
        } else {
            println!("{} cannot be used in this way.", self.name);
        }
    }
}

// Basic function to create items
pub fn create_items() -> HashMap<u32, Item> {
    let mut items = HashMap::new();

    // Currency
    items.insert(
        100001,
        Item::new(100001, "Gold Coins", ItemType::Currency, 0.01, None, None),
    );
    items.insert(
        100002,
        Item::new(100002, "Silver Coins", ItemType::Currency, 0.01, None, None),
    );
    items.insert(
        100003,
        Item::new(100003, "Copper Coins", ItemType::Currency, 0.01, None, None),
    );

    // Weapons and Armor
    items.insert(
        100004,
        Item::new(100004, "Bronze Dagger", ItemType::Weapon, 1.5, Some(100), None),
    );
    items.insert(
        100008,
        Item::new(100008, "Leather Gloves", ItemType::Armor, 0.5, Some(50), None),
    );
    items.insert(
        100009,
        Item::new(100009, "Leather Boots", ItemType::Armor, 0.7, Some(60), None),
    );

    // Miscellaneous
    items.insert(
        100005,
        Item::new(100005, "Leather Scrap", ItemType::Misc, 0.2, None, None),
    );
    items.insert(
        100006,
        Item::new(100006, "Empty Vial", ItemType::Misc, 0.1, None, None),
    );
    items.insert(
        100007,
        Item::new(100007, "Small Bone", ItemType::Misc, 0.3, None, None),
    );
    items.insert(
        100013,
        Item::new(100013, "Fishing Rod", ItemType::Misc, 2.0, Some(200), None),
    );

    // Consumables
    items.insert(
        100015,
        Item::new(
            100015,
            "Raw Shrimp",
            ItemType::Consumable,
            0.3,
            None,
            Some(Effect {
                health_change: 5,
                stamina_change: 0,
            }),
        ),
    );
    items.insert(
        100016,
        Item::new(
            100016,
            "Cooked Shrimp",
            ItemType::Consumable,
            0.3,
            None,
            Some(Effect {
                health_change: 10,
                stamina_change: 5,
            }),
        ),
    );

    // Additional Consumables
    items.insert(
        100017,
        Item::new(
            100017,
            "Raw Beef",
            ItemType::Consumable,
            0.5,
            None,
            Some(Effect {
                health_change: 8,
                stamina_change: 0,
            }),
        ),
    );
    items.insert(
        100018,
        Item::new(
            100018,
            "Cooked Beef",
            ItemType::Consumable,
            0.5,
            None,
            Some(Effect {
                health_change: 20,
                stamina_change: 10,
            }),
        ),
    );

    // Basic food items
    items.insert(
        100019,
        Item::new(
            100019,
            "Cabbage",
            ItemType::Consumable,
            0.2,
            None,
            Some(Effect {
                health_change: 4,
                stamina_change: 2,
            }),
        ),
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
