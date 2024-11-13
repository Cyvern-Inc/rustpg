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
    pub equipment_slot: Option<u8>, // 1-9 for equipment slots, None for non-equipment
}

impl Item {
    pub fn new(id: u32, name: &str, item_type: ItemType, equipment_slot: Option<u8>) -> Self {
        Item {
            id,
            name: name.to_string(),
            item_type,
            weight: 0.0,
            durability: None,
            effect: None,
            attack_bonus: None,
            defense_bonus: None,
            equipment_slot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    Currency,
    Weapon,
    Armor,
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
    items.insert(100001, Item::new(100001, "Gold Coins", ItemType::Currency, None));

    // Bronze Weapons (100101-100107)
    items.insert(100101, Item::new(100101, "Bronze Dagger", ItemType::Weapon, Some(1)));
    items.insert(100102, Item::new(100102, "Bronze Scimitar", ItemType::Weapon, Some(1)));
    items.insert(100103, Item::new(100103, "Bronze Long Sword", ItemType::Weapon, Some(1)));
    items.insert(100104, Item::new(100104, "Bronze Off-hand Dagger", ItemType::Weapon, Some(2)));
    items.insert(100105, Item::new(100105, "Bronze Mace", ItemType::Weapon, Some(1)));
    items.insert(100106, Item::new(100106, "Bronze Battleaxe", ItemType::Weapon, Some(1)));
    items.insert(100107, Item::new(100107, "Bronze Pike", ItemType::Weapon, Some(1)));

    // Iron Weapons (100111-100117)
    items.insert(100111, Item::new(100111, "Iron Dagger", ItemType::Weapon, Some(1)));
    items.insert(100112, Item::new(100112, "Iron Scimitar", ItemType::Weapon, Some(1)));
    items.insert(100113, Item::new(100113, "Iron Long Sword", ItemType::Weapon, Some(1)));
    items.insert(100114, Item::new(100114, "Iron Off-hand Dagger", ItemType::Weapon, Some(2)));
    items.insert(100115, Item::new(100115, "Iron Mace", ItemType::Weapon, Some(1)));
    items.insert(100116, Item::new(100116, "Iron Battleaxe", ItemType::Weapon, Some(1)));
    items.insert(100117, Item::new(100117, "Iron Pike", ItemType::Weapon, Some(1)));

    // Steel Weapons (100121-100127)
    items.insert(100121, Item::new(100121, "Steel Dagger", ItemType::Weapon, Some(1)));
    items.insert(100122, Item::new(100122, "Steel Scimitar", ItemType::Weapon, Some(1)));
    items.insert(100123, Item::new(100123, "Steel Long Sword", ItemType::Weapon, Some(1)));
    items.insert(100124, Item::new(100124, "Steel Off-hand Dagger", ItemType::Weapon, Some(2)));
    items.insert(100125, Item::new(100125, "Steel Mace", ItemType::Weapon, Some(1)));
    items.insert(100126, Item::new(100126, "Steel Battleaxe", ItemType::Weapon, Some(1)));
    items.insert(100127, Item::new(100127, "Steel Pike", ItemType::Weapon, Some(1)));

    // Mithril Weapons (100131-100137)
    items.insert(100131, Item::new(100131, "Mithril Dagger", ItemType::Weapon, Some(1)));
    items.insert(100132, Item::new(100132, "Mithril Scimitar", ItemType::Weapon, Some(1)));
    items.insert(100133, Item::new(100133, "Mithril Long Sword", ItemType::Weapon, Some(1)));
    items.insert(100134, Item::new(100134, "Mithril Off-hand Dagger", ItemType::Weapon, Some(2)));
    items.insert(100135, Item::new(100135, "Mithril Mace", ItemType::Weapon, Some(1)));
    items.insert(100136, Item::new(100136, "Mithril Battleaxe", ItemType::Weapon, Some(1)));
    items.insert(100137, Item::new(100137, "Mithril Pike", ItemType::Weapon, Some(1)));

    // Legendary Weapons (100141-100147)
    items.insert(100141, Item::new(100141, "Legendary Dagger", ItemType::Weapon, Some(1)));
    items.insert(100142, Item::new(100142, "Legendary Scimitar", ItemType::Weapon, Some(1)));
    items.insert(100143, Item::new(100143, "Legendary Long Sword", ItemType::Weapon, Some(1)));
    items.insert(100144, Item::new(100144, "Legendary Off-hand Dagger", ItemType::Weapon, Some(2)));
    items.insert(100145, Item::new(100145, "Legendary Mace", ItemType::Weapon, Some(1)));
    items.insert(100146, Item::new(100146, "Legendary Battleaxe", ItemType::Weapon, Some(1)));
    items.insert(100147, Item::new(100147, "Legendary Pike", ItemType::Weapon, Some(1)));

    // Bronze Armor Set (100201-100210)
    items.insert(100201, Item::new(100201, "Bronze Full Helmet", ItemType::Armor, Some(3)));
    items.insert(100202, Item::new(100202, "Bronze Lite Helmet", ItemType::Armor, Some(3)));
    items.insert(100203, Item::new(100203, "Bronze Platebody", ItemType::Armor, Some(4)));
    items.insert(100204, Item::new(100204, "Bronze Chainbody", ItemType::Armor, Some(4)));
    items.insert(100205, Item::new(100205, "Bronze Platelegs", ItemType::Armor, Some(5)));
    items.insert(100206, Item::new(100206, "Bronze Chainlegs", ItemType::Armor, Some(5)));
    items.insert(100207, Item::new(100207, "Bronze Heavy Shield", ItemType::Armor, Some(2)));
    items.insert(100208, Item::new(100208, "Bronze Lite Shield", ItemType::Armor, Some(2)));
    items.insert(100209, Item::new(100209, "Bronze Armoured Boots", ItemType::Armor, Some(6)));
    items.insert(100210, Item::new(100210, "Bronze Armoured Gloves", ItemType::Armor, Some(6)));

    // Steel Armor Set (100211-100220)
    items.insert(100211, Item::new(100211, "Steel Full Helmet", ItemType::Armor, Some(3)));
    items.insert(100212, Item::new(100212, "Steel Lite Helmet", ItemType::Armor, Some(3)));
    items.insert(100213, Item::new(100213, "Steel Platebody", ItemType::Armor, Some(4)));
    items.insert(100214, Item::new(100214, "Steel Chainbody", ItemType::Armor, Some(4)));
    items.insert(100215, Item::new(100215, "Steel Platelegs", ItemType::Armor, Some(5)));
    items.insert(100216, Item::new(100216, "Steel Chainlegs", ItemType::Armor, Some(5)));
    items.insert(100217, Item::new(100217, "Steel Heavy Shield", ItemType::Armor, Some(2)));
    items.insert(100218, Item::new(100218, "Steel Lite Shield", ItemType::Armor, Some(2)));
    items.insert(100219, Item::new(100219, "Steel Armoured Boots", ItemType::Armor, Some(6)));
    items.insert(100220, Item::new(100220, "Steel Armoured Gloves", ItemType::Armor, Some(6)));

    // Mithril Armor Set (100221-100230)
    items.insert(100221, Item::new(100221, "Mithril Full Helmet", ItemType::Armor, Some(3)));
    items.insert(100222, Item::new(100222, "Mithril Lite Helmet", ItemType::Armor, Some(3)));
    items.insert(100223, Item::new(100223, "Mithril Platebody", ItemType::Armor, Some(4)));
    items.insert(100224, Item::new(100224, "Mithril Chainbody", ItemType::Armor, Some(4)));
    items.insert(100225, Item::new(100225, "Mithril Platelegs", ItemType::Armor, Some(5)));
    items.insert(100226, Item::new(100226, "Mithril Chainlegs", ItemType::Armor, Some(5)));
    items.insert(100227, Item::new(100227, "Mithril Heavy Shield", ItemType::Armor, Some(2)));
    items.insert(100228, Item::new(100228, "Mithril Lite Shield", ItemType::Armor, Some(2)));
    items.insert(100229, Item::new(100229, "Mithril Armoured Boots", ItemType::Armor, Some(6)));
    items.insert(100230, Item::new(100230, "Mithril Armoured Gloves", ItemType::Armor, Some(6)));

    // Enchanted Armor Set (100231-100240)
    items.insert(100231, Item::new(100231, "Enchanted Full Helmet", ItemType::Armor, Some(3)));
    items.insert(100232, Item::new(100232, "Enchanted Lite Helmet", ItemType::Armor, Some(3)));
    items.insert(100233, Item::new(100233, "Enchanted Platebody", ItemType::Armor, Some(4)));
    items.insert(100234, Item::new(100234, "Enchanted Chainbody", ItemType::Armor, Some(4)));
    items.insert(100235, Item::new(100235, "Enchanted Platelegs", ItemType::Armor, Some(5)));
    items.insert(100236, Item::new(100236, "Enchanted Chainlegs", ItemType::Armor, Some(5)));
    items.insert(100237, Item::new(100237, "Enchanted Heavy Shield", ItemType::Armor, Some(2)));
    items.insert(100238, Item::new(100238, "Enchanted Lite Shield", ItemType::Armor, Some(2)));
    items.insert(100239, Item::new(100239, "Enchanted Armoured Boots", ItemType::Armor, Some(6)));
    items.insert(100240, Item::new(100240, "Enchanted Armoured Gloves", ItemType::Armor, Some(6)));

    // Dragon Scale Armor Set (100241-100250)
    items.insert(100241, Item::new(100241, "Dragon Scale Full Helmet", ItemType::Armor, Some(3)));
    items.insert(100242, Item::new(100242, "Dragon Scale Lite Helmet", ItemType::Armor, Some(3)));
    items.insert(100243, Item::new(100243, "Dragon Scale Platebody", ItemType::Armor, Some(4)));
    items.insert(100244, Item::new(100244, "Dragon Scale Chainbody", ItemType::Armor, Some(4)));
    items.insert(100245, Item::new(100245, "Dragon Scale Platelegs", ItemType::Armor, Some(5)));
    items.insert(100246, Item::new(100246, "Dragon Scale Chainlegs", ItemType::Armor, Some(5)));
    items.insert(100247, Item::new(100247, "Dragon Scale Heavy Shield", ItemType::Armor, Some(2)));
    items.insert(100248, Item::new(100248, "Dragon Scale Lite Shield", ItemType::Armor, Some(2)));
    items.insert(100249, Item::new(100249, "Dragon Scale Armoured Boots", ItemType::Armor, Some(6)));
    items.insert(100250, Item::new(100250, "Dragon Scale Armoured Gloves", ItemType::Armor, Some(6)));

    // Consumables
    items.insert(100301, Item::new(100301, "Raw Shrimp", ItemType::Consumable, None));
    items.insert(100302, Item::new(100302, "Health Potion", ItemType::Consumable, None));
    items.insert(100303, Item::new(100303, "Magic Scroll", ItemType::Consumable, None));

    // Materials/Special Items
    items.insert(100401, Item::new(100401, "Leather Scrap", ItemType::Misc, None));
    items.insert(100402, Item::new(100402, "Rare Gem", ItemType::Misc, None));
    items.insert(100403, Item::new(100403, "Ancient Artifact", ItemType::Misc, None));
    items.insert(100404, Item::new(100404, "Mysterious Crystal", ItemType::Misc, None));

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

    // Common drops (Bronze gear)
    loot_tables.insert(
        "common".to_string(),
        LootTable {
            items: vec![
                (100001, Some((1, 5)), 50.0),   // Gold Coins
                (100101, Some((1, 1)), 10.0),   // Bronze Dagger
                (100202, Some((1, 1)), 8.0),    // Bronze Lite Helmet
                (100204, Some((1, 1)), 8.0),    // Bronze Chainbody
                (100206, Some((1, 1)), 8.0),    // Bronze Chainlegs
                (100208, Some((1, 1)), 8.0),    // Bronze Lite Shield
                (100209, Some((1, 1)), 8.0),    // Bronze Boots
                (0, None, 5.0),                 // Nothing
            ],
        },
    );

    // Rare drops (Steel gear)
    loot_tables.insert(
        "rare".to_string(),
        LootTable {
            items: vec![
                (100001, Some((5, 15)), 40.0),  // Gold Coins
                (100102, Some((1, 1)), 15.0),   // Iron Sword
                (100211, Some((1, 1)), 10.0),   // Steel Full Helmet
                (100213, Some((1, 1)), 10.0),   // Steel Platebody
                (100215, Some((1, 1)), 10.0),   // Steel Platelegs
                (100217, Some((1, 1)), 10.0),   // Steel Heavy Shield
                (100219, Some((1, 1)), 5.0),    // Steel Boots
                (0, None, 2.0),                 // Nothing
            ],
        },
    );

    // Very rare drops (Mithril/Enchanted gear)
    loot_tables.insert(
        "very_rare".to_string(),
        LootTable {
            items: vec![
                (100001, Some((15, 30)), 35.0), // Gold Coins
                (100104, Some((1, 1)), 20.0),   // Mithril Sword
                (100221, Some((1, 1)), 15.0),   // Mithril Full Helmet
                (100223, Some((1, 1)), 15.0),   // Mithril Platebody
                (100231, Some((1, 1)), 10.0),   // Enchanted Full Helmet
                (100233, Some((1, 1)), 10.0),   // Enchanted Platebody
                (100237, Some((1, 1)), 5.0),    // Enchanted Heavy Shield
            ],
        },
    );

    // Boss drops (Dragon Scale gear)
    loot_tables.insert(
        "boss".to_string(),
        LootTable {
            items: vec![
                (100001, Some((50, 100)), 100.0), // Gold Coins
                (100105, Some((1, 1)), 40.0),     // Legendary Sword
                (100241, Some((1, 1)), 30.0),     // Dragon Scale Full Helmet
                (100243, Some((1, 1)), 30.0),     // Dragon Scale Platebody
                (100245, Some((1, 1)), 30.0),     // Dragon Scale Platelegs
                (100247, Some((1, 1)), 20.0),     // Dragon Scale Heavy Shield
                (100249, Some((1, 1)), 20.0),     // Dragon Scale Boots
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,      // slot 3: Full/Lite Helmets
    Chest,     // slot 4: Plate/Chain bodies
    Legs,      // slot 5: Plate/Chain legs
    Feet,      // slot 6: Boots
    Hands,     // slot 7: Gloves
    MainHand,  // slot 1: Main weapons
    OffHand,   // slot 2: Shields, off-hand weapons
}

impl EquipmentSlot {
    pub fn from_slot_number(slot: u8) -> Option<Self> {
        match slot {
            1 => Some(Self::MainHand),
            2 => Some(Self::OffHand),
            3 => Some(Self::Head),
            4 => Some(Self::Chest),
            5 => Some(Self::Legs),
            6 => Some(Self::Feet),
            7 => Some(Self::Hands),
            _ => None
        }
    }
}
