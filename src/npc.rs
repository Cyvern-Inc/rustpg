use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::map::Tile;
use crate::player::Player;

// ---------------------------------------------------------------------------
// Interaction system
// ---------------------------------------------------------------------------

/// A condition that can be evaluated against the player's current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    PlayerEquipping(u32),   // item_id equipped in weapon or armor slot
    PlayerHasItem(u32),     // item_id present in inventory
    QuestActive(u32),       // quest_id is in progress (given and not completed)
    QuestCompleted(u32),    // quest_id is completed
    Not(Box<Condition>),
}

impl Condition {
    pub fn evaluate(&self, player: &Player) -> bool {
        match self {
            Condition::PlayerEquipping(id) => player.has_item_equipped(*id),
            Condition::PlayerHasItem(id)   => player.has_item(*id),
            Condition::QuestActive(id)     => player.quests.iter().any(|q| q.id == *id && !q.completed),
            Condition::QuestCompleted(id)  => player.quests.iter().any(|q| q.id == *id && q.completed),
            Condition::Not(inner)          => !inner.evaluate(player),
        }
    }
}

/// What happens when the player interacts with an NPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionAction {
    Combat,
    OpenDialogue(String), // dialogue root node id
    Flee,
    Ignore,
}

/// One row in an NPC's behavior table. Higher priority is evaluated first.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub condition: Option<Condition>,
    pub action: InteractionAction,
    pub priority: u8,
}

/// Evaluate an NPC's interaction table against the player and return the
/// winning action. Returns `Combat` as the implicit default when:
///   - no explicit interactions are defined, AND
///   - the NPC has non-zero attack stats (i.e. it's a hostile enemy).
pub fn resolve_interaction(npc: &OverworldNpc, player: &Player) -> InteractionAction {
    let mut sorted: Vec<&Interaction> = npc.interactions.iter().collect();
    sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

    for interaction in sorted {
        let passes = match &interaction.condition {
            Some(cond) => cond.evaluate(player),
            None => true,
        };
        if passes {
            return interaction.action.clone();
        }
    }

    // Implicit fallback: hostile NPCs default to combat
    if npc.attack > 0 {
        InteractionAction::Combat
    } else {
        InteractionAction::Ignore
    }
}

// ---------------------------------------------------------------------------
// Spawn configuration (static, not serialized)
// ---------------------------------------------------------------------------

/// Per-type configuration used when spawning NPCs onto the map.
/// These values encode the behavioral fingerprint the player learns over time.
pub struct NpcSpawnConfig {
    pub enemy_name: &'static str,
    pub health: i32,
    pub attack: i32,
    pub loot_table: &'static str,
    /// Probability (0.0–1.0) of moving toward the player on each tick
    /// when the player is within sight range.
    pub aggression: f32,
    /// Maximum Manhattan distance at which this NPC can detect the player.
    pub sight_range: usize,
    /// This NPC moves once every N player steps.
    pub move_every: u32,
    /// How many of this type to scatter across a new map.
    pub count: usize,
    /// Preferred spawn biome: "forest", "rocky", "riverside", or "plains".
    /// None = no preference (spawns anywhere).
    pub preferred_biome: Option<&'static str>,
    /// Whether this NPC stays in place (skips movement tick).
    pub stationary: bool,
    /// The tile type placed on the map when this NPC occupies a cell.
    pub tile_type: Tile,
    /// Interaction overrides applied on top of the combat default.
    pub interactions: Vec<Interaction>,
    /// Dialogue entry point (root node id). None = no dialogue.
    pub dialogue_root: Option<&'static str>,
}

/// All NPC types and their behavioral parameters.
///
/// Behavioral fingerprints (what an experienced player notices):
///   Wolf     — moves every step, very aggressive, huge sight → fast and relentless
///   Goblin   — moves often, moderate aggression          → erratic, unpredictable
///   Skeleton — moves slowly, high aggression             → methodical, steady
///   Orc      — very slow, very aggressive                → lumbering but dangerous
///   Bandit   — moderate speed, low aggression, far sight → aware but cautious
///   Troll    — very slow, moderate aggression, short sight→ barely notices you
/// Item ID for the Goblin Mask — used in the goblin interaction condition.
pub const GOBLIN_MASK_ID: u32 = 100053;

pub fn npc_definitions() -> Vec<NpcSpawnConfig> {
    vec![
        NpcSpawnConfig {
            enemy_name: "Goblin",
            health: 30,
            attack: 5,
            loot_table: "common",
            aggression: 0.6,
            sight_range: 8,
            move_every: 2,
            count: 3,
            preferred_biome: Some("forest"),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![
                // When wearing the goblin mask, goblins talk instead of fight
                Interaction {
                    condition: Some(Condition::PlayerEquipping(GOBLIN_MASK_ID)),
                    action: InteractionAction::OpenDialogue("goblin_neutral_greeting".to_string()),
                    priority: 10,
                },
            ],
            dialogue_root: Some("goblin_neutral_greeting"),
        },
        NpcSpawnConfig {
            enemy_name: "Orc",
            health: 50,
            attack: 10,
            loot_table: "uncommon",
            aggression: 0.85,
            sight_range: 7,
            move_every: 3,
            count: 3,
            preferred_biome: Some("rocky"),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![],
            dialogue_root: None,
        },
        NpcSpawnConfig {
            enemy_name: "Bandit",
            health: 40,
            attack: 8,
            loot_table: "common_food",
            aggression: 0.3,
            sight_range: 10,
            move_every: 2,
            count: 4,
            preferred_biome: Some("riverside"),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![],
            dialogue_root: None,
        },
        NpcSpawnConfig {
            enemy_name: "Wolf",
            health: 35,
            attack: 7,
            loot_table: "uncommon",
            aggression: 0.9,
            sight_range: 12,
            move_every: 1,
            count: 5,
            preferred_biome: Some("forest"),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![],
            dialogue_root: None,
        },
        NpcSpawnConfig {
            enemy_name: "Skeleton",
            health: 45,
            attack: 9,
            loot_table: "uncommon",
            aggression: 0.7,
            sight_range: 6,
            move_every: 3,
            count: 3,
            preferred_biome: Some("rocky"),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![],
            dialogue_root: None,
        },
        NpcSpawnConfig {
            enemy_name: "Troll",
            health: 80,
            attack: 15,
            loot_table: "rare",
            aggression: 0.5,
            sight_range: 5,
            move_every: 4,
            count: 2,
            preferred_biome: Some("forest"),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![],
            dialogue_root: None,
        },
    ]
}

/// The Woodsman — spawned near the player campfire. Not part of the random
/// scatter pool; placed explicitly by Map::new().
pub fn woodsman_config() -> NpcSpawnConfig {
    NpcSpawnConfig {
        enemy_name: "Woodsman",
        health: 0,
        attack: 0,
        loot_table: "",
        aggression: 0.0,
        sight_range: 0,
        move_every: u32::MAX,
        count: 1,
        preferred_biome: None,
        stationary: true,
        tile_type: Tile::Npc,
        interactions: vec![
            Interaction {
                condition: None,
                action: InteractionAction::OpenDialogue("woodsman_gateway".to_string()),
                priority: 0,
            },
        ],
        dialogue_root: Some("woodsman_gateway"),
    }
}

// ---------------------------------------------------------------------------
// Encampment — a persistent overworld structure that respawns enemies
// ---------------------------------------------------------------------------

/// A permanent enemy encampment placed on the map at world-gen time.
/// The camp structure (campfire + huts) stays on the map even when all
/// enemies are killed. Enemies respawn from the camp every `respawn_moves`
/// player steps after the last death.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encampment {
    /// Center tile (EnemyCampfire) position.
    pub x: usize,
    pub y: usize,
    /// Enemy type that spawns here.
    pub enemy_name: String,
    pub health: i32,
    pub attack: i32,
    pub loot_table: String,
    pub aggression: f32,
    pub sight_range: usize,
    pub move_every: u32,
    /// Maximum number of live NPCs this camp will maintain.
    pub max_count: usize,
    /// Player steps to wait before spawning a replacement NPC.
    pub respawn_moves: u64,
    /// The move_count value at which the next spawn should occur.
    pub next_respawn: u64,
}

// ---------------------------------------------------------------------------
// Live NPC instance (serialized — persists in the save file)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverworldNpc {
    pub x: usize,
    pub y: usize,
    pub enemy_name: String,
    pub health: i32,
    pub attack: i32,
    pub loot_table: String,
    /// Chance per move tick to step toward the player when in sight.
    pub aggression: f32,
    pub sight_range: usize,
    /// The tile that was at this position before the NPC moved here.
    /// Restored when the NPC vacates the tile.
    pub underlying_tile: Tile,
    /// Counts player steps since the NPC last moved.
    pub move_counter: u32,
    /// How many player steps between each NPC move.
    pub move_every: u32,
    /// The encampment (x, y) this NPC belongs to, if any.
    #[serde(default)]
    pub home_camp: Option<(usize, usize)>,
    /// When true the NPC never wanders or chases — movement tick is skipped.
    #[serde(default)]
    pub stationary: bool,
    /// Which tile to place on the map grid when this NPC occupies a cell.
    /// Defaults to Tile::Enemy for backwards compatibility.
    #[serde(default = "default_enemy_tile")]
    pub tile_type: Tile,
    /// Conditional interaction overrides (evaluated before combat fallback).
    #[serde(default)]
    pub interactions: Vec<Interaction>,
    /// Dialogue tree entry point. None = no dialogue.
    #[serde(default)]
    pub dialogue_root: Option<String>,
}

fn default_enemy_tile() -> Tile { Tile::Enemy }

impl OverworldNpc {
    pub fn new(x: usize, y: usize, cfg: &NpcSpawnConfig) -> Self {
        OverworldNpc {
            x,
            y,
            enemy_name: cfg.enemy_name.to_string(),
            health: cfg.health,
            attack: cfg.attack,
            loot_table: cfg.loot_table.to_string(),
            aggression: cfg.aggression,
            sight_range: cfg.sight_range,
            underlying_tile: Tile::Empty,
            move_counter: 0,
            move_every: cfg.move_every,
            home_camp: None,
            stationary: cfg.stationary,
            tile_type: cfg.tile_type,
            interactions: cfg.interactions.clone(),
            dialogue_root: cfg.dialogue_root.map(|s| s.to_string()),
        }
    }

    /// Construct a camp-linked NPC. Sets `home_camp` automatically.
    pub fn from_camp(x: usize, y: usize, camp: &Encampment) -> Self {
        OverworldNpc {
            x,
            y,
            enemy_name: camp.enemy_name.clone(),
            health: camp.health,
            attack: camp.attack,
            loot_table: camp.loot_table.clone(),
            aggression: camp.aggression,
            sight_range: camp.sight_range,
            underlying_tile: Tile::Empty,
            move_counter: 0,
            move_every: camp.move_every,
            home_camp: Some((camp.x, camp.y)),
            stationary: false,
            tile_type: Tile::Enemy,
            interactions: vec![],
            dialogue_root: None,
        }
    }

    /// Advance this NPC one game step.
    ///
    /// `occupied` is a slice of (x, y) positions held by *other* NPCs — used
    /// to prevent two NPCs occupying the same tile.
    ///
    /// Returns `true` if the NPC stepped onto the player's tile, which the
    /// caller should treat as a combat trigger.
    pub fn tick(
        &mut self,
        tiles: &mut Vec<Vec<Tile>>,
        player_x: usize,
        player_y: usize,
        width: usize,
        height: usize,
        occupied: &[(usize, usize)],
        rng: &mut impl Rng,
    ) -> bool {
        if self.stationary {
            return false;
        }
        self.move_counter += 1;
        if self.move_counter < self.move_every {
            return false;
        }
        self.move_counter = 0;

        let in_sight =
            manhattan(self.x, self.y, player_x, player_y) <= self.sight_range;

        let (new_x, new_y) = if in_sight && rng.gen::<f32>() < self.aggression {
            step_toward(
                self.x, self.y, player_x, player_y,
                tiles, width, height, occupied, rng,
            )
        } else {
            random_step(self.x, self.y, tiles, width, height, occupied, rng)
        };

        // No movement possible
        if new_x == self.x && new_y == self.y {
            return false;
        }

        // Combat: NPC walked into the player
        if new_x == player_x && new_y == player_y {
            return true;
        }

        // Vacate old tile, occupy new tile
        tiles[self.y][self.x] = self.underlying_tile;
        self.underlying_tile = tiles[new_y][new_x];
        tiles[new_y][new_x] = self.tile_type;
        self.x = new_x;
        self.y = new_y;

        false
    }
}

// ---------------------------------------------------------------------------
// Private movement helpers
// ---------------------------------------------------------------------------

fn manhattan(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    let dx = (x1 as isize - x2 as isize).abs() as usize;
    let dy = (y1 as isize - y2 as isize).abs() as usize;
    dx + dy
}

/// A tile the NPC can move onto (never trees, rocks, campfire, or other NPCs).
fn is_passable(
    x: usize,
    y: usize,
    tiles: &[Vec<Tile>],
    occupied: &[(usize, usize)],
) -> bool {
    matches!(tiles[y][x], Tile::Empty | Tile::Stump) && !occupied.contains(&(x, y))
}

/// Move one step toward (to_x, to_y). Prefers the axis with the larger gap;
/// breaks ties randomly. Falls back to the perpendicular axis if blocked.
/// Returns the player's position directly when adjacent — the caller handles
/// the combat trigger rather than the passability check.
fn step_toward(
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
    tiles: &[Vec<Tile>],
    width: usize,
    height: usize,
    occupied: &[(usize, usize)],
    rng: &mut impl Rng,
) -> (usize, usize) {
    let dx = to_x as isize - from_x as isize;
    let dy = to_y as isize - from_y as isize;

    // Candidate step in the horizontal direction
    let h_step = if dx > 0 {
        (from_x + 1, from_y)
    } else {
        (from_x.wrapping_sub(1), from_y)
    };
    // Candidate step in the vertical direction
    let v_step = if dy > 0 {
        (from_x, from_y + 1)
    } else {
        (from_x, from_y.wrapping_sub(1))
    };

    // Prefer the axis closing the larger gap; break ties randomly
    let prefer_h =
        dx.abs() > dy.abs() || (dx.abs() == dy.abs() && rng.gen_bool(0.5));

    let mut candidates: Vec<(usize, usize)> = Vec::new();
    if prefer_h {
        if dx != 0 {
            candidates.push(h_step);
        }
        if dy != 0 {
            candidates.push(v_step);
        }
    } else {
        if dy != 0 {
            candidates.push(v_step);
        }
        if dx != 0 {
            candidates.push(h_step);
        }
    }

    for (nx, ny) in candidates {
        if nx >= width || ny >= height {
            continue;
        }
        // Allow stepping onto the player tile — combat check is in tick()
        if nx == to_x && ny == to_y {
            return (nx, ny);
        }
        if is_passable(nx, ny, tiles, occupied) {
            return (nx, ny);
        }
    }

    (from_x, from_y) // Blocked — stay put
}

/// Move one step in a random passable direction.
fn random_step(
    from_x: usize,
    from_y: usize,
    tiles: &[Vec<Tile>],
    width: usize,
    height: usize,
    occupied: &[(usize, usize)],
    rng: &mut impl Rng,
) -> (usize, usize) {
    // All four cardinal directions in a shuffled order
    let mut dirs: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    for i in (1..4usize).rev() {
        let j = rng.gen_range(0..=i);
        dirs.swap(i, j);
    }

    for (ddx, ddy) in dirs {
        let nx = from_x as isize + ddx;
        let ny = from_y as isize + ddy;
        if nx < 0 || ny < 0 {
            continue;
        }
        let nx = nx as usize;
        let ny = ny as usize;
        if nx < width && ny < height && is_passable(nx, ny, tiles, occupied) {
            return (nx, ny);
        }
    }

    (from_x, from_y) // Nowhere to go
}
