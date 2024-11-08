use std::collections::HashMap;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
}

#[derive(Clone, Debug)]
pub enum ItemType {
    Currency,
    Combat,
    Consumable,
    Misc,
}

// Example function to create items
pub fn create_items() -> HashMap<u32, Item> {
    let mut items = HashMap::new();

    items.insert(100001, Item { id: 100001, name: "Gold Coins".to_string(), item_type: ItemType::Currency });
    items.insert(100002, Item { id: 100002, name: "Silver Coins".to_string(), item_type: ItemType::Currency });
    items.insert(100003, Item { id: 100003, name: "Copper Coins".to_string(), item_type: ItemType::Currency });
    items.insert(100004, Item { id: 100004, name: "Bronze Dagger".to_string(), item_type: ItemType::Combat });
    items.insert(100005, Item { id: 100005, name: "Leather Scrap".to_string(), item_type: ItemType::Misc });
    items.insert(100006, Item { id: 100006, name: "Empty Vile".to_string(), item_type: ItemType::Misc });
    items.insert(100007, Item { id: 100007, name: "Small Bone".to_string(), item_type: ItemType::Misc });
    items.insert(100008, Item { id: 100008, name: "Gloves".to_string(), item_type: ItemType::Combat });
    items.insert(100009, Item { id: 100009, name: "Boots".to_string(), item_type: ItemType::Combat });
    items.insert(100010, Item { id: 100010, name: "Bronze Pickaxe".to_string(), item_type: ItemType::Combat });
    items.insert(100011, Item { id: 100011, name: "Bronze Hatchet".to_string(), item_type: ItemType::Combat });
    items.insert(100012, Item { id: 100012, name: "Bronze Chainmail".to_string(), item_type: ItemType::Combat });
    items.insert(100013, Item { id: 100013, name: "Fishing Rod".to_string(), item_type: ItemType::Misc });
    items.insert(100015, Item { id: 100015, name: "Raw Shrimp".to_string(), item_type: ItemType::Consumable });
    items.insert(100016, Item { id: 100016, name: "Shrimp".to_string(), item_type: ItemType::Consumable });
    items.insert(100017, Item { id: 100017, name: "Raw Beef".to_string(), item_type: ItemType::Consumable });
    items.insert(100018, Item { id: 100018, name: "Beef".to_string(), item_type: ItemType::Consumable });
    items.insert(100019, Item { id: 100019, name: "Cabbage".to_string(), item_type: ItemType::Consumable });

    items
}

// Example Loot Table Struct
#[derive(Debug, Clone)]
pub struct LootTable {
    pub items: Vec<(u32, Option<(u32, u32)>, f32)>, // (Item ID, Optional Quantity Range, Weight)
}

pub fn create_loot_tables() -> HashMap<String, LootTable> {
    let mut loot_tables = HashMap::new();

    loot_tables.insert("common".to_string(), LootTable {
        items: vec![
            (100001, Some((1, 1)), 0.5),      // 1 gold coin [0.5%]
                       (100002, Some((1, 3)), 25.0),     // 1-3 silver coins [25%]
                       (100003, Some((1, 3)), 15.0),     // 1-3 copper coins [15%]
                       (100004, Some((1, 1)), 15.0),     // 1 bronze dagger [15%]
                       (100005, Some((1, 2)), 25.0),     // 1-2 leather scraps [25%]
                       (100006, Some((1, 1)), 10.0),     // 1 empty vile [10%]
                       (100007, Some((1, 3)), 80.0),     // 1-3 small bones [80%]
                       (0, None, 5.0),                  // Nothing [5%]
        ],
    });

    loot_tables
}

// Function to calculate loot
pub fn calculate_loot(loot_table: &LootTable) -> HashMap<u32, u32> {
    let mut rng = rand::thread_rng();
    let mut loot_result = HashMap::new();

    for &(item_id, quantity_range, drop_chance) in &loot_table.items {
        let roll: f32 = rng.gen_range(0.0..100.0);
        if roll < drop_chance {
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
