use rand::Rng;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use crate::enemy::{basic_enemies, Enemy};

// ─── Dungeon Tile ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum DungeonTile {
    Rock,
    Floor,
    Exit,
    Enemy,
    LootPile,
}

impl DungeonTile {
    pub fn render(self) -> &'static str {
        match self {
            DungeonTile::Rock     => "\x1B[90m#\x1B[0m",
            DungeonTile::Floor    => "\x1B[37m.\x1B[0m",
            DungeonTile::Exit     => "\x1B[90mD\x1B[0m",
            DungeonTile::Enemy    => "\x1B[31mE\x1B[0m",
            DungeonTile::LootPile => "\x1B[33mL\x1B[0m",
        }
    }

    pub fn to_char(self) -> char {
        match self {
            DungeonTile::Rock     => '#',
            DungeonTile::Floor    => '.',
            DungeonTile::Exit     => 'D',
            DungeonTile::Enemy    => 'E',
            DungeonTile::LootPile => 'L',
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            '.' => DungeonTile::Floor,
            'D' => DungeonTile::Exit,
            'E' => DungeonTile::Enemy,
            'L' => DungeonTile::LootPile,
            _   => DungeonTile::Rock,
        }
    }

    pub fn is_passable(self) -> bool {
        matches!(self, DungeonTile::Floor | DungeonTile::Exit | DungeonTile::LootPile)
    }
}

// ─── Dungeon Size ─────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DungeonSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl DungeonSize {
    pub fn dimensions(&self, ow: usize, oh: usize) -> (usize, usize) {
        match self {
            DungeonSize::Small      => ((ow / 2).max(40), (oh / 2).max(40)),
            DungeonSize::Medium     => ((ow * 707 / 1000).max(60), (oh * 707 / 1000).max(60)),
            DungeonSize::Large      => (ow, oh),
            DungeonSize::ExtraLarge => (ow * 2, oh),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DungeonSize::Small      => "Small",
            DungeonSize::Medium     => "Medium",
            DungeonSize::Large      => "Large",
            DungeonSize::ExtraLarge => "Extra Large",
        }
    }
}

// ─── Dungeon Entrance Reference (stored on the overworld Map) ─────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonEntranceRef {
    pub x: usize,
    pub y: usize,
    pub tier: u32,
    pub size: DungeonSize,
}

// ─── Dungeon NPC ──────────────────────────────────────────────────────────────

fn default_move_every() -> u32 { 2 }
fn default_aggression() -> f32  { 0.65 }
fn default_sight_range() -> usize { 8 }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DungeonNpc {
    pub x: usize,
    pub y: usize,
    pub enemy_name: String,
    pub health: i32,
    pub attack: i32,
    pub strength: i32,
    pub defense: i32,
    pub loot_table: String,
    pub bfs_depth: usize,
    pub dead: bool,
    #[serde(default)]
    pub move_counter: u32,
    #[serde(default = "default_move_every")]
    pub move_every: u32,
    #[serde(default = "default_aggression")]
    pub aggression: f32,
    #[serde(default = "default_sight_range")]
    pub sight_range: usize,
}

impl DungeonNpc {
    pub fn to_enemy(&self) -> Enemy {
        Enemy::new(&self.enemy_name, self.health, self.attack,
                   self.strength, self.defense, &self.loot_table)
    }

    /// Advance this NPC one game step. Returns `true` if the NPC stepped onto
    /// the player tile (combat trigger). The caller must handle combat and mark
    /// the NPC dead — this method does NOT modify the NPC on a combat trigger.
    pub fn tick(
        &mut self,
        tiles: &mut Vec<Vec<DungeonTile>>,
        player_x: usize,
        player_y: usize,
        width: usize,
        height: usize,
        occupied: &[(usize, usize)],
        rng: &mut impl Rng,
    ) -> bool {
        if self.dead { return false; }
        self.move_counter += 1;
        if self.move_counter < self.move_every { return false; }
        self.move_counter = 0;

        let dist = self.x.abs_diff(player_x) + self.y.abs_diff(player_y);
        let in_sight = dist <= self.sight_range;

        let (new_x, new_y) = if in_sight && rng.gen::<f32>() < self.aggression {
            dungeon_step_toward(self.x, self.y, player_x, player_y,
                                tiles, width, height, occupied, rng)
        } else {
            dungeon_random_step(self.x, self.y, tiles, width, height, occupied, rng)
        };

        if new_x == self.x && new_y == self.y { return false; }

        // Combat: NPC walked onto player tile — don't move, caller handles it
        if new_x == player_x && new_y == player_y { return true; }

        tiles[self.y][self.x] = DungeonTile::Floor;
        tiles[new_y][new_x] = DungeonTile::Enemy;
        self.x = new_x;
        self.y = new_y;
        false
    }
}

// ─── Monster Camp (kept for serialization compatibility) ──────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MonsterCamp {
    pub center_x: usize,
    pub center_y: usize,
    pub bfs_depth: usize,
    pub cleared: bool,
}

// ─── Floor Loot ───────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FloorLoot {
    pub x: usize,
    pub y: usize,
    pub items: HashMap<u32, u32>,
    pub collected: bool,
}

// ─── Dungeon Map ──────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DungeonMap {
    pub width: usize,
    pub height: usize,
    /// Tile grid — not stored in character.json; loaded from dungeon_<key>.txt
    #[serde(skip)]
    pub tiles: Vec<Vec<DungeonTile>>,
    pub spawn_x: usize,
    pub spawn_y: usize,
    pub exit_x: usize,
    pub exit_y: usize,
}

impl DungeonMap {
    pub fn render_viewport(&self, player_x: usize, player_y: usize,
                           h_radius: usize, v_radius: usize) -> String {
        let start_x = player_x.saturating_sub(h_radius);
        let end_x   = (player_x + h_radius).min(self.width.saturating_sub(1));
        let start_y = player_y.saturating_sub(v_radius);
        let end_y   = (player_y + v_radius).min(self.height.saturating_sub(1));

        let mut out = String::new();
        for y in start_y..=end_y {
            for x in start_x..=end_x {
                if x == player_x && y == player_y {
                    out.push_str("\x1B[93mP\x1B[0m ");
                } else if y < self.tiles.len() && x < self.tiles[y].len() {
                    out.push_str(self.tiles[y][x].render());
                    out.push(' ');
                } else {
                    out.push_str("\x1B[90m#\x1B[0m ");
                }
            }
            out.push_str("\r\n");
        }
        out
    }

    pub fn serialize_tiles(&self) -> String {
        let mut s = String::with_capacity(self.width * (self.height + 1));
        for row in &self.tiles {
            for tile in row { s.push(tile.to_char()); }
            s.push('\n');
        }
        s
    }

    pub fn load_tiles(&mut self, data: &str) {
        self.tiles = vec![vec![DungeonTile::Rock; self.width]; self.height];
        for (y, line) in data.lines().enumerate() {
            if y >= self.height { break; }
            for (x, ch) in line.chars().enumerate() {
                if x >= self.width { break; }
                self.tiles[y][x] = DungeonTile::from_char(ch);
            }
        }
    }
}

// ─── Dungeon Instance ─────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DungeonInstance {
    pub id_key: String,
    pub tier: u32,
    pub size: DungeonSize,
    pub map: DungeonMap,
    pub npcs: Vec<DungeonNpc>,
    pub camps: Vec<MonsterCamp>,
    pub floor_loot: Vec<FloorLoot>,
    pub explored: bool,
}

impl DungeonInstance {
    /// Advance all live NPCs one step. Returns `Some(index)` when an NPC walks
    /// onto the player tile — caller must fight it, mark it dead, and restore its
    /// tile. Only the first per-tick collision is returned.
    pub fn update_npcs(&mut self, player_x: usize, player_y: usize,
                       rng: &mut impl Rng) -> Option<usize> {
        let w = self.map.width;
        let h = self.map.height;
        let mut positions: Vec<(usize, usize)> =
            self.npcs.iter().map(|n| (n.x, n.y)).collect();

        let mut npcs = std::mem::take(&mut self.npcs);
        let mut combat_idx: Option<usize> = None;

        for i in 0..npcs.len() {
            if npcs[i].dead { continue; }
            let occupied: Vec<(usize, usize)> = positions.iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, &p)| p)
                .collect();

            let hit = npcs[i].tick(&mut self.map.tiles, player_x, player_y,
                                   w, h, &occupied, rng);
            positions[i] = (npcs[i].x, npcs[i].y);
            if hit && combat_idx.is_none() { combat_idx = Some(i); }
        }

        self.npcs = npcs;
        combat_idx
    }

    /// BFS first-step direction toward the nearest live dungeon enemy.
    /// Returns `None` when there are no live enemies or none are reachable.
    pub fn bfs_toward_nearest_enemy(&self, player_x: usize, player_y: usize)
        -> Option<crate::map::Direction>
    {
        use crate::map::Direction;
        let w = self.map.width;
        let h = self.map.height;

        let enemy_positions: Vec<(usize, usize)> = self.npcs.iter()
            .filter(|n| !n.dead)
            .map(|n| (n.x, n.y))
            .collect();
        if enemy_positions.is_empty() { return None; }

        let passable_or_enemy = |x: usize, y: usize| {
            matches!(self.map.tiles[y][x],
                DungeonTile::Floor | DungeonTile::Exit | DungeonTile::Enemy)
        };

        let mut visited = vec![vec![false; w]; h];
        let mut queue: VecDeque<(usize, usize, Direction)> = VecDeque::new();
        visited[player_y][player_x] = true;

        for &(dir, dx, dy) in &[
            (Direction::Up,    0isize, -1isize),
            (Direction::Down,  0,  1),
            (Direction::Left, -1,  0),
            (Direction::Right, 1,  0),
        ] {
            let nx = player_x as isize + dx;
            let ny = player_y as isize + dy;
            if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize { continue; }
            let (nx, ny) = (nx as usize, ny as usize);
            if visited[ny][nx] || !passable_or_enemy(nx, ny) { continue; }
            visited[ny][nx] = true;
            if enemy_positions.contains(&(nx, ny)) { return Some(dir); }
            queue.push_back((nx, ny, dir));
        }

        while let Some((cx, cy, first_dir)) = queue.pop_front() {
            for &(dx, dy) in &[(0isize,-1isize),(0,1),(-1,0),(1,0)] {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;
                if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize { continue; }
                let (nx, ny) = (nx as usize, ny as usize);
                if visited[ny][nx] || !passable_or_enemy(nx, ny) { continue; }
                visited[ny][nx] = true;
                if enemy_positions.contains(&(nx, ny)) { return Some(first_dir); }
                queue.push_back((nx, ny, first_dir));
            }
        }
        None
    }

    pub fn generate(
        id_key: &str,
        tier: u32,
        size: DungeonSize,
        overworld_w: usize,
        overworld_h: usize,
        rng: &mut impl Rng,
    ) -> DungeonInstance {
        let (w, h) = size.dimensions(overworld_w, overworld_h);
        let mut tiles = vec![vec![DungeonTile::Rock; w]; h];

        // Spawn at center
        let spawn_x = w / 2;
        let spawn_y = h / 2;
        tiles[spawn_y][spawn_x] = DungeonTile::Floor;

        // Multiple drunkard walks branching from spawn and existing floor tiles.
        // Each walk picks a random already-carved tile as its start, so arms
        // branch off arms rather than all radiating from one point.
        let num_walks = (7 + w / 18).min(18);
        let steps_per_walk = (w + h) * 3;
        let dirs: [(isize, isize); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        const ROOM_CHANCE: f64 = 0.05;

        // floor_pool accumulates newly carved tiles for walk-start selection
        let mut floor_pool: Vec<(usize, usize)> = vec![(spawn_x, spawn_y)];
        let mut room_centers: Vec<(usize, usize)> = Vec::new();

        for walk_i in 0..num_walks {
            let (mut wx, mut wy) = if walk_i == 0 || floor_pool.is_empty() {
                (spawn_x, spawn_y)
            } else {
                floor_pool[rng.gen_range(0..floor_pool.len())]
            };

            for _ in 0..steps_per_walk {
                let (dx, dy) = dirs[rng.gen_range(0..4)];
                let nx = wx as isize + dx;
                let ny = wy as isize + dy;
                if nx >= 1 && ny >= 1 && nx < w as isize - 1 && ny < h as isize - 1 {
                    wx = nx as usize;
                    wy = ny as usize;
                    if tiles[wy][wx] == DungeonTile::Rock {
                        tiles[wy][wx] = DungeonTile::Floor;
                        floor_pool.push((wx, wy));
                    }
                }
                if rng.gen_bool(ROOM_CHANCE) {
                    let radius = rng.gen_range(1usize..=3);
                    let r = radius as isize;
                    for dy2 in -r..=r {
                        for dx2 in -r..=r {
                            let rx = wx as isize + dx2;
                            let ry = wy as isize + dy2;
                            if rx > 0 && ry > 0 && rx < w as isize - 1 && ry < h as isize - 1 {
                                let (rx, ry) = (rx as usize, ry as usize);
                                if tiles[ry][rx] == DungeonTile::Rock {
                                    tiles[ry][rx] = DungeonTile::Floor;
                                    floor_pool.push((rx, ry));
                                }
                            }
                        }
                    }
                    room_centers.push((wx, wy));
                }
            }
        }

        // BFS depth from spawn
        let depth_map = bfs_depth_map(&tiles, spawn_x, spawn_y, w, h);
        let max_depth = depth_map.iter()
            .flat_map(|r| r.iter())
            .filter(|&&d| d != usize::MAX)
            .copied()
            .max()
            .unwrap_or(1)
            .max(1);

        // Place exit at the farthest reachable floor tile from spawn
        let (exit_x, exit_y) = floor_pool.iter()
            .filter(|&&(x, y)| {
                let d = depth_map[y][x];
                d != usize::MAX && d > 0
                    && !(x == spawn_x && y == spawn_y)
            })
            .max_by_key(|&&(x, y)| depth_map[y][x])
            .copied()
            .unwrap_or((spawn_x, (spawn_y + 1).min(h - 2)));
        tiles[exit_y][exit_x] = DungeonTile::Exit;

        // Enemy base stats
        let base_enemy = {
            let enemies = basic_enemies();
            let name = if tier <= 10 { "Cave Goblin" } else { "Orc" };
            enemies.into_iter().find(|e| e.name == name)
                .unwrap_or_else(|| Enemy::new("Cave Goblin", 18, 5, 6, 4, "dungeon_common"))
        };

        // Collect reachable floor tiles (excluding spawn and exit)
        let reachable: Vec<(usize, usize, usize)> = floor_pool.iter()
            .filter(|&&(x, y)| {
                let d = depth_map[y][x];
                d != usize::MAX && d >= 4
                    && tiles[y][x] == DungeonTile::Floor
            })
            .map(|&(x, y)| (x, y, depth_map[y][x]))
            .collect();

        let mut npcs: Vec<DungeonNpc> = Vec::new();
        let mut used: std::collections::HashSet<(usize, usize)> =
            std::collections::HashSet::new();

        // Small scattered clusters (2–4 enemies near each other)
        let num_clusters = (4 + w / 30).min(10);
        for _ in 0..num_clusters {
            if reachable.is_empty() { break; }
            let &(cx, cy, _) = &reachable[rng.gen_range(0..reachable.len())];
            let cluster_size = rng.gen_range(2..=4usize);
            let mut placed = 0;
            for dy in -4isize..=4 {
                if placed >= cluster_size { break; }
                for dx in -4isize..=4 {
                    if placed >= cluster_size { break; }
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize { continue; }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if tiles[ny][nx] != DungeonTile::Floor { continue; }
                    if used.contains(&(nx, ny)) { continue; }
                    let d = depth_map[ny][nx];
                    if d == usize::MAX { continue; }
                    let mut npc = scale_enemy_for_depth(&base_enemy, tier, d, max_depth);
                    npc.x = nx; npc.y = ny;
                    tiles[ny][nx] = DungeonTile::Enemy;
                    used.insert((nx, ny));
                    npcs.push(npc);
                    placed += 1;
                }
            }
        }

        // Deep encampments — stronger clusters in the far reaches
        let deep_threshold = max_depth * 3 / 5;
        let deep_tiles: Vec<(usize, usize, usize)> = reachable.iter()
            .filter(|&&(x, y, d)| d >= deep_threshold && !used.contains(&(x, y))
                    && tiles[y][x] == DungeonTile::Floor)
            .copied()
            .collect();

        let num_camps = (1 + deep_tiles.len() / 40).min(3);
        let mut camp_centers: Vec<(usize, usize)> = Vec::new();
        let mut camp_attempts = 0usize;

        for _ in 0..num_camps {
            if deep_tiles.is_empty() { break; }
            if camp_attempts > 20 { break; }
            camp_attempts += 1;

            let &(cx, cy, _) = &deep_tiles[rng.gen_range(0..deep_tiles.len())];
            // Keep camps spread out
            if camp_centers.iter().any(|&(px, py)| {
                px.abs_diff(cx) + py.abs_diff(cy) < 20
            }) { continue; }
            camp_centers.push((cx, cy));

            let camp_size = rng.gen_range(3..=6usize);
            let mut placed = 0;
            for dy in -5isize..=5 {
                if placed >= camp_size { break; }
                for dx in -5isize..=5 {
                    if placed >= camp_size { break; }
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize { continue; }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if tiles[ny][nx] != DungeonTile::Floor { continue; }
                    if used.contains(&(nx, ny)) { continue; }
                    let d = depth_map[ny][nx];
                    if d == usize::MAX { continue; }
                    let mut npc = scale_enemy_for_depth(&base_enemy, tier, d, max_depth);
                    // Camp enemies are a bit tougher
                    npc.health = (npc.health as f32 * 1.3) as i32;
                    npc.strength += 1;
                    npc.x = nx; npc.y = ny;
                    tiles[ny][nx] = DungeonTile::Enemy;
                    used.insert((nx, ny));
                    npcs.push(npc);
                    placed += 1;
                }
            }
        }

        let camps: Vec<MonsterCamp> = camp_centers.iter()
            .map(|&(x, y)| MonsterCamp {
                center_x: x,
                center_y: y,
                bfs_depth: depth_map[y][x],
                cleared: false,
            })
            .collect();

        let dmap = DungeonMap {
            width: w, height: h,
            tiles,
            spawn_x, spawn_y,
            exit_x, exit_y,
        };

        DungeonInstance {
            id_key: id_key.to_string(),
            tier,
            size,
            map: dmap,
            npcs,
            camps,
            floor_loot: Vec::new(),
            explored: false,
        }
    }

    pub fn reset(&mut self, rng: &mut impl Rng) {
        let new = DungeonInstance::generate(
            &self.id_key, self.tier, self.size.clone(), 300, 300, rng,
        );
        *self = new;
    }
}

// ─── Dungeon State ────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct DungeonState {
    pub instances: HashMap<String, DungeonInstance>,
    pub return_x: Option<usize>,
    pub return_y: Option<usize>,
    pub active_dungeon: Option<String>,
    pub dungeon_player_x: usize,
    pub dungeon_player_y: usize,
    /// Step counter for NPC tick pacing inside the dungeon.
    #[serde(default)]
    pub dungeon_move_count: u64,
}

impl DungeonState {
    pub fn dungeon_key(x: usize, y: usize) -> String {
        format!("{},{}", x, y)
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn bfs_depth_map(
    tiles: &[Vec<DungeonTile>],
    start_x: usize,
    start_y: usize,
    w: usize,
    h: usize,
) -> Vec<Vec<usize>> {
    let mut depth = vec![vec![usize::MAX; w]; h];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    depth[start_y][start_x] = 0;
    queue.push_back((start_x, start_y));
    while let Some((x, y)) = queue.pop_front() {
        let d = depth[y][x];
        for (dx, dy) in [(0isize,-1),(0,1),(-1,0),(1,0)] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0 && ny >= 0 {
                let (nx, ny) = (nx as usize, ny as usize);
                if nx < w && ny < h && depth[ny][nx] == usize::MAX {
                    let t = tiles[ny][nx];
                    if t == DungeonTile::Floor || t == DungeonTile::Exit
                        || t == DungeonTile::Enemy
                    {
                        depth[ny][nx] = d + 1;
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }
    depth
}

fn scale_enemy_for_depth(base: &Enemy, tier: u32, bfs_depth: usize, max_depth: usize) -> DungeonNpc {
    let factor = bfs_depth as f32 / max_depth as f32;
    let bonus = (tier as f32 * factor) as i32;
    DungeonNpc {
        x: 0, y: 0,
        enemy_name: base.name.clone(),
        health:   (base.health   + bonus * 2).max(1),
        attack:   (base.attack   + bonus / 3).max(1),
        strength: (base.strength + bonus / 2).max(1),
        defense:  (base.defense  + bonus / 4).max(0),
        loot_table: "dungeon_common".to_string(),
        bfs_depth,
        dead: false,
        move_counter: 0,
        move_every: 2,
        aggression: 0.65,
        sight_range: 8,
    }
}

fn dungeon_is_passable(
    x: usize, y: usize,
    tiles: &[Vec<DungeonTile>],
    occupied: &[(usize, usize)],
) -> bool {
    matches!(tiles[y][x], DungeonTile::Floor | DungeonTile::Exit)
        && !occupied.contains(&(x, y))
}

fn dungeon_step_toward(
    from_x: usize, from_y: usize,
    to_x: usize, to_y: usize,
    tiles: &[Vec<DungeonTile>],
    width: usize, height: usize,
    occupied: &[(usize, usize)],
    rng: &mut impl Rng,
) -> (usize, usize) {
    let dx = to_x as isize - from_x as isize;
    let dy = to_y as isize - from_y as isize;

    let h_step = if dx > 0 { (from_x + 1, from_y) } else { (from_x.wrapping_sub(1), from_y) };
    let v_step = if dy > 0 { (from_x, from_y + 1) } else { (from_x, from_y.wrapping_sub(1)) };

    let prefer_h = dx.abs() > dy.abs() || (dx.abs() == dy.abs() && rng.gen_bool(0.5));

    let mut candidates: Vec<(usize, usize)> = Vec::new();
    if prefer_h {
        if dx != 0 { candidates.push(h_step); }
        if dy != 0 { candidates.push(v_step); }
    } else {
        if dy != 0 { candidates.push(v_step); }
        if dx != 0 { candidates.push(h_step); }
    }

    for (nx, ny) in candidates {
        if nx >= width || ny >= height { continue; }
        if nx == to_x && ny == to_y { return (nx, ny); } // player tile → combat
        if dungeon_is_passable(nx, ny, tiles, occupied) { return (nx, ny); }
    }
    (from_x, from_y)
}

fn dungeon_random_step(
    from_x: usize, from_y: usize,
    tiles: &[Vec<DungeonTile>],
    width: usize, height: usize,
    occupied: &[(usize, usize)],
    rng: &mut impl Rng,
) -> (usize, usize) {
    let mut dirs: [(isize, isize); 4] = [(0,-1),(0,1),(-1,0),(1,0)];
    for i in (1..4usize).rev() {
        let j = rng.gen_range(0..=i);
        dirs.swap(i, j);
    }
    for (ddx, ddy) in dirs {
        let nx = from_x as isize + ddx;
        let ny = from_y as isize + ddy;
        if nx < 0 || ny < 0 { continue; }
        let (nx, ny) = (nx as usize, ny as usize);
        if nx < width && ny < height && dungeon_is_passable(nx, ny, tiles, occupied) {
            return (nx, ny);
        }
    }
    (from_x, from_y)
}
