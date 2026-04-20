use crate::actions::{ActionEntry, CombatKill};
use crate::enemy::Enemy;
use crate::inventory::display_and_handle_inventory;
use crate::items::{calculate_loot, get_items, ItemType, LootTable};
use crate::player::Player;
use crate::skill::{combat_xp_calculation, AttackType};
use rand::thread_rng;
use crate::utils::{check_for_input, draw_in_game_box, health_bar, wrap_text, HEALTH_BAR_W};

/// Returned by `handle_combat` to distinguish a kill (with structured data)
/// from any other outcome (ran, defeated, stopped).
pub enum CombatOutcome {
    Kill(CombatKill),
    Other(String),
}

use crossterm::event::{self as xterm_event, Event, KeyCode};
use crossterm::terminal as xterm_terminal;
use log::{debug, info};
use rand::Rng;
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

/// Width of the box interior (space between the ║ characters).
/// Must be HEALTH_BAR_W + 6 so `║  [bar]  ║` fills exactly INNER visible chars.
const INNER: usize = HEALTH_BAR_W + 6;

// ---------------------------------------------------------------------------
// Combat tuning constants
// ---------------------------------------------------------------------------

const BASE_ATTACK_DAMAGE: i32 = 10;
const SPELL_DAMAGE: i32 = 15;
const CHARGED_DAMAGE: i32 = 30;
const RUN_SUCCESS_CHANCE: f64 = 0.5;

// ---------------------------------------------------------------------------
// FAF (auto-combat) constants and types
// ---------------------------------------------------------------------------

/// Milliseconds between auto-combat ticks.
const AUTO_TICK_MS: u64 = 700;
/// Milliseconds to display the victory / defeat screen before auto-dismissing.
const AUTO_DISMISS_MS: u64 = 2000;
/// Auto-eat food when health falls below this percentage of max (integer math).
const AUTO_EAT_THRESHOLD_PCT: i32 = 40;

/// Returned by `handle_combat` in auto mode when the player presses the stop key.
/// `handle_faf_loop` checks for this and exits the FAF loop.
pub const AUTO_COMBAT_STOPPED: &str = "__FAF_STOP__";

/// Which attack type the FAF loop fires automatically each round.
#[derive(Debug, Clone, PartialEq)]
pub enum FafAttackStyle {
    Main,
    Spell,
    Charged,
}

impl FafAttackStyle {
    pub fn display_name(&self) -> &'static str {
        match self {
            FafAttackStyle::Main    => "MAIN",
            FafAttackStyle::Spell   => "SPELL",
            FafAttackStyle::Charged => "CHARGED",
        }
    }
}

// ---------------------------------------------------------------------------
// Auto-combat helper
// ---------------------------------------------------------------------------

/// Try to eat the highest-healing consumable in the player's inventory.
/// Returns a message string on success, `None` if no food is available.
fn try_eat_food(player: &mut Player) -> Option<String> {
    let items = get_items();
    let best = player.inventory.iter()
        .filter_map(|(&item_id, &qty)| {
            if qty == 0 { return None; }
            let item = items.get(&item_id)?;
            if item.item_type != ItemType::Consumable { return None; }
            let heal = item.effect.as_ref()?.health_change;
            if heal <= 0 { return None; }
            Some((item_id, item.name.clone(), heal))
        })
        .max_by_key(|&(_, _, heal)| heal);

    if let Some((item_id, name, heal)) = best {
        player.health = (player.health + heal).min(player.max_health);
        player.remove_item(item_id, 1);
        Some(format!("You eat {}. (+{} HP)", name, heal))
    } else {
        None
    }
}

/// One line with `label` left-aligned and `current / max` right-aligned.
/// Visible width == INNER exactly.
fn stat_line(label: &str, current: i32, max: i32) -> String {
    let hp = format!("{} / {}", current, max);
    let left = format!("  {}", label);
    let right = format!("{}  ", hp);
    let spaces = INNER.saturating_sub(left.len() + right.len());
    format!("{}{}{}", left, " ".repeat(spaces), right)
}

/// Build the complete \r\n-terminated screen string for a combat frame.
/// `auto_label` is shown in the action bar instead of the normal key hints
/// when in FAF mode (e.g. `Some("MAIN")` → "AUTO [MAIN]  |  x = stop").
fn build_combat_frame(
    enemy: &Enemy,
    enemy_max_hp: i32,
    player: &Player,
    msg_lines: &[String],
    charging: bool,
    auto_label: Option<&str>,
) -> String {
    let (tw, _) = term_size::dimensions().unwrap_or((80, 24));
    let pad = " ".repeat(tw.saturating_sub(INNER + 2) / 2);

    let title = "C O M B A T";
    let title_lpad = INNER.saturating_sub(title.len()) / 2;
    let title_line = format!("{}{}", " ".repeat(title_lpad), title);

    // content width after the mandatory 2-space left margin
    let w = INNER - 2;

    macro_rules! row {
        () => {
            format!("{}║{:<width$}║\r\n", pad, "", width = INNER)
        };
        ($content:expr) => {
            format!("{}║  {:<w$}║\r\n", pad, $content, w = w)
        };
    }

    let mut out = String::new();
    out.push_str("\x1B[2J\x1B[1;1H");

    out.push_str(&format!("{}╔{}╗\r\n", pad, "═".repeat(INNER)));
    out.push_str(&format!("{}║{:<width$}║\r\n", pad, title_line, width = INNER));
    out.push_str(&format!("{}╠{}╣\r\n", pad, "═".repeat(INNER)));

    // Enemy
    out.push_str(&row!());
    out.push_str(&format!(
        "{}║{}║\r\n",
        pad,
        stat_line(&enemy.name, enemy.health, enemy_max_hp)
    ));
    // Health bar: ║  [bar]  ║  — visible width = 2 + (HEALTH_BAR_W+2) + 2 = HEALTH_BAR_W+6 = INNER ✓
    out.push_str(&format!(
        "{}║  {}  ║\r\n",
        pad,
        health_bar(enemy.health, enemy_max_hp)
    ));
    out.push_str(&row!());

    // Action log
    for line in msg_lines {
        out.push_str(&row!(line.as_str()));
    }

    // Player
    out.push_str(&row!());
    out.push_str(&format!(
        "{}║{}║\r\n",
        pad,
        stat_line("You", player.health, player.max_health)
    ));
    out.push_str(&format!(
        "{}║  {}  ║\r\n",
        pad,
        health_bar(player.health, player.max_health)
    ));
    out.push_str(&row!());

    // Action bar
    out.push_str(&format!("{}╠{}╣\r\n", pad, "═".repeat(INNER)));
    if let Some(label) = auto_label {
        if charging {
            out.push_str(&row!(format!("AUTO [{}] — firing charged attack…", label)));
        } else {
            out.push_str(&row!(format!("AUTO [{}]  |  x or q = stop", label)));
        }
        out.push_str(&row!());
    } else if charging {
        out.push_str(&row!("Charged attack ready!"));
        out.push_str(&row!("(any key) Fire charged attack"));
    } else {
        out.push_str(&row!("(m) Attack    (c) Charged    (s) Spell"));
        out.push_str(&row!("(i) Items     (r) Run"));
    }
    out.push_str(&format!("{}╚{}╝\r\n", pad, "═".repeat(INNER)));

    out
}

// ---------------------------------------------------------------------------
// Attack helpers — return damage dealt, no side-effect output
// ---------------------------------------------------------------------------

fn main_attack(player: &Player, enemy: &mut Enemy) -> i32 {
    let mut damage = BASE_ATTACK_DAMAGE;
    if let Some(weapon) = &player.equipped_weapon {
        damage += weapon.attack_bonus.unwrap_or(0);
    }
    enemy.take_damage(damage);
    damage
}

fn spell_attack(player: &Player, enemy: &mut Enemy) -> i32 {
    if player.skills.get("Magic").is_some() {
        let damage = SPELL_DAMAGE;
        enemy.take_damage(damage);
        damage
    } else {
        0
    }
}

fn charged_attack(enemy: &mut Enemy, charge_damage: i32) -> i32 {
    enemy.take_damage(charge_damage);
    charge_damage
}

// ---------------------------------------------------------------------------
// Main combat loop
// ---------------------------------------------------------------------------

pub fn handle_combat(
    player: &mut Player,
    mut enemy: Enemy,
    loot_tables: &HashMap<String, LootTable>,
    auto_style: Option<FafAttackStyle>,
) -> CombatOutcome {
    info!("Entering combat with {}", enemy.name);

    let is_auto = auto_style.is_some();
    let auto_label = auto_style.as_ref().map(|s| s.display_name());

    let enemy_max_hp = enemy.health;
    let mut charging = false;
    let mut charge_damage = 0;
    let mut msg_lines: Vec<String> = vec![
        format!("You've encountered a {}!", enemy.name),
        String::new(),
    ];
    let mut attack_counts: HashMap<AttackType, usize> = HashMap::new();
    let mut rng = rand::thread_rng();
    let mut combat_result: Option<CombatOutcome> = None;

    xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");

    loop {
        print!(
            "{}",
            build_combat_frame(&enemy, enemy_max_hp, player, &msg_lines, charging, auto_label)
        );
        io::stdout().flush().unwrap();

        // ---------------------------------------------------------------
        // AUTO path — timed ticks, no blocking key reads
        // ---------------------------------------------------------------
        if is_auto {
            // Poll for the stop key (non-blocking)
            if let Some(key) = check_for_input() {
                if key == "q" || key == "x" {
                    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                    return CombatOutcome::Other(AUTO_COMBAT_STOPPED.to_string());
                }
            }

            thread::sleep(Duration::from_millis(AUTO_TICK_MS));

            // Health management — eat food if below threshold
            if player.health * 100 < player.max_health * AUTO_EAT_THRESHOLD_PCT {
                if let Some(eat_msg) = try_eat_food(player) {
                    msg_lines = vec![eat_msg, String::new()];
                    print!(
                        "{}",
                        build_combat_frame(&enemy, enemy_max_hp, player, &msg_lines, charging, auto_label)
                    );
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(AUTO_TICK_MS / 2));
                }
            }

            if charging {
                // Fire the queued charged attack
                let dmg = charged_attack(&mut enemy, charge_damage);
                charging = false;
                charge_damage = 0;
                *attack_counts.entry(AttackType::Charged).or_insert(0) += 1;

                if enemy.is_defeated() {
                    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                    let xp_gains = combat_xp_calculation(&attack_counts);
                    combat_result = Some(CombatOutcome::Kill(handle_enemy_defeat(player, &enemy, loot_tables, xp_gains, true)));
                    break;
                }
                enemy.attack_player(&mut player.health);
                msg_lines = vec![
                    format!("You unleash a charged attack for {} damage!", dmg),
                    format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                ];
                if player.health <= 0 {
                    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                    combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, true)));
                    break;
                }
            } else {
                match auto_style.as_ref().unwrap() {
                    FafAttackStyle::Main => {
                        let dmg = main_attack(player, &mut enemy);
                        *attack_counts.entry(AttackType::Main).or_insert(0) += 1;
                        debug!("AUTO: hit {} for {} damage", enemy.name, dmg);

                        if enemy.is_defeated() {
                            xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                            let xp_gains = combat_xp_calculation(&attack_counts);
                            combat_result = Some(CombatOutcome::Kill(handle_enemy_defeat(player, &enemy, loot_tables, xp_gains, true)));
                            break;
                        }
                        enemy.attack_player(&mut player.health);
                        msg_lines = vec![
                            format!("You hit the {} for {} damage!", enemy.name, dmg),
                            format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                        ];
                        if player.health <= 0 {
                            xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                            combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, true)));
                            break;
                        }
                    }
                    FafAttackStyle::Spell => {
                        let dmg = spell_attack(player, &mut enemy);
                        *attack_counts.entry(AttackType::Magic).or_insert(0) += 1;
                        debug!("AUTO: spell hit {} for {} damage", enemy.name, dmg);

                        if enemy.is_defeated() {
                            xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                            let xp_gains = combat_xp_calculation(&attack_counts);
                            combat_result = Some(CombatOutcome::Kill(handle_enemy_defeat(player, &enemy, loot_tables, xp_gains, true)));
                            break;
                        }
                        enemy.attack_player(&mut player.health);
                        msg_lines = if dmg > 0 {
                            vec![
                                format!("You cast a spell for {} damage!", dmg),
                                format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                            ]
                        } else {
                            vec![
                                "No magic ability — spell fizzled!".to_string(),
                                format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                            ]
                        };
                        if player.health <= 0 {
                            xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                            combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, true)));
                            break;
                        }
                    }
                    FafAttackStyle::Charged => {
                        // Wind-up round: enemy hits, player charges
                        charging = true;
                        charge_damage = CHARGED_DAMAGE;
                        enemy.attack_player(&mut player.health);
                        msg_lines = vec![
                            "Charging a powerful attack…".to_string(),
                            format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                        ];
                        if player.health <= 0 {
                            xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                            combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, true)));
                            break;
                        }
                    }
                }
            }

            continue; // go back to top of loop (render, then next tick)
        }

        // ---------------------------------------------------------------
        // INTERACTIVE path — block on key press (unchanged)
        // ---------------------------------------------------------------

        // Block until the player presses a key
        let key = loop {
            match xterm_event::read().expect("Failed to read event") {
                Event::Key(ke) => break ke.code,
                _ => continue,
            }
        };

        if charging {
            // Any key fires the charged attack
            let dmg = charged_attack(&mut enemy, charge_damage);
            charging = false;
            charge_damage = 0;
            *attack_counts.entry(AttackType::Charged).or_insert(0) += 1;
            debug!("Player fired charged attack for {} damage", dmg);

            if enemy.is_defeated() {
                xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                let xp_gains = combat_xp_calculation(&attack_counts);
                combat_result = Some(CombatOutcome::Kill(handle_enemy_defeat(player, &enemy, loot_tables, xp_gains, false)));
                break;
            }

            enemy.attack_player(&mut player.health);
            debug!("{} hit player for {} damage", enemy.name, enemy.attack);
            msg_lines = vec![
                format!("You unleash a charged attack for {} damage!", dmg),
                format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
            ];

            if player.health <= 0 {
                xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, false)));
                break;
            }
        } else {
            match key {
                // --- Main attack ---
                KeyCode::Char('m') => {
                    let dmg = main_attack(player, &mut enemy);
                    *attack_counts.entry(AttackType::Main).or_insert(0) += 1;
                    debug!("Player hit {} for {} damage", enemy.name, dmg);

                    if enemy.is_defeated() {
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        let xp_gains = combat_xp_calculation(&attack_counts);
                        combat_result = Some(CombatOutcome::Kill(handle_enemy_defeat(player, &enemy, loot_tables, xp_gains, false)));
                        break;
                    }

                    enemy.attack_player(&mut player.health);
                    debug!("{} hit player for {} damage", enemy.name, enemy.attack);
                    msg_lines = vec![
                        format!("You hit the {} for {} damage!", enemy.name, dmg),
                        format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                    ];

                    if player.health <= 0 {
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, false)));
                        break;
                    }
                }

                // --- Charged attack (wind-up round) ---
                KeyCode::Char('c') => {
                    charging = true;
                    charge_damage = CHARGED_DAMAGE;
                    enemy.attack_player(&mut player.health);
                    debug!("{} hit player for {} damage", enemy.name, enemy.attack);
                    msg_lines = vec![
                        "You begin charging a powerful attack...".to_string(),
                        format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                    ];

                    if player.health <= 0 {
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, false)));
                        break;
                    }
                }

                // --- Spell attack ---
                KeyCode::Char('s') => {
                    let dmg = spell_attack(player, &mut enemy);
                    *attack_counts.entry(AttackType::Magic).or_insert(0) += 1;
                    debug!("Player cast spell on {} for {} damage", enemy.name, dmg);

                    if enemy.is_defeated() {
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        let xp_gains = combat_xp_calculation(&attack_counts);
                        combat_result = Some(CombatOutcome::Kill(handle_enemy_defeat(player, &enemy, loot_tables, xp_gains, false)));
                        break;
                    }

                    enemy.attack_player(&mut player.health);
                    debug!("{} hit player for {} damage", enemy.name, enemy.attack);

                    if dmg > 0 {
                        msg_lines = vec![
                            format!("You cast a spell for {} damage!", dmg),
                            format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                        ];
                    } else {
                        msg_lines = vec![
                            "You lack the magic ability to cast spells!".to_string(),
                            format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                        ];
                    }

                    if player.health <= 0 {
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, false)));
                        break;
                    }
                }

                // --- Open inventory ---
                KeyCode::Char('i') => {
                    xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                    display_and_handle_inventory(player, None, None);
                    xterm_terminal::enable_raw_mode().expect("Failed to enable raw mode");
                    msg_lines = vec![
                        "You checked your inventory.".to_string(),
                        String::new(),
                    ];
                }

                // --- Attempt to run ---
                KeyCode::Char('r') => {
                    if rng.gen_bool(RUN_SUCCESS_CHANCE) {
                        info!("Player successfully ran away from combat.");
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        combat_result = Some(CombatOutcome::Other("Ran away from combat.".to_string()));
                        break;
                    }

                    enemy.attack_player(&mut player.health);
                    debug!("{} hit player for {} damage", enemy.name, enemy.attack);
                    msg_lines = vec![
                        "You failed to escape!".to_string(),
                        format!("The {} hits you for {} damage!", enemy.name, enemy.attack),
                    ];

                    if player.health <= 0 {
                        xterm_terminal::disable_raw_mode().expect("Failed to disable raw mode");
                        combat_result = Some(CombatOutcome::Other(handle_player_defeat(player, &enemy, false)));
                        break;
                    }
                }

                _ => {} // Ignore unrecognised keys
            }
        }
    }

    combat_result.unwrap_or_else(|| CombatOutcome::Other("Combat ended.".to_string()))
}

// ---------------------------------------------------------------------------
// Outcome screens
// ---------------------------------------------------------------------------

fn handle_enemy_defeat(
    player: &mut Player,
    enemy: &Enemy,
    loot_tables: &HashMap<String, LootTable>,
    xp_gains: HashMap<String, f32>,
    auto: bool,
) -> CombatKill {
    info!("Enemy {} has been defeated", enemy.name);

    // Apply skill XP and collect display lines
    let mut xp_rows: Vec<String> = Vec::new();
    let mut sorted_gains: Vec<(&String, &f32)> = xp_gains.iter().collect();
    sorted_gains.sort_by_key(|(name, _)| name.as_str());
    for (skill_name, &xp) in &sorted_gains {
        if xp > 0.0 {
            if let Some(skill) = player.skills.get_mut(*skill_name) {
                let before = skill.level;
                skill.add_experience(xp as f64);
                if skill.level > before {
                    xp_rows.push(format!(
                        "  {} +{:.0} XP  ** LEVEL UP -> {} **",
                        skill_name, xp, skill.level
                    ));
                } else {
                    xp_rows.push(format!("  {} +{:.0} XP", skill_name, xp));
                }
            }
        }
    }

    let combat_xp = 10i32;
    player.add_experience(combat_xp);

    // Collect loot — build both a display string and a structured map
    let mut loot_message = String::new();
    let mut loot_map: HashMap<u32, u32> = HashMap::new();
    let mut looted_ids: Vec<u32> = Vec::new();
    if let Some(table) = loot_tables.get(&enemy.loot_table) {
        let loot = calculate_loot(table);
        player.add_loot(&loot);
        let items = get_items();
        for (item_id, qty) in &loot {
            if *item_id != 0 {
                looted_ids.push(*item_id);
                *loot_map.entry(*item_id).or_insert(0) += qty;
                if let Some(item) = items.get(item_id) {
                    loot_message.push_str(&format!("({}) {}, ", qty, item.name));
                }
            }
        }
        if loot_message.ends_with(", ") {
            loot_message.truncate(loot_message.len() - 2);
        }
    }

    // Quest-conditional drops (e.g. goblin head while quest is active)
    let quest_drop_ids = check_quest_drops(player, &enemy.name);
    for &id in &quest_drop_ids {
        looted_ids.push(id);
        *loot_map.entry(id).or_insert(0) += 1;
        let items = get_items();
        if let Some(item) = items.get(&id) {
            loot_message.push_str(&format!("(1) {}, ", item.name));
        }
    }
    if loot_message.ends_with(", ") {
        loot_message.truncate(loot_message.len() - 2);
    }

    let mut quest_notes = player.on_enemy_killed(&enemy.name);
    quest_notes.extend(player.on_item_gained(&looted_ids));

    // Build result box rows
    let mut rows: Vec<String> = vec![
        String::new(),
        format!("  You defeated the {}!", enemy.name),
        String::new(),
        format!("  Combat XP   +{}", combat_xp),
    ];
    rows.extend(xp_rows);
    rows.push(String::new());
    if loot_message.is_empty() {
        rows.push("  No items dropped.".to_string());
    } else {
        let loot_line = format!("  Looted: {}", loot_message);
        let wrapped = wrap_text(&loot_line, 52);
        rows.extend(wrapped);
    }
    if !quest_notes.is_empty() {
        rows.push(String::new());
        for note in &quest_notes {
            rows.push(format!("  {}", note));
        }
    }
    rows.push(String::new());
    rows.push("  Press Enter to continue.".to_string());
    rows.push(String::new());

    draw_in_game_box("V I C T O R Y", &rows);
    if auto {
        thread::sleep(Duration::from_millis(AUTO_DISMISS_MS));
    } else {
        let _ = io::stdin().read_line(&mut String::new());
    }

    CombatKill {
        enemy_name: enemy.name.clone(),
        kills: 1,
        xp_by_skill: xp_gains,
        combat_xp,
        loot: loot_map,
    }
}

/// Check active quest drops against the killed enemy and return item IDs of
/// anything that should be added to the player's inventory.
fn check_quest_drops(player: &mut Player, enemy_name: &str) -> Vec<u32> {
    let mut rng = thread_rng();
    let mut dropped = Vec::new();

    let active_drops: Vec<_> = player
        .quests
        .iter()
        .filter(|q| !q.completed)
        .flat_map(|q| q.drops.iter())
        .filter(|d| {
            d.enemy_name.eq_ignore_ascii_case(enemy_name) || d.enemy_name == "any"
        })
        .cloned()
        .collect();

    for drop in active_drops {
        let in_inv = player.inventory.get(&drop.item_id).copied().unwrap_or(0);
        if in_inv >= drop.max_in_inventory {
            continue;
        }
        if rng.gen_range(0..drop.chance) == 0 {
            player.add_item_to_inventory(drop.item_id, 1);
            dropped.push(drop.item_id);
        }
    }

    dropped
}

fn handle_player_defeat(player: &mut Player, enemy: &Enemy, auto: bool) -> String {
    info!("Player has been defeated by {}", enemy.name);
    player.in_combat = false;

    let rows = vec![
        String::new(),
        format!("  You were defeated by the {}...", enemy.name),
        String::new(),
        "  Press Enter to continue.".to_string(),
        String::new(),
    ];
    draw_in_game_box("D E F E A T", &rows);
    if auto {
        thread::sleep(Duration::from_millis(AUTO_DISMISS_MS));
    } else {
        let _ = io::stdin().read_line(&mut String::new());
    }

    "You were defeated...".to_string()
}
