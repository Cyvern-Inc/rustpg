use std::fmt;
use rand::Rng;
use serde::{Serialize, Deserialize};
use term_size;

use crate::npc::{npc_definitions, woodsman_config, Encampment, OverworldNpc};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Tile {
    Empty,
    Player,
    Tree,
    Rock,
    Campfire,
    Stump,
    Enemy,
    Water,
    /// Centre of a goblin encampment. Impassable.
    EnemyCampfire,
    /// Hut/tent structure belonging to an encampment. Impassable.
    Hut,
    /// A friendly/neutral NPC on the map. Interactable via "talk".
    Npc,
}

impl Tile {
    pub fn render(&self) -> &str {
        match self {
            Tile::Empty          => "\x1B[90m.\x1B[0m",
            Tile::Player         => "\x1B[93mP\x1B[0m",
            Tile::Tree           => "\x1B[32mt\x1B[0m",
            Tile::Rock           => "\x1B[37mr\x1B[0m",
            Tile::Campfire       => "\x1B[91m#\x1B[0m",
            Tile::Stump          => "\x1B[33ms\x1B[0m",
            Tile::Enemy          => "\x1B[31mE\x1B[0m",
            Tile::Water          => "\x1B[34m~\x1B[0m",
            Tile::EnemyCampfire  => "\x1B[31m*\x1B[0m",
            Tile::Hut            => "\x1B[33mH\x1B[0m",
            Tile::Npc            => "\x1B[93mN\x1B[0m",
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Tile::Empty         => '.',
            Tile::Player        => 'P',
            Tile::Tree          => 't',
            Tile::Rock          => 'r',
            Tile::Campfire      => '#',
            Tile::Stump         => 's',
            Tile::Enemy         => 'E',
            Tile::Water         => '~',
            Tile::EnemyCampfire => 'f',
            Tile::Hut           => 'H',
            Tile::Npc           => 'N',
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Empty,
            'P' => Tile::Player,
            't' => Tile::Tree,
            'r' => Tile::Rock,
            '#' => Tile::Campfire,
            's' => Tile::Stump,
            'E' => Tile::Enemy,
            '~' => Tile::Water,
            'f' => Tile::EnemyCampfire,
            'H' => Tile::Hut,
            'N' => Tile::Npc,
            _   => Tile::Empty,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub player_x: usize,
    pub player_y: usize,
    pub view_radius: usize,
    pub campfire_x: usize,
    pub campfire_y: usize,
    /// Incremented by 1 on every successful player step.
    pub move_count: u64,
    /// Tracks depleted tree stumps as (x, y, move_count_at_depletion).
    /// A stump regenerates once move_count exceeds its creation point by STUMP_REGEN_MOVES.
    pub stumps: Vec<(usize, usize, u64)>,
    /// All live NPC instances on this map.
    pub npcs: Vec<OverworldNpc>,
    /// Persistent goblin encampments placed at world-gen time.
    #[serde(default)]
    pub encampments: Vec<Encampment>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();

        let player_x = width / 2;
        let player_y = height / 2;
        let campfire_x = player_x;
        let campfire_y = player_y + 1;

        let mut tiles = generate_terrain(width, height, &mut rng);
        clear_spawn_area(&mut tiles, player_x, player_y, 6, width, height);

        tiles[player_y][player_x] = Tile::Player;
        if campfire_y < height {
            tiles[campfire_y][campfire_x] = Tile::Campfire;
        }

        let mut map = Self {
            width,
            height,
            tiles,
            player_x,
            player_y,
            view_radius: 15,
            campfire_x,
            campfire_y,
            move_count: 0,
            stumps: Vec::new(),
            npcs: Vec::new(),
            encampments: Vec::new(),
        };
        // Encampments first — scattered NPCs won't land on their tiles
        map.place_encampments(&mut rng);
        map.spawn_npcs(&mut rng);
        map.spawn_woodsman();
        map
    }

    pub fn move_player(&mut self, direction: &Direction) {
        let (new_x, new_y) = match direction {
            Direction::Up => (self.player_x, self.player_y.saturating_sub(1)),
            Direction::Down => (
                self.player_x,
                usize::min(self.player_y + 1, self.height - 1),
            ),
            Direction::Left => (self.player_x.saturating_sub(1), self.player_y),
            Direction::Right => (
                usize::min(self.player_x + 1, self.width - 1),
                self.player_y,
            ),
        };

        let passable = matches!(
            self.tiles[new_y][new_x],
            Tile::Empty | Tile::Campfire | Tile::Stump
        );
        if passable {
            // Restore the vacated tile to whatever it actually was
            let old_tile = if self.player_x == self.campfire_x && self.player_y == self.campfire_y {
                Tile::Campfire
            } else if self.stumps.iter().any(|&(sx, sy, _)| sx == self.player_x && sy == self.player_y) {
                Tile::Stump
            } else {
                Tile::Empty
            };
            self.tiles[self.player_y][self.player_x] = old_tile;
            self.player_x = new_x;
            self.player_y = new_y;
            self.tiles[self.player_y][self.player_x] = Tile::Player;
            self.move_count += 1;
        }
    }

    /// Number of moves before a stump regenerates into a tree.
    pub const STUMP_REGEN_MOVES: u64 = 100;

    /// Restore any stumps that have been depleted long enough.
    /// Call once per game-loop iteration before rendering.
    pub fn regenerate_stumps(&mut self) {
        let threshold = self.move_count;
        let mut i = 0;
        while i < self.stumps.len() {
            let (x, y, created_at) = self.stumps[i];
            if threshold.saturating_sub(created_at) >= Self::STUMP_REGEN_MOVES {
                // Only restore if the tile is still a stump (not overwritten)
                if self.tiles[y][x] == Tile::Stump {
                    self.tiles[y][x] = Tile::Tree;
                }
                self.stumps.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Mark a tree tile at (x, y) as depleted: set to Stump and record it.
    pub fn deplete_tree(&mut self, x: usize, y: usize) {
        if self.tiles[y][x] == Tile::Tree {
            self.tiles[y][x] = Tile::Stump;
            self.stumps.push((x, y, self.move_count));
        }
    }

    pub fn render(&self) -> String {
        self.render_viewport(self.view_radius, self.view_radius)
    }

    /// Render the viewport with explicit horizontal and vertical radii.
    /// Tiles visible = (2*h_radius + 1) wide by (2*v_radius + 1) tall,
    /// centred on the player. Both axes are clamped to the map boundaries.
    pub fn render_viewport(&self, h_radius: usize, v_radius: usize) -> String {
        let start_x = self.player_x.saturating_sub(h_radius);
        let end_x   = usize::min(self.player_x + h_radius, self.width  - 1);
        let start_y = self.player_y.saturating_sub(v_radius);
        let end_y   = usize::min(self.player_y + v_radius, self.height - 1);

        let mut output = String::new();
        for y in start_y..=end_y {
            for x in start_x..=end_x {
                output.push_str(self.tiles[y][x].render());
                output.push(' ');
            }
            output.push('\n');
        }
        output
    }

    pub fn render_full(&self) -> String {
        if let Some((term_width, term_height)) = term_size::dimensions() {
            // Each tile is two characters wide (symbol + space)
            let tile_width = 2;
            let viewport_width = term_width / tile_width;
            let viewport_height = term_height - 2; // Adjust for any UI elements

            let half_viewport_width = viewport_width / 2;
            let half_viewport_height = viewport_height / 2;

            let start_x = if self.player_x >= half_viewport_width {
                self.player_x - half_viewport_width
            } else {
                0
            };

            let end_x = usize::min(start_x + viewport_width - 1, self.width - 1);

            let start_y = if self.player_y >= half_viewport_height {
                self.player_y - half_viewport_height
            } else {
                0
            };

            let end_y = usize::min(start_y + viewport_height - 1, self.height - 1);

            let mut output = String::new();

            for y in start_y..=end_y {
                for x in start_x..=end_x {
                    output.push_str(self.tiles[y][x].render());
                    output.push(' ');
                }
                output.push('\n');
            }

            output
        } else {
            self.render()
        }
    }

    pub fn serialize_map(&self) -> String {
        let mut serialized = String::new();
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let tile_char = tile.to_char();
                serialized.push(tile_char);
                log::debug!("Serializing Tile at ({}, {}): {}", x, y, tile_char);
            }
            serialized.push('\n');
        }
        serialized
    }

    pub fn deserialize_map(
        width: usize,
        height: usize,
        data: &str,
        player_x: usize,
        player_y: usize,
    ) -> Self {
        let mut tiles = vec![vec![Tile::Empty; width]; height];
        
        // Parse the map data
        for (y, line) in data.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                tiles[y][x] = Tile::from_char(ch);
            }
        }

        // Set the player's position
        tiles[player_y][player_x] = Tile::Player;

        // campfire_x/y are restored by the caller from CharacterSave.game_map after deserialization
        let campfire_x = 0;
        let campfire_y = 0;

        Map {
            width,
            height,
            tiles,
            player_x,
            player_y,
            view_radius: 15,
            campfire_x,
            campfire_y,
            move_count: 0,
            stumps: Vec::new(),
            // NPCs and encampments are restored from character.json via restore_runtime_fields
            npcs: Vec::new(),
            encampments: Vec::new(),
        }
    }

    /// Copy the runtime fields that `deserialize_map` leaves at default values
    /// back from a full saved `Map`. Moves `stumps` and `npcs` instead of cloning.
    pub fn restore_runtime_fields(&mut self, saved: Map) {
        self.campfire_x   = saved.campfire_x;
        self.campfire_y   = saved.campfire_y;
        self.move_count   = saved.move_count;
        self.stumps       = saved.stumps;
        self.npcs         = saved.npcs;
        self.encampments  = saved.encampments;
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if y < self.height && x < self.width {
            self.tiles[y][x] = tile;
        }
    }

    // -----------------------------------------------------------------------
    // NPC management
    // -----------------------------------------------------------------------

    /// Scatter NPCs across the map on a new game. Called once from `new()`.
    /// NPCs with a `preferred_biome` are placed preferentially in matching
    /// terrain; they still have a small chance to spawn elsewhere so counts
    /// are always filled even on maps with few matching tiles.
    fn spawn_npcs(&mut self, rng: &mut impl Rng) {
        let defs = npc_definitions();
        let center_x = self.player_x;
        let center_y = self.player_y;
        const MIN_SPAWN_DIST: usize = 8;
        const MAX_ATTEMPTS: usize = 500;
        /// Probability of accepting an off-biome tile when a preference is set.
        const OFF_BIOME_CHANCE: f64 = 0.15;

        for def in &defs {
            let mut spawned = 0;
            let mut attempts = 0;
            while spawned < def.count && attempts < MAX_ATTEMPTS {
                attempts += 1;
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);

                if self.tiles[y][x] != Tile::Empty {
                    continue;
                }
                let dx = (x as isize - center_x as isize).abs() as usize;
                let dy = (y as isize - center_y as isize).abs() as usize;
                if dx + dy < MIN_SPAWN_DIST {
                    continue;
                }
                if self.npcs.iter().any(|n| n.x == x && n.y == y) {
                    continue;
                }

                if let Some(preferred) = def.preferred_biome {
                    let biome =
                        classify_biome(&self.tiles, x, y, self.width, self.height);
                    if biome != preferred && !rng.gen_bool(OFF_BIOME_CHANCE) {
                        continue;
                    }
                }

                let npc = OverworldNpc::new(x, y, def);
                self.tiles[y][x] = def.tile_type;
                self.npcs.push(npc);
                spawned += 1;
            }
        }
    }

    /// Place the Woodsman NPC 2 tiles to the right of the player campfire.
    /// Tries up to 8 offsets clockwise if the preferred spot is occupied.
    fn spawn_woodsman(&mut self) {
        let cfg = woodsman_config();
        let candidates = [
            (self.campfire_x + 2, self.campfire_y),
            (self.campfire_x + 1, self.campfire_y - 1),
            (self.campfire_x,     self.campfire_y - 2),
            (self.campfire_x - 1, self.campfire_y - 1),
            (self.campfire_x - 2, self.campfire_y),
            (self.campfire_x + 1, self.campfire_y + 1),
            (self.campfire_x,     self.campfire_y + 2),
            (self.campfire_x - 1, self.campfire_y + 1),
        ];
        for (x, y) in candidates {
            if x >= self.width || y >= self.height {
                continue;
            }
            if self.tiles[y][x] != Tile::Empty {
                continue;
            }
            if self.npcs.iter().any(|n| n.x == x && n.y == y) {
                continue;
            }
            let npc = OverworldNpc::new(x, y, &cfg);
            self.tiles[y][x] = Tile::Npc;
            self.npcs.push(npc);
            return;
        }
    }

    /// Advance all NPCs one game step. Returns `Some(index)` if the NPC at
    /// that index in `self.npcs` walked onto the player's tile — the caller
    /// should remove it and initiate combat. Only the first collision per tick
    /// is returned.
    ///
    /// Uses `std::mem::take` to avoid a double-mutable-borrow between
    /// `self.npcs` and `self.tiles`.
    pub fn update_npcs(&mut self, rng: &mut impl Rng) -> Option<usize> {
        // Snapshot positions so each NPC avoids its peers during this tick
        let mut positions: Vec<(usize, usize)> =
            self.npcs.iter().map(|n| (n.x, n.y)).collect();

        let px = self.player_x;
        let py = self.player_y;
        let w = self.width;
        let h = self.height;

        // Take ownership of npcs to allow `&mut self.tiles` simultaneously
        let mut npcs = std::mem::take(&mut self.npcs);

        let mut combat_idx: Option<usize> = None;

        for i in 0..npcs.len() {
            // Other NPCs' current positions (updated each iteration)
            let occupied: Vec<(usize, usize)> = positions
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, &p)| p)
                .collect();

            let hit_player = npcs[i].tick(
                &mut self.tiles,
                px, py, w, h,
                &occupied,
                rng,
            );

            // Keep position snapshot in sync for subsequent NPCs
            positions[i] = (npcs[i].x, npcs[i].y);

            if hit_player && combat_idx.is_none() {
                combat_idx = Some(i);
            }
        }

        self.npcs = npcs;
        combat_idx
    }

    /// Clears all player positions from the map.
    pub fn clear_player_positions(&mut self) {
        for row in &mut self.tiles {
            for tile in row.iter_mut() {
                if *tile == Tile::Player {
                    *tile = Tile::Empty; // Or whatever represents an empty tile
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Encampment management
    // -----------------------------------------------------------------------

    /// Place 2–3 goblin encampments near forest edges on a newly generated map.
    /// Called from `new()` before `spawn_npcs` so scattered NPCs never land on
    /// the campfire or hut tiles.
    fn place_encampments(&mut self, rng: &mut impl Rng) {
        const NUM_CAMPS: usize = 6;
        const MIN_SPAWN_DIST: usize = 20;   // Manhattan distance from player spawn
        const MIN_CAMP_DIST: usize  = 30;   // Manhattan distance between camps
        const TREE_RADIUS: isize    = 12;   // Must have a tree within this many tiles
        const CLEAR_RADIUS: isize   = 3;    // Radius to clear trees around the centre
        const MAX_ATTEMPTS: usize   = 300;
        const INITIAL_GOBLINS: usize = 2;

        // Hut positions relative to the campfire centre (asymmetric layout)
        const HUT_OFFSETS: [(isize, isize); 3] = [(-2, -1), (2, 0), (-1, 2)];

        let center_x = self.player_x;
        let center_y = self.player_y;
        let mut placed: Vec<(usize, usize)> = Vec::new();

        'camp: for _ in 0..NUM_CAMPS {
            for _ in 0..MAX_ATTEMPTS {
                let x = rng.gen_range(10..self.width.saturating_sub(10));
                let y = rng.gen_range(10..self.height.saturating_sub(10));

                if self.tiles[y][x] != Tile::Empty { continue; }

                // Far enough from player spawn
                let dx = (x as isize - center_x as isize).abs() as usize;
                let dy = (y as isize - center_y as isize).abs() as usize;
                if dx + dy < MIN_SPAWN_DIST { continue; }

                // Far enough from existing camps
                if placed.iter().any(|&(cx, cy)| {
                    let ddx = (x as isize - cx as isize).abs() as usize;
                    let ddy = (y as isize - cy as isize).abs() as usize;
                    ddx + ddy < MIN_CAMP_DIST
                }) { continue; }

                // Near at least one tree tile
                let near_tree = 'search: {
                    for ty in -TREE_RADIUS..=TREE_RADIUS {
                        for tx in -TREE_RADIUS..=TREE_RADIUS {
                            let nx = x as isize + tx;
                            let ny = y as isize + ty;
                            if nx >= 0 && ny >= 0
                                && (nx as usize) < self.width
                                && (ny as usize) < self.height
                                && self.tiles[ny as usize][nx as usize] == Tile::Tree
                            {
                                break 'search true;
                            }
                        }
                    }
                    false
                };
                if !near_tree { continue; }

                // All hut offsets within map bounds
                let huts_ok = HUT_OFFSETS.iter().all(|&(hx, hy)| {
                    let nx = x as isize + hx;
                    let ny = y as isize + hy;
                    nx >= 0 && ny >= 0
                        && (nx as usize) < self.width
                        && (ny as usize) < self.height
                });
                if !huts_ok { continue; }

                // Valid location — build the camp
                placed.push((x, y));

                // Clear trees in the camp clearing
                for ty in -CLEAR_RADIUS..=CLEAR_RADIUS {
                    for tx in -CLEAR_RADIUS..=CLEAR_RADIUS {
                        if tx * tx + ty * ty <= CLEAR_RADIUS * CLEAR_RADIUS {
                            let nx = x as isize + tx;
                            let ny = y as isize + ty;
                            if nx >= 0 && ny >= 0
                                && (nx as usize) < self.width
                                && (ny as usize) < self.height
                            {
                                let (ux, uy) = (nx as usize, ny as usize);
                                if self.tiles[uy][ux] == Tile::Tree {
                                    self.tiles[uy][ux] = Tile::Empty;
                                }
                            }
                        }
                    }
                }

                // Place campfire and huts
                self.tiles[y][x] = Tile::EnemyCampfire;
                for &(hx, hy) in &HUT_OFFSETS {
                    let nx = (x as isize + hx) as usize;
                    let ny = (y as isize + hy) as usize;
                    self.tiles[ny][nx] = Tile::Hut;
                }

                // Build the Encampment record before spawning so from_camp can use it
                let camp = Encampment {
                    x,
                    y,
                    enemy_name: "Goblin".to_string(),
                    health: 10,
                    attack: 4,
                    strength: 4,
                    defense: 3,
                    loot_table: "common".to_string(),
                    aggression: 0.6,
                    sight_range: 8,
                    move_every: 2,
                    max_count: 3,
                    respawn_moves: 60,
                    next_respawn: 0,
                };

                // Spawn initial goblins on adjacent empty tiles
                let adj_dirs: [(isize, isize); 8] = [
                    (1, 0), (-1, 0), (0, 1), (0, -1),
                    (1, 1), (-1, 1), (1, -1), (-1, -1),
                ];
                let mut spawned = 0;
                for (ddx, ddy) in adj_dirs {
                    if spawned >= INITIAL_GOBLINS { break; }
                    let sx = x as isize + ddx;
                    let sy = y as isize + ddy;
                    if sx < 0 || sy < 0 { continue; }
                    let (sx, sy) = (sx as usize, sy as usize);
                    if sx >= self.width || sy >= self.height { continue; }
                    if self.tiles[sy][sx] != Tile::Empty { continue; }

                    let npc = OverworldNpc::from_camp(sx, sy, &camp);
                    self.tiles[sy][sx] = Tile::Enemy;
                    self.npcs.push(npc);
                    spawned += 1;
                }

                self.encampments.push(camp);
                continue 'camp;
            }
            // Could not place camp after MAX_ATTEMPTS — skip
        }
    }

    /// Spawn a replacement goblin for any encampment whose respawn timer has
    /// elapsed and whose live NPC count is below max. Call every game loop tick.
    ///
    /// `fast` halves the effective respawn delay — used by FAF mode to keep
    /// enemies flowing at the camp while the player is actively farming.
    pub fn regenerate_encampments(&mut self, _rng: &mut impl Rng, fast: bool) {
        let move_count = self.move_count;

        // Collect indices of camps that need a spawn this tick
        let mut to_spawn: Vec<(usize, usize, usize)> = Vec::new(); // (spawn_x, spawn_y, camp_idx)

        for (idx, camp) in self.encampments.iter().enumerate() {
            if move_count < camp.next_respawn { continue; }

            let live = self.npcs.iter()
                .filter(|n| n.home_camp == Some((camp.x, camp.y)))
                .count();
            if live >= camp.max_count { continue; }

            if let Some((sx, sy)) = find_spawn_adjacent(
                camp.x, camp.y, &self.tiles, self.width, self.height,
            ) {
                to_spawn.push((sx, sy, idx));
            }
        }

        for (sx, sy, camp_idx) in to_spawn {
            // Fast mode uses a shorter cooldown so enemies flow continuously
            let base = self.encampments[camp_idx].respawn_moves;
            let effective = if fast { (base / 4).max(1) } else { base };
            self.encampments[camp_idx].next_respawn = move_count + effective;

            let npc = OverworldNpc::from_camp(sx, sy, &self.encampments[camp_idx]);
            self.tiles[sy][sx] = Tile::Enemy;
            self.npcs.push(npc);
        }
    }

    /// Called when a camp-linked NPC is killed. Schedules a respawn cooldown
    /// so the camp doesn't immediately replace the NPC.
    pub fn notify_camp_npc_removed(&mut self, camp_x: usize, camp_y: usize) {
        let move_count = self.move_count;
        if let Some(camp) = self.encampments.iter_mut()
            .find(|c| c.x == camp_x && c.y == camp_y)
        {
            let new_respawn = move_count + camp.respawn_moves;
            if new_respawn > camp.next_respawn {
                camp.next_respawn = new_respawn;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Terrain generation helpers
// ---------------------------------------------------------------------------

/// Axis-aligned rectangular zone: (x, y, width, height).
type ZoneRect = (usize, usize, usize, usize);

/// True if the zone overlaps the player's safe spawn circle (radius 30).
fn overlaps_spawn(fx: usize, fy: usize, fw: usize, fh: usize, cx: usize, cy: usize) -> bool {
    const SAFE_R: usize = 30;
    let sl = cx.saturating_sub(SAFE_R);
    let sr = cx + SAFE_R;
    let st = cy.saturating_sub(SAFE_R);
    let sb = cy + SAFE_R;
    !(fx + fw < sl || fx > sr || fy + fh < st || fy > sb)
}

/// Place a handful of forest and rocky zone rectangles, avoiding the spawn
/// center and the map margins.
fn place_biome_zones(
    width: usize,
    height: usize,
    cx: usize,
    cy: usize,
    rng: &mut impl Rng,
) -> (Vec<ZoneRect>, Vec<ZoneRect>) {
    let num_forests = rng.gen_range(2..=4usize);
    let num_rocky   = rng.gen_range(1..=3usize);
    let mut forests: Vec<ZoneRect> = Vec::new();
    let mut rocky:   Vec<ZoneRect> = Vec::new();

    for _ in 0..num_forests {
        for _ in 0..20usize {
            let fw = rng.gen_range(28..=65usize);
            let fh = rng.gen_range(28..=65usize);
            let margin = 10usize;
            if width <= fw + 2 * margin || height <= fh + 2 * margin { break; }
            let fx = rng.gen_range(margin..width  - fw - margin);
            let fy = rng.gen_range(margin..height - fh - margin);
            if !overlaps_spawn(fx, fy, fw, fh, cx, cy) {
                forests.push((fx, fy, fw, fh));
                break;
            }
        }
    }

    for _ in 0..num_rocky {
        for _ in 0..20usize {
            let rw = rng.gen_range(15..=40usize);
            let rh = rng.gen_range(15..=40usize);
            let margin = 8usize;
            if width <= rw + 2 * margin || height <= rh + 2 * margin { break; }
            let rx = rng.gen_range(margin..width  - rw - margin);
            let ry = rng.gen_range(margin..height - rh - margin);
            if !overlaps_spawn(rx, ry, rw, rh, cx, cy) {
                rocky.push((rx, ry, rw, rh));
                break;
            }
        }
    }

    (forests, rocky)
}

/// Generate a winding 2-tile-wide river. Covers the middle 60 % of one map
/// axis so the player can always walk around an open end without crossing.
fn generate_river(width: usize, height: usize, rng: &mut impl Rng) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    // 60 % north→south, 40 % east→west
    if rng.gen_bool(0.6) {
        let y0 = height / 5;
        let y1 = 4 * height / 5;
        let mut x = rng.gen_range(width / 4..3 * width / 4) as isize;
        for y in y0..y1 {
            if rng.gen_bool(0.3) {
                let d: isize = if rng.gen_bool(0.5) { -1 } else { 1 };
                let nx = x + d;
                if nx > 4 && (nx as usize) < width - 4 { x = nx; }
            }
            let xu = x as usize;
            path.push((xu, y));
            if xu + 1 < width { path.push((xu + 1, y)); }
        }
    } else {
        let x0 = width / 5;
        let x1 = 4 * width / 5;
        let mut y = rng.gen_range(height / 4..3 * height / 4) as isize;
        for x in x0..x1 {
            if rng.gen_bool(0.3) {
                let d: isize = if rng.gen_bool(0.5) { -1 } else { 1 };
                let ny = y + d;
                if ny > 4 && (ny as usize) < height - 4 { y = ny; }
            }
            let yu = y as usize;
            path.push((x, yu));
            if yu + 1 < height { path.push((x, yu + 1)); }
        }
    }
    path
}

/// Pick the initial tile for a cell based on which biome zone it falls in.
fn seed_tile_biome(
    x: usize,
    y: usize,
    forests: &[ZoneRect],
    rocky: &[ZoneRect],
    river_set: &std::collections::HashSet<(usize, usize)>,
    rng: &mut impl Rng,
) -> Tile {
    if river_set.contains(&(x, y)) {
        return Tile::Water;
    }

    for &(fx, fy, fw, fh) in forests {
        if x >= fx && x < fx + fw && y >= fy && y < fy + fh {
            // Dense outer ring → near-solid tree wall; sparser interior
            const BORDER: usize = 4;
            let near_border = x < fx + BORDER || x >= fx + fw - BORDER
                || y < fy + BORDER || y >= fy + fh - BORDER;
            let tree_prob: f32 = if near_border { 0.88 } else { 0.55 };
            let roll: f32 = rng.gen();
            return if roll < tree_prob { Tile::Tree }
                   else if roll < tree_prob + 0.02 { Tile::Rock }
                   else { Tile::Empty };
        }
    }

    for &(rx, ry, rw, rh) in rocky {
        if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
            let roll: f32 = rng.gen();
            return if roll < 0.28 { Tile::Rock }
                   else if roll < 0.34 { Tile::Tree }
                   else { Tile::Empty };
        }
    }

    // Plains — slightly varied density for texture
    let roll: f32 = rng.gen();
    if roll < 0.05 { Tile::Rock } else if roll < 0.23 { Tile::Tree } else { Tile::Empty }
}

/// After CA smoothing, punch two open corridors through the north and south
/// faces of each forest so the player has clear entry/exit points.
fn carve_forest_entrances(
    tiles: &mut Vec<Vec<Tile>>,
    forests: &[ZoneRect],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) {
    const GAP_W: usize = 5; // corridor width in tiles
    const GAP_D: usize = 5; // depth to clear inward from the face

    for &(fx, fy, fw, fh) in forests {
        if fw < GAP_W + 8 { continue; }
        let col_min = fx + 4;
        let col_max = fx + fw - GAP_W - 4;
        if col_max <= col_min { continue; }

        // North entrance
        let col_n = rng.gen_range(col_min..col_max);
        for col in col_n..col_n + GAP_W {
            for row in fy..usize::min(fy + GAP_D, height) {
                if col < width && tiles[row][col] == Tile::Tree {
                    tiles[row][col] = Tile::Empty;
                }
            }
        }

        // South entrance (independent random column)
        let col_s = rng.gen_range(col_min..col_max);
        let row0  = (fy + fh).saturating_sub(GAP_D);
        for col in col_s..col_s + GAP_W {
            for row in row0..usize::min(fy + fh, height) {
                if col < width && tiles[row][col] == Tile::Tree {
                    tiles[row][col] = Tile::Empty;
                }
            }
        }
    }
}

/// Classify the dominant terrain around a map coordinate. Returns one of
/// `"forest"`, `"rocky"`, `"riverside"`, or `"plains"`. Used for biome-aware
/// NPC spawning.
fn classify_biome(
    tiles: &[Vec<Tile>],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> &'static str {
    const RADIUS: isize = 7;
    let (mut trees, mut rocks, mut water, mut total) = (0u32, 0u32, 0u32, 0u32);
    for dy in -RADIUS..=RADIUS {
        for dx in -RADIUS..=RADIUS {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
                total += 1;
                match tiles[ny as usize][nx as usize] {
                    Tile::Tree | Tile::Stump => trees += 1,
                    Tile::Rock  => rocks += 1,
                    Tile::Water => water += 1,
                    _ => {}
                }
            }
        }
    }
    if water >= 2 {
        "riverside"
    } else if total > 0 && trees * 100 / total >= 35 {
        "forest"
    } else if total > 0 && rocks * 100 / total >= 15 {
        "rocky"
    } else {
        "plains"
    }
}

/// Seed a grid with biome-aware tile probabilities, then smooth with two
/// cellular-automata passes to produce natural-looking terrain.
fn generate_terrain(width: usize, height: usize, rng: &mut impl Rng) -> Vec<Vec<Tile>> {
    let (cx, cy) = (width / 2, height / 2);
    let (forests, rocky_zones) = place_biome_zones(width, height, cx, cy, rng);

    let num_rivers = rng.gen_range(1..=2usize);
    let river_set: std::collections::HashSet<(usize, usize)> = (0..num_rivers)
        .flat_map(|_| generate_river(width, height, rng))
        .collect();

    let mut tiles = vec![vec![Tile::Empty; width]; height];
    for y in 0..height {
        for x in 0..width {
            tiles[y][x] = seed_tile_biome(x, y, &forests, &rocky_zones, &river_set, rng);
        }
    }

    // Two CA smoothing passes — Water tiles are stable and never overwritten
    for _ in 0..2 {
        let prev = tiles.clone();
        for y in 0..height {
            for x in 0..width {
                if prev[y][x] == Tile::Water { continue; }
                let tree_n = count_tile_neighbors(&prev, x, y, width, height, Tile::Tree);
                let rock_n  = count_tile_neighbors(&prev, x, y, width, height, Tile::Rock);
                let was_tree = prev[y][x] == Tile::Tree;
                let was_rock = prev[y][x] == Tile::Rock;

                tiles[y][x] = if tree_n >= 5 || (was_tree && tree_n >= 4) {
                    Tile::Tree
                } else if rock_n >= 3 || (was_rock && rock_n >= 2) {
                    Tile::Rock
                } else {
                    Tile::Empty
                };
            }
        }
    }

    carve_forest_entrances(&mut tiles, &forests, width, height, rng);

    tiles
}

/// Find an empty tile adjacent (8-directional) to the given position.
/// Used to find a spawn point for encampment goblins.
fn find_spawn_adjacent(
    cx: usize,
    cy: usize,
    tiles: &[Vec<Tile>],
    width: usize,
    height: usize,
) -> Option<(usize, usize)> {
    const DIRS: [(isize, isize); 8] = [
        (1, 0), (-1, 0), (0, 1), (0, -1),
        (1, 1), (-1, 1), (1, -1), (-1, -1),
    ];
    for (dx, dy) in DIRS {
        let nx = cx as isize + dx;
        let ny = cy as isize + dy;
        if nx >= 0 && ny >= 0 {
            let (nx, ny) = (nx as usize, ny as usize);
            if nx < width && ny < height && tiles[ny][nx] == Tile::Empty {
                return Some((nx, ny));
            }
        }
    }
    None
}

/// Count how many of the 8 Moore-neighbourhood cells match `target`.
fn count_tile_neighbors(
    tiles: &[Vec<Tile>],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    target: Tile,
) -> u8 {
    let mut count = 0u8;
    let ix = x as isize;
    let iy = y as isize;
    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = ix + dx;
            let ny = iy + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
                if tiles[ny as usize][nx as usize] == target {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Clear a circular area of terrain around the spawn point so the player
/// always starts with open space around them.
fn clear_spawn_area(
    tiles: &mut Vec<Vec<Tile>>,
    cx: usize,
    cy: usize,
    radius: usize,
    width: usize,
    height: usize,
) {
    let r = radius as isize;
    let icx = cx as isize;
    let icy = cy as isize;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let nx = icx + dx;
                let ny = icy + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
                    let ux = nx as usize;
                    let uy = ny as usize;
                    if tiles[uy][ux] != Tile::Player && tiles[uy][ux] != Tile::Campfire {
                        tiles[uy][ux] = Tile::Empty;
                    }
                }
            }
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
