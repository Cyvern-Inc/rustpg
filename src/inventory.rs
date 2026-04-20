use crate::cooking::{do_cook, find_cookable, is_adjacent_to_campfire};
use crate::items::{get_items, ItemType};
use crate::map::{Direction, Map, Tile};
use crate::player::Player;
use crate::utils::draw_in_game_box;
use std::io::{self, Write};

// Item IDs for crafting interactions
const FLINT_N_STEEL: u32 = 100020;
const LOG: u32 = 100022;

/// Resolve a command token (1-based index string or item name) against the
/// numbered list built each render pass. Returns a reference to the matching
/// entry `(item_id, name, quantity)`, or `None`.
fn resolve_item<'a>(token: &str, numbered: &'a [(u32, String, u32)]) -> Option<&'a (u32, String, u32)> {
    let token = token.trim();
    if let Ok(n) = token.parse::<usize>() {
        return if n >= 1 { numbered.get(n - 1) } else { None };
    }
    numbered.iter().find(|(_, name, _)| name.eq_ignore_ascii_case(token))
}

/// Check adjacency, prompt for quantity if needed, then delegate to `do_cook`.
/// `map` may be `None` when called from a context without map access (e.g. combat).
fn attempt_cook(
    player: &mut Player,
    map: Option<&Map>,
    item_id: u32,
    item_name: &str,
    qty_available: u32,
) -> String {
    let cookable = match find_cookable(item_id) {
        Some(c) => c,
        None => return format!("You can't cook {}.", item_name),
    };

    match map {
        Some(m) if is_adjacent_to_campfire(m) => {}
        Some(_) => return "You need to be standing next to a campfire to cook.".to_string(),
        None    => return "You can't cook here.".to_string(),
    }

    let count = if qty_available > 1 {
        println!();
        println!(
            "  How many {} would you like to cook? (1-{}, Enter = all)",
            item_name, qty_available
        );
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).expect("Failed to read");
        let trimmed = buf.trim();
        if trimmed.is_empty() {
            qty_available
        } else {
            trimmed.parse::<u32>().unwrap_or(qty_available).clamp(1, qty_available)
        }
    } else {
        1
    };

    do_cook(player, cookable, count)
}

/// Attempt to place a campfire on the tile the player is facing.
/// Consumes one Log; Flint 'n Steel is reusable.
fn use_flint_with_log(player: &mut Player, map: &mut Map) -> String {
    if player.inventory.get(&LOG).copied().unwrap_or(0) == 0 {
        return "You don't have any logs to burn.".to_string();
    }

    let (tx, ty) = match player.facing {
        Direction::Up    => (map.player_x, map.player_y.wrapping_sub(1)),
        Direction::Down  => (map.player_x, map.player_y + 1),
        Direction::Left  => (map.player_x.wrapping_sub(1), map.player_y),
        Direction::Right => (map.player_x + 1, map.player_y),
    };

    if tx >= map.width || ty >= map.height {
        return "There's no room to place a campfire there.".to_string();
    }
    if map.tiles[ty][tx] != Tile::Empty {
        return "You can't place a campfire on that tile.".to_string();
    }

    // Consume one log
    let qty = player.inventory.get_mut(&LOG).unwrap();
    *qty -= 1;
    if *qty == 0 {
        player.inventory.remove(&LOG);
    }

    map.set_tile(tx, ty, Tile::Campfire);
    map.campfire_x = tx;
    map.campfire_y = ty;

    "You strike the flint 'n steel and light a campfire!".to_string()
}

pub fn display_inventory(
    player: &mut Player,
    filter_type: Option<ItemType>,
    mut map: Option<&mut Map>,
) -> Option<String> {
    let mut feedback: Option<String> = None;

    loop {
        let items = get_items();

        // Collect matching items sorted alphabetically; index = 1-based slot number
        let mut numbered: Vec<(u32, String, u32)> = player
            .inventory
            .iter()
            .filter_map(|(&item_id, &quantity)| {
                let item = items.get(&item_id)?;
                if filter_type.as_ref().map_or(true, |f| item.item_type == *f) && quantity > 0 {
                    Some((item_id, item.name.clone(), quantity))
                } else {
                    None
                }
            })
            .collect();
        numbered.sort_by(|a, b| a.1.cmp(&b.1));

        // Build box rows
        let mut rows: Vec<String> = Vec::new();

        // Equipped section
        rows.push(String::new());
        rows.push("  Equipped".to_string());
        rows.push("  --------".to_string());
        match &player.equipped_weapon {
            Some(w) => {
                let bonus = w.attack_bonus.map(|b| format!(" (+{} atk)", b)).unwrap_or_default();
                rows.push(format!("  Weapon  {}{}", w.name, bonus));
            }
            None => rows.push("  Weapon  (none)".to_string()),
        }
        for (label, slot_key) in &[
            ("Head  ", "head"),
            ("Body  ", "body"),
            ("Legs  ", "legs"),
            ("Shield", "shield"),
            ("Boots ", "boots"),
            ("Hands ", "hands"),
        ] {
            match player.armor_slots.get(*slot_key) {
                Some(a) => {
                    let bonus = a.defense_bonus.map(|b| format!(" (+{} def)", b)).unwrap_or_default();
                    rows.push(format!("  {}  {}{}", label, a.name, bonus));
                }
                None => rows.push(format!("  {}  (none)", label)),
            }
        }
        rows.push(String::new());

        // Inventory list
        if numbered.is_empty() {
            rows.push("  No items found.".to_string());
        } else {
            for (i, (_, name, qty)) in numbered.iter().enumerate() {
                rows.push(format!("  [{:>2}] {:<22} x{}", i + 1, name, qty));
            }
        }
        rows.push(String::new());
        if let Some(msg) = &feedback {
            for line in msg.lines() {
                rows.push(format!("  {}", line));
            }
            rows.push(String::new());
        }
        rows.push("  eat <item|#>   cook <item|#>   use <item|#> with <item|#>".to_string());
        rows.push("  equip <item|#>   unequip <weapon|head|body|legs|shield|boots|hands>".to_string());
        rows.push("  q = close".to_string());
        rows.push(String::new());

        let pad = draw_in_game_box("I N V E N T O R Y", &rows);
        print!("\n{}> ", pad);
        io::stdout().flush().unwrap();

        feedback = None;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "q" => return None,

            cmd if cmd.starts_with("eat ") => {
                let token = cmd.trim_start_matches("eat ").trim();
                match resolve_item(token, &numbered) {
                    Some(entry) => {
                        let (item_id, name, qty) = (entry.0, entry.1.clone(), entry.2);
                        if qty > 0 {
                            match items.get(&item_id).map(|i| &i.item_type) {
                                Some(ItemType::Consumable) => {
                                    feedback = Some(handle_eat_command(player, &name));
                                }
                                _ => feedback = Some("You can't eat that.".to_string()),
                            }
                        } else {
                            feedback = Some(format!("You don't have any '{}' to eat.", name));
                        }
                    }
                    None => feedback = Some(format!("You don't have any '{}' to eat.", token)),
                }
                continue;
            }

            cmd if cmd.starts_with("use ") && cmd.contains(" with ") => {
                let rest = cmd.trim_start_matches("use ");
                let with_pos = rest.find(" with ").unwrap();
                let left_token  = rest[..with_pos].trim();
                let right_token = rest[with_pos + 6..].trim();

                // "use X with fire/campfire" → cook X on the fire
                let right_is_fire = right_token == "fire" || right_token == "campfire";
                if right_is_fire {
                    match resolve_item(left_token, &numbered).map(|e| (e.0, e.1.clone(), e.2)) {
                        Some((item_id, name, qty)) => {
                            feedback = Some(attempt_cook(
                                player, map.as_deref(), item_id, &name, qty,
                            ));
                        }
                        None => feedback = Some(format!("You don't have '{}'.", left_token)),
                    }
                    continue;
                }

                // Resolve both sides to owned data before any mutable borrows
                let left  = resolve_item(left_token,  &numbered).map(|e| (e.0, e.1.clone()));
                let right = resolve_item(right_token, &numbered).map(|e| (e.0, e.1.clone()));

                match (left, right) {
                    (Some((lid, _)), Some((rid, _)))
                        if (lid == FLINT_N_STEEL && rid == LOG)
                            || (lid == LOG && rid == FLINT_N_STEEL) =>
                    {
                        feedback = Some(match map.as_deref_mut() {
                            Some(m) => use_flint_with_log(player, m),
                            None    => "You can't light a campfire here.".to_string(),
                        });
                    }
                    (None, _) => {
                        feedback = Some(format!("You don't have '{}'.", left_token));
                    }
                    (_, None) => {
                        feedback = Some(format!("You don't have '{}'.", right_token));
                    }
                    _ => {
                        feedback = Some("Nothing happens.".to_string());
                    }
                }
                continue;
            }

            "cook" => {
                feedback = Some("Cook what? Usage: cook <item|#>".to_string());
                continue;
            }

            cmd if cmd.starts_with("cook ") => {
                let token = cmd.trim_start_matches("cook ").trim();
                match resolve_item(token, &numbered).map(|e| (e.0, e.1.clone(), e.2)) {
                    Some((item_id, name, qty)) => {
                        feedback = Some(attempt_cook(
                            player, map.as_deref(), item_id, &name, qty,
                        ));
                    }
                    None => feedback = Some(format!("You don't have any '{}' to cook.", token)),
                }
                continue;
            }

            cmd if cmd.starts_with("use ") => {
                let token = cmd.trim_start_matches("use ").trim();
                match resolve_item(token, &numbered) {
                    Some(entry) => {
                        let (item_id, name, qty) = (entry.0, entry.1.clone(), entry.2);
                        if qty > 0 {
                            match items.get(&item_id).map(|i| &i.item_type) {
                                Some(ItemType::Consumable) => {
                                    feedback = Some(handle_eat_command(player, &name));
                                }
                                _ => feedback = Some(
                                    "Try: use <item> with <item>".to_string(),
                                ),
                            }
                        } else {
                            feedback = Some(format!("You don't have any '{}' to use.", name));
                        }
                    }
                    None => feedback = Some(format!("You don't have any '{}' to use.", token)),
                }
                continue;
            }

            cmd if cmd.starts_with("equip ") => {
                let token = cmd.trim_start_matches("equip ").trim();
                match resolve_item(token, &numbered).map(|e| (e.0, e.1.clone())) {
                    Some((item_id, name)) => {
                        let item = items.get(&item_id).cloned();
                        match item {
                            Some(ref it) if it.equip_slot.is_some() => {
                                let slot = it.equip_slot.clone().unwrap();
                                let required = it.equip_level.unwrap_or(1);
                                let attack_level = player
                                    .skills
                                    .get("Attack")
                                    .map(|s| s.level)
                                    .unwrap_or(1);
                                if attack_level < required {
                                    feedback = Some(format!(
                                        "You need Attack level {} to equip {}. (Your level: {})",
                                        required, name, attack_level
                                    ));
                                } else {
                                    // Remove from inventory first
                                    if let Some(qty) = player.inventory.get_mut(&item_id) {
                                        *qty -= 1;
                                        if *qty == 0 {
                                            player.inventory.remove(&item_id);
                                        }
                                    }
                                    if slot == "weapon" {
                                        // Swap old weapon back to inventory
                                        if let Some(old) = player.equipped_weapon.take() {
                                            *player.inventory.entry(old.id).or_insert(0) += 1;
                                        }
                                        let bonus = it.attack_bonus
                                            .map(|b| format!(" (+{} atk)", b))
                                            .unwrap_or_default();
                                        player.equipped_weapon = item;
                                        feedback = Some(format!("Equipped {}{}.", name, bonus));
                                    } else {
                                        // Swap old armor in this slot back to inventory
                                        if let Some(old) = player.armor_slots.remove(&slot) {
                                            *player.inventory.entry(old.id).or_insert(0) += 1;
                                        }
                                        let bonus = it.defense_bonus
                                            .map(|b| format!(" (+{} def)", b))
                                            .unwrap_or_default();
                                        player.armor_slots.insert(slot, item.unwrap());
                                        feedback = Some(format!("Equipped {}{}.", name, bonus));
                                    }
                                }
                            }
                            Some(_) => {
                                feedback = Some(format!("{} cannot be equipped.", name));
                            }
                            None => {
                                feedback = Some(format!("Unknown item: {}.", token));
                            }
                        }
                    }
                    None => feedback = Some(format!("You don't have '{}'.", token)),
                }
                continue;
            }

            cmd if cmd.starts_with("unequip") => {
                let slot_token = cmd.trim_start_matches("unequip").trim();
                // Normalise common aliases
                let slot = match slot_token {
                    "weapon" | ""        => "weapon",
                    "head"   | "helm"    => "head",
                    "body"   | "chest"   => "body",
                    "legs"               => "legs",
                    "shield"             => "shield",
                    "boots"  | "feet"    => "boots",
                    "hands"  | "gloves"
                    | "gauntlets"        => "hands",
                    other => {
                        feedback = Some(format!(
                            "Unknown slot '{}'. Try: weapon, head, body, legs, shield, boots, hands.",
                            other
                        ));
                        continue;
                    }
                };
                if slot == "weapon" {
                    match player.equipped_weapon.take() {
                        Some(w) => {
                            let name = w.name.clone();
                            *player.inventory.entry(w.id).or_insert(0) += 1;
                            feedback = Some(format!("Unequipped {}.", name));
                        }
                        None => feedback = Some("No weapon equipped.".to_string()),
                    }
                } else {
                    match player.armor_slots.remove(slot) {
                        Some(a) => {
                            let name = a.name.clone();
                            *player.inventory.entry(a.id).or_insert(0) += 1;
                            feedback = Some(format!("Unequipped {}.", name));
                        }
                        None => feedback = Some(format!("Nothing equipped in {} slot.", slot)),
                    }
                }
                continue;
            }

            _ => {
                feedback = Some("Invalid command.".to_string());
                continue;
            }
        }
    }
}

pub fn consume_item(player: &mut Player, item_name: &str) -> Option<String> {
    let items = get_items();

    if let Some(item) = items
        .values()
        .find(|i| i.name.eq_ignore_ascii_case(item_name))
    {
        if item.item_type != ItemType::Consumable {
            return None;
        }

        if let Some(quantity) = player.inventory.get_mut(&item.id) {
            if *quantity > 0 {
                *quantity -= 1;
                if *quantity == 0 {
                    player.inventory.remove(&item.id);
                }

                let mut message = format!("You ate the {}!", item.name);
                if let Some(effect) = &item.effect {
                    if effect.health_change != 0 {
                        player.health = (player.health + effect.health_change)
                            .min(player.max_health)
                            .max(0);
                        message.push_str(&format!(
                            "\nHealth restored: {}. Current health: {}/{}",
                            effect.health_change, player.health, player.max_health
                        ));
                    }
                    if effect.stamina_change != 0 {
                        message.push_str(&format!("\nStamina change: {}", effect.stamina_change));
                    }
                }
                return Some(message);
            }
        }
    }
    None
}

pub fn handle_eat_command(player: &mut Player, item_name: &str) -> String {
    if let Some(result) = consume_item(player, item_name) {
        result
    } else {
        "You can't eat that!".to_string()
    }
}

pub fn display_and_handle_inventory(
    player: &mut Player,
    item_type_filter: Option<ItemType>,
    map: Option<&mut Map>,
) -> String {
    display_inventory(player, item_type_filter, map);
    "Viewed inventory.".to_string()
}
