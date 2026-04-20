use rand::Rng;
use crate::map::{Map, Tile};
use crate::player::Player;

// ---------------------------------------------------------------------------
// Cookable item definitions
// ---------------------------------------------------------------------------

pub struct CookableItem {
    pub raw_id: u32,
    pub cooked_id: u32,
    pub burnt_id: u32,
    pub raw_name: &'static str,
    pub required_level: i32,
    pub xp: f64,
}

pub static COOKABLES: &[CookableItem] = &[
    CookableItem {
        raw_id: 100015,
        cooked_id: 100016,
        burnt_id: 100024,
        raw_name: "Raw Shrimp",
        required_level: 1,
        xp: 30.0,
    },
    CookableItem {
        raw_id: 100017,
        cooked_id: 100018,
        burnt_id: 100025,
        raw_name: "Raw Beef",
        required_level: 7,
        xp: 70.0,
    },
    CookableItem {
        raw_id: 100027,
        cooked_id: 100028,
        burnt_id: 100029,
        raw_name: "Raw Anchovies",
        required_level: 10,
        xp: 50.0,
    },
];

/// Find the cooking recipe for a given raw item ID.
pub fn find_cookable(item_id: u32) -> Option<&'static CookableItem> {
    COOKABLES.iter().find(|c| c.raw_id == item_id)
}

/// Returns true if any of the four cardinal tiles adjacent to the player
/// contains a campfire.
pub fn is_adjacent_to_campfire(map: &Map) -> bool {
    let (px, py) = (map.player_x, map.player_y);
    let candidates = [
        (px, py.wrapping_sub(1)),
        (px, py + 1),
        (px.wrapping_sub(1), py),
        (px + 1, py),
    ];
    candidates.iter().any(|&(x, y)| {
        x < map.width && y < map.height && map.tiles[y][x] == Tile::Campfire
    })
}

/// Cook `count` of the raw item described by `cookable`.
/// Removes raw items, adds cooked or burnt versions, and awards XP.
/// Returns a feedback string describing the outcome.
pub fn do_cook(player: &mut Player, cookable: &CookableItem, count: u32) -> String {
    let cooking_level = player
        .skills
        .get("Cooking")
        .map(|s| s.level)
        .unwrap_or(1);

    if cooking_level < cookable.required_level {
        return format!(
            "You need Cooking level {} to cook that. (Your level: {})",
            cookable.required_level, cooking_level
        );
    }

    // 50% success at required level, 100% at required+10, linear between.
    let level_delta = (cooking_level - cookable.required_level).max(0) as f64;
    let success_rate = (0.5 + level_delta / 10.0 * 0.5).min(1.0);

    let mut rng = rand::thread_rng();
    let mut cooked = 0u32;
    let mut burnt = 0u32;
    let mut total_xp = 0.0f64;

    for _ in 0..count {
        // Defensive check — remove one raw item
        match player.inventory.get_mut(&cookable.raw_id) {
            Some(qty) if *qty > 0 => {
                *qty -= 1;
                if *qty == 0 {
                    player.inventory.remove(&cookable.raw_id);
                }
            }
            _ => break, // ran out unexpectedly
        }

        if rng.gen::<f64>() < success_rate {
            *player.inventory.entry(cookable.cooked_id).or_insert(0) += 1;
            total_xp += cookable.xp;
            cooked += 1;
        } else {
            *player.inventory.entry(cookable.burnt_id).or_insert(0) += 1;
            burnt += 1;
        }
    }

    // Award XP only for successful cooks
    if total_xp > 0.0 {
        if let Some(skill) = player.skills.get_mut("Cooking") {
            skill.add_experience(total_xp);
        }
    }

    match (cooked, burnt) {
        (c, 0) => format!("Successfully cooked {}! (+{:.0} Cooking XP)", c, total_xp),
        (0, b) => format!("You burnt all {}. No XP gained.", b),
        (c, b) => format!("Cooked {}, burnt {}. (+{:.0} Cooking XP)", c, b, total_xp),
    }
}
