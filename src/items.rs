use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use rand::Rng;
use std::fmt;
use std::fs;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub weight: f32,
    pub durability: Option<u32>,
    pub effect: Option<Effect>,
    /// Added to the d20 accuracy roll for melee attacks.
    #[serde(alias = "attack_bonus", default)]
    pub melee_accuracy: Option<i32>,
    /// Contributes to the Max Hit calculation: floor(1 + str*0.2 + gear_str*0.2).
    #[serde(default)]
    pub melee_strength: Option<i32>,
    /// Added to the defender's Armour Class against melee attacks.
    #[serde(alias = "defense_bonus", default)]
    pub melee_defense: Option<i32>,
    /// Added to the d20 accuracy roll for magic attacks.
    #[serde(default)]
    pub magic_accuracy: Option<i32>,
    /// Added to the defender's Armour Class against magic attacks.
    #[serde(default)]
    pub magic_defense: Option<i32>,
    pub tool_tag: Option<ToolTag>,
    /// Minimum Attack level required to equip this weapon.
    pub equip_level: Option<i32>,
    /// Minimum Defence level required to equip this armour piece.
    #[serde(default)]
    pub defence_req: Option<i32>,
    /// Descriptive weapon class (e.g. "Scimitar", "2h Sword"). Informational only.
    pub weapon_type: Option<String>,
    /// Which equipment slot this item occupies when equipped.
    /// Weapons: "weapon". Armor: "head", "body", "legs", "shield", "boots", "hands".
    pub equip_slot: Option<String>,
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

/// Identifies what skill a tool is used for. Lets skill code check for a
/// valid tool without hard-coding item IDs or names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolTag {
    Axe,
    Pickaxe,
    FishingRod,
    FishingNet,
}

// Implement the Display trait for ItemType
impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self) // Use the Debug implementation for simplicity
    }
}

static ITEMS: OnceLock<HashMap<u32, Item>> = OnceLock::new();
static LOOT_TABLES: OnceLock<HashMap<String, LootTable>> = OnceLock::new();

/// Returns a reference to the global item table, initialised once on first call.
pub fn get_items() -> &'static HashMap<u32, Item> {
    ITEMS.get_or_init(create_items)
}

/// Returns a reference to the global loot table map, initialised once on first call.
pub fn get_loot_tables() -> &'static HashMap<String, LootTable> {
    LOOT_TABLES.get_or_init(create_loot_tables)
}

fn create_items() -> HashMap<u32, Item> {
    let content = fs::read_to_string("data/items.json")
        .expect("Could not read data/items.json");
    let items_vec: Vec<Item> = serde_json::from_str(&content)
        .expect("Failed to parse data/items.json");
    items_vec.into_iter().map(|item| (item.id, item)).collect()
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
    starting_items.insert(100026, 1);  // 1 Small Net
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

#[derive(Deserialize)]
struct LootEntryDef {
    item_id: u32,
    qty_min: Option<u32>,
    qty_max: Option<u32>,
    weight: f32,
}

fn create_loot_tables() -> HashMap<String, LootTable> {
    let content = fs::read_to_string("data/loot_tables.json")
        .expect("Could not read data/loot_tables.json");
    let raw: HashMap<String, Vec<LootEntryDef>> = serde_json::from_str(&content)
        .expect("Failed to parse data/loot_tables.json");

    raw.into_iter()
        .map(|(name, entries)| {
            let items = entries
                .into_iter()
                .map(|e| {
                    let qty = match (e.qty_min, e.qty_max) {
                        (Some(min), Some(max)) => Some((min, max)),
                        _ => None,
                    };
                    (e.item_id, qty, e.weight)
                })
                .collect();
            (name, LootTable { items })
        })
        .collect()
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
