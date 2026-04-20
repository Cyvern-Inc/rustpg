use std::collections::HashMap;
use crate::items::get_items;

#[derive(Clone)]
pub struct CombatKill {
    pub enemy_name: String,
    pub kills: u32,
    pub xp_by_skill: HashMap<String, f32>,
    pub combat_xp: i32,
    pub loot: HashMap<u32, u32>, // item_id → qty
}

#[derive(Clone)]
pub enum ActionEntry {
    Kill(CombatKill),
    Generic(String, u32), // text, consecutive-count
}

impl ActionEntry {
    /// Format this entry as display lines for the sidebar.
    /// Lines are NOT pre-wrapped — callers should apply wrap_text per line.
    pub fn format_for_sidebar(&self) -> Vec<String> {
        match self {
            ActionEntry::Generic(text, count) => {
                if *count > 1 {
                    vec![format!("{} (x{})", text, count)]
                } else {
                    vec![text.clone()]
                }
            }
            ActionEntry::Kill(kill) => {
                let mut lines = Vec::new();

                // Header: name + kill count
                if kill.kills > 1 {
                    lines.push(format!("Defeated a {} ({})", kill.enemy_name, kill.kills));
                } else {
                    lines.push(format!("Defeated a {}", kill.enemy_name));
                }

                // Per-skill XP, sorted by skill name
                let mut xp_entries: Vec<(&String, &f32)> = kill
                    .xp_by_skill
                    .iter()
                    .filter(|(_, xp)| **xp > 0.0)
                    .collect();
                xp_entries.sort_by_key(|(name, _)| name.as_str());
                let xp_parts: Vec<String> = xp_entries
                    .iter()
                    .map(|(name, xp)| format!("{} +{:.0}", name, xp))
                    .collect();
                if !xp_parts.is_empty() {
                    lines.push(format!("  {}", xp_parts.join("  ")));
                }

                // Loot list, sorted by item name
                if kill.loot.is_empty() {
                    lines.push("  No loot".to_string());
                } else {
                    let items = get_items();
                    let mut loot_entries: Vec<(String, u32)> = kill
                        .loot
                        .iter()
                        .filter_map(|(&id, &qty)| {
                            items.get(&id).map(|item| (item.name.clone(), qty))
                        })
                        .collect();
                    loot_entries.sort_by_key(|(name, _)| name.clone());
                    let loot_str = loot_entries
                        .iter()
                        .map(|(name, qty)| format!("{} x{}", name, qty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("  {}", loot_str));
                }

                lines
            }
        }
    }
}
