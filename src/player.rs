use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::skill::{Skill, initialize_skills};
use crate::items::get_starting_items;
use crate::quest::{Quest, QuestReward, ObjectiveKind, all_quests, quest_by_id};
use crate::items::Item;
use crate::items::get_items;
use crate::map::{Map, Direction};
use crate::utils::{draw_in_game_box, health_bar};
use std::io::{self, Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub health: i32,
    pub max_health: i32,
    pub attack: i32,
    pub level: i32,
    pub experience: i32,
    pub quests: Vec<Quest>,
    pub inventory: HashMap<u32, u32>,
    pub equipped_weapon: Option<Item>,
    /// Armor worn in each body slot. Keys: "head", "body", "legs", "shield", "boots", "hands".
    #[serde(default)]
    pub armor_slots: HashMap<String, Item>,
    pub skills: HashMap<String, Skill>,
    pub active_quest: Option<Quest>,
    pub in_combat: bool,
    pub facing: Direction,
    #[serde(default)]
    pub quest_points: u32,
}

impl Player {
    pub fn new() -> Self {
        let mut player = Player {
            health: 100,
            max_health: 100,
            attack: 10,
            level: 1,
            experience: 0,
            quests: vec![],
            inventory: HashMap::new(),
            equipped_weapon: None,
            armor_slots: HashMap::new(),
            skills: initialize_skills(),
            active_quest: None,
            in_combat: false,
            facing: Direction::Down,
            quest_points: 0,
        };
        player.add_starting_items();
        player
    }

    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.push(quest);
    }

    pub fn add_item_to_inventory(&mut self, item_id: u32, quantity: u32) {
        *self.inventory.entry(item_id).or_insert(0) += quantity;
    }

    pub fn add_starting_items(&mut self) {
        for (item_id, quantity) in get_starting_items() {
            self.add_item_to_inventory(item_id, quantity);
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health <= 0 {
            println!("Player has been defeated!");
            self.health = 0; // Ensure health does not go negative
        }
    }

    pub fn add_experience(&mut self, amount: i32) {
        self.experience += amount;
        if self.experience >= 100 {
            self.level_up();
        }
    }

    pub fn level_up(&mut self) {
        self.level += 1;
        self.health = self.max_health; // Restore health to max on level up
        println!("Player leveled up to level {}!", self.level);
    }

    pub fn display_status(&self) {
        let mut rows: Vec<String> = Vec::new();

        // --- Stats ---
        rows.push(String::new());
        let hp_label = format!("  Health  {}/{}", self.health, self.max_health);
        rows.push(hp_label);
        rows.push(format!("  {}", health_bar(self.health, self.max_health)));
        rows.push(String::new());
        rows.push(format!("  Level       {}", self.level));
        rows.push(format!("  Experience  {}", self.experience));
        rows.push(format!("  Quest Points {}", self.quest_points));

        // --- Skills ---
        rows.push(String::new());
        rows.push("  Skills".to_string());
        rows.push("  ------".to_string());
        let mut skill_names: Vec<&String> = self.skills.keys().collect();
        skill_names.sort();
        for name in skill_names {
            let skill = &self.skills[name];
            rows.push(format!(
                "  {:<16} Lv {:>2}   XP {:.0}",
                name, skill.level, skill.experience
            ));
        }

        // --- Equipped ---
        rows.push(String::new());
        rows.push("  Equipped".to_string());
        rows.push("  --------".to_string());
        match &self.equipped_weapon {
            Some(w) => {
                let acc = w.melee_accuracy.map(|b| format!(" (+{} acc)", b)).unwrap_or_default();
                let str_bonus = w.melee_strength.map(|b| format!(" (+{} str)", b)).unwrap_or_default();
                rows.push(format!("  Weapon  {}{}{}", w.name, acc, str_bonus));
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
            match self.armor_slots.get(*slot_key) {
                Some(a) => {
                    let bonus = a.melee_defense.map(|b| format!(" (+{} def)", b)).unwrap_or_default();
                    rows.push(format!("  {}  {}{}", label, a.name, bonus));
                }
                None => rows.push(format!("  {}  (none)", label)),
            }
        }

        rows.push(String::new());
        rows.push("  Press Enter to continue.".to_string());
        rows.push(String::new());

        draw_in_game_box("P L A Y E R   S T A T U S", &rows);
        let _ = io::stdin().read_line(&mut String::new());
    }

    // Train a skill by adding experience to it
    pub fn train_skill(&mut self, skill_name: &str, xp_gain: f32) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.add_experience(xp_gain as f64);
        } else {
            println!("Skill not found: {}", skill_name);
        }
    }

    // Add loot to player's inventory
    pub fn add_loot(&mut self, loot: &HashMap<u32, u32>) {
        for (&item_id, &quantity) in loot {
            *self.inventory.entry(item_id).or_insert(0) += quantity;
        }
    }

    pub fn remove_item(&mut self, item_id: u32, amount: u32) -> bool {
        if let Some(quantity) = self.inventory.get_mut(&item_id) {
            if *quantity >= amount {
                *quantity -= amount;
                if *quantity == 0 {
                    self.inventory.remove(&item_id);
                }
                return true;
            }
        }
        false
    }

    pub fn total_level(&self) -> i32 {
        self.skills.values().map(|skill| skill.level.min(99)).sum()
    }

    /// Called after killing an enemy. Updates kill objectives, completes quests,
    /// awards rewards, and unlocks follow-up quests. Returns notification lines
    /// to display to the player.
    pub fn on_enemy_killed(&mut self, enemy_name: &str) -> Vec<String> {
        let mut notifications: Vec<String> = Vec::new();
        let mut completed_ids: Vec<u32> = Vec::new();
        let mut pending_rewards: Vec<Vec<QuestReward>> = Vec::new();

        for quest in &mut self.quests {
            if quest.completed {
                continue;
            }
            let mut progressed = false;
            for obj in &mut quest.objectives {
                if let ObjectiveKind::KillEnemy { enemy_name: ref target } = obj.kind {
                    let matches = target == "any" || target.eq_ignore_ascii_case(enemy_name);
                    if matches && !obj.is_complete() {
                        obj.current += 1;
                        progressed = true;
                    }
                }
            }
            if progressed && quest.objectives_met() && quest.auto_complete {
                quest.completed = true;
                completed_ids.push(quest.id);
                notifications.push(format!("*** Quest complete: {} ***", quest.name));
                pending_rewards.push(quest.rewards.clone());
            }
        }

        // Apply rewards after the mutable borrow of self.quests ends
        for rewards in pending_rewards {
            notifications.extend(self.collect_rewards(&rewards));
        }
        self.unlock_follow_up_quests(&completed_ids, &mut notifications);
        notifications
    }

    /// Called after loot is added to inventory. Updates HaveItem objectives,
    /// completes quests, awards rewards, and unlocks follow-up quests.
    pub fn on_item_gained(&mut self, gained_item_ids: &[u32]) -> Vec<String> {
        let mut notifications: Vec<String> = Vec::new();
        let mut completed_ids: Vec<u32> = Vec::new();
        let mut pending_rewards: Vec<Vec<QuestReward>> = Vec::new();

        for quest in &mut self.quests {
            if quest.completed {
                continue;
            }
            let mut progressed = false;
            for obj in &mut quest.objectives {
                if let ObjectiveKind::HaveItem { item_id } = obj.kind {
                    if gained_item_ids.contains(&item_id) && !obj.is_complete() {
                        let qty = self.inventory.get(&item_id).copied().unwrap_or(0);
                        obj.current = qty.min(obj.required);
                        if obj.is_complete() {
                            progressed = true;
                        }
                    }
                }
            }
            if progressed && quest.objectives_met() && quest.auto_complete {
                quest.completed = true;
                completed_ids.push(quest.id);
                notifications.push(format!("*** Quest complete: {} ***", quest.name));
                pending_rewards.push(quest.rewards.clone());
            }
        }

        for rewards in pending_rewards {
            notifications.extend(self.collect_rewards(&rewards));
        }
        self.unlock_follow_up_quests(&completed_ids, &mut notifications);
        notifications
    }

    /// Apply quest rewards and return display lines for each reward granted.
    fn collect_rewards(&mut self, rewards: &[QuestReward]) -> Vec<String> {
        let items = get_items();
        let mut lines = Vec::new();
        for reward in rewards {
            match reward {
                QuestReward::Experience(xp) => {
                    self.add_experience(*xp);
                    lines.push(format!("  +{} XP", xp));
                }
                QuestReward::Item(item_id, qty) => {
                    self.add_item_to_inventory(*item_id, *qty);
                    let name = items
                        .get(item_id)
                        .map(|i| i.name.as_str())
                        .unwrap_or("Unknown item");
                    lines.push(format!("  +{} {}", qty, name));
                }
                QuestReward::QuestPoints(pts) => {
                    self.quest_points += pts;
                    lines.push(format!("  +{} Quest Point{}", pts, if *pts == 1 { "" } else { "s" }));
                }
            }
        }
        lines
    }

    /// Complete a quest via dialogue turn-in. Marks it complete, grants rewards,
    /// and unlocks follow-up quests. Returns display lines for the UI.
    pub fn complete_quest_via_dialogue(&mut self, quest_id: u32) -> Vec<String> {
        let mut notifications = Vec::new();
        let mut rewards_to_give: Option<Vec<QuestReward>> = None;

        for quest in &mut self.quests {
            if quest.id == quest_id && !quest.completed {
                quest.completed = true;
                rewards_to_give = Some(quest.rewards.clone());
                notifications.push(format!("*** Quest complete: {} ***", quest.name));
                break;
            }
        }

        if let Some(rewards) = rewards_to_give {
            notifications.extend(self.collect_rewards(&rewards));
            self.unlock_follow_up_quests(&[quest_id], &mut notifications);
        }
        notifications
    }

    /// Give the player a quest by id. Does nothing if already held.
    pub fn give_quest(&mut self, quest_id: u32) -> bool {
        if self.quests.iter().any(|q| q.id == quest_id) {
            return false;
        }
        if let Some(quest) = quest_by_id(quest_id) {
            self.quests.push(quest);
            return true;
        }
        false
    }

    /// True if the player currently holds the item (qty >= 1).
    pub fn has_item(&self, item_id: u32) -> bool {
        self.inventory.get(&item_id).copied().unwrap_or(0) >= 1
    }

    /// True if item_id matches any equipped item (weapon or armor slot).
    pub fn has_item_equipped(&self, item_id: u32) -> bool {
        if self.equipped_weapon.as_ref().map(|w| w.id) == Some(item_id) {
            return true;
        }
        self.armor_slots.values().any(|a| a.id == item_id)
    }

    /// After completing quests, check if any follow-up quests should be unlocked.
    fn unlock_follow_up_quests(&mut self, completed_ids: &[u32], notifications: &mut Vec<String>) {
        for &completed_id in completed_ids {
            let to_add: Vec<_> = all_quests()
                .iter()
                .filter(|q| q.prerequisite_id == Some(completed_id))
                .filter(|q| q.auto_assign)
                .filter(|q| !self.quests.iter().any(|pq| pq.id == q.id))
                .cloned()
                .collect();
            for quest in to_add {
                notifications.push(format!("  New quest unlocked: {}", quest.name));
                self.quests.push(quest);
            }
        }
    }

    pub fn respawn(&mut self, map: &mut Map) {
        self.health = self.max_health;
        self.in_combat = false;
        self.facing = Direction::Down;

        // Restore whatever tile was under the player at the death position
        let old_x = map.player_x;
        let old_y = map.player_y;
        let old_tile = if old_x == map.campfire_x && old_y == map.campfire_y {
            crate::map::Tile::Campfire
        } else if map.stumps.iter().any(|&(sx, sy, _)| sx == old_x && sy == old_y) {
            crate::map::Tile::Stump
        } else {
            crate::map::Tile::Empty
        };
        map.tiles[old_y][old_x] = old_tile;

        // Move coordinates to campfire respawn point
        map.player_x = map.campfire_x;
        map.player_y = if map.campfire_y > 0 { map.campfire_y - 1 } else { 0 };
        if map.player_y >= map.height {
            map.player_y = map.height - 1;
        }

        // Place the player tile at the new position
        map.tiles[map.player_y][map.player_x] = crate::map::Tile::Player;
    }

    pub fn add_experience_to_skill(&mut self, skill_name: &str, amount: f32) {
        if let Some(skill) = self.skills.get_mut(skill_name) {
            skill.add_experience(amount as f64);
            println!("{} gained {} XP.", skill_name, amount);
        } else {
            println!("Skill not found: {}", skill_name);
        }
    }

}
