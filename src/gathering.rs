use std::collections::{HashMap, VecDeque};

use crate::items::ToolTag;
use crate::map::{Direction, Map, Tile};
use crate::player::Player;
use crate::utils::MovementWeights;

// ---------------------------------------------------------------------------
// BFS pathfinding helpers (used by FAF auto-combat movement)
// ---------------------------------------------------------------------------

/// BFS from the player's position toward `(target_x, target_y)`, bounded to
/// the map's view_radius. Returns the first step direction along the shortest
/// path, or `None` if the target is unreachable within the search area.
///
/// The goal is considered reached when BFS arrives at any cardinal neighbor of
/// the target that is passable — matching how move_player handles walkability.
pub fn bfs_next_step(map: &Map, target_x: usize, target_y: usize) -> Option<Direction> {
    let start = (map.player_x, map.player_y);
    if start == (target_x, target_y) {
        return None;
    }

    let r = map.view_radius as isize;
    let px = map.player_x as isize;
    let py = map.player_y as isize;

    // Cells passable to walk through (same contract as move_player)
    let is_passable = |x: usize, y: usize| {
        matches!(map.tiles[y][x], Tile::Empty | Tile::Campfire | Tile::Stump | Tile::DungeonEntrance)
    };

    // Goal: a cardinal neighbor of the target that we can actually step on
    let goal_cells: Vec<(usize, usize)> = [
        (target_x as isize, target_y as isize - 1),
        (target_x as isize, target_y as isize + 1),
        (target_x as isize - 1, target_y as isize),
        (target_x as isize + 1, target_y as isize),
    ]
    .iter()
    .filter(|&&(x, y)| {
        x >= 0 && y >= 0
            && x < map.width as isize
            && y < map.height as isize
            && is_passable(x as usize, y as usize)
    })
    .map(|&(x, y)| (x as usize, y as usize))
    .collect();

    if goal_cells.is_empty() {
        return None; // target is completely surrounded by impassable tiles
    }

    // Standard BFS — track which direction was taken from the start tile
    let mut visited = vec![vec![false; map.width]; map.height];
    // Queue entries: (x, y, first_direction_from_start)
    let mut queue: VecDeque<(usize, usize, Direction)> = VecDeque::new();

    visited[start.1][start.0] = true;

    for &(dir, dx, dy) in &[
        (Direction::Up,    0isize, -1isize),
        (Direction::Down,  0,       1),
        (Direction::Left,  -1,      0),
        (Direction::Right,  1,      0),
    ] {
        let nx = start.0 as isize + dx;
        let ny = start.1 as isize + dy;
        if nx < 0 || ny < 0 || nx >= map.width as isize || ny >= map.height as isize {
            continue;
        }
        let (nx, ny) = (nx as usize, ny as usize);
        // Bound search to view radius
        if (nx as isize - px).abs() > r || (ny as isize - py).abs() > r {
            continue;
        }
        if !visited[ny][nx] && is_passable(nx, ny) {
            visited[ny][nx] = true;
            if goal_cells.contains(&(nx, ny)) {
                return Some(dir);
            }
            queue.push_back((nx, ny, dir));
        }
    }

    while let Some((cx, cy, first_dir)) = queue.pop_front() {
        for &(dx, dy) in &[(0isize, -1isize), (0, 1), (-1, 0), (1, 0)] {
            let nx = cx as isize + dx;
            let ny = cy as isize + dy;
            if nx < 0 || ny < 0 || nx >= map.width as isize || ny >= map.height as isize {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if (nx as isize - px).abs() > r || (ny as isize - py).abs() > r {
                continue;
            }
            if !visited[ny][nx] && is_passable(nx, ny) {
                visited[ny][nx] = true;
                if goal_cells.contains(&(nx, ny)) {
                    return Some(first_dir);
                }
                queue.push_back((nx, ny, first_dir));
            }
        }
    }

    None // no path found within view radius
}

/// Collect all visible tiles of `target` sorted by Manhattan distance, then
/// return the first-step Direction toward the nearest reachable one via BFS.
/// Returns `None` if every visible target is blocked.
pub fn bfs_toward_nearest_reachable(map: &Map, target: Tile) -> Option<Direction> {
    let px = map.player_x as isize;
    let py = map.player_y as isize;
    let r = map.view_radius as isize;

    let x_start = (px - r).max(0) as usize;
    let x_end = (px + r).min(map.width as isize - 1) as usize;
    let y_start = (py - r).max(0) as usize;
    let y_end = (py + r).min(map.height as isize - 1) as usize;

    let mut candidates: Vec<(usize, usize, isize)> = Vec::new();
    for y in y_start..=y_end {
        for x in x_start..=x_end {
            if map.tiles[y][x] == target {
                let dist = (x as isize - px).abs() + (y as isize - py).abs();
                candidates.push((x, y, dist));
            }
        }
    }
    candidates.sort_by_key(|&(_, _, d)| d);

    for (tx, ty, _) in candidates {
        if let Some(dir) = bfs_next_step(map, tx, ty) {
            return Some(dir);
        }
    }
    None
}

/// Outcome of a single gathering attempt.
pub enum GatherResult {
    /// Resource successfully harvested.
    Success {
        item_id: u32,
        quantity: u32,
        xp: f64,
        /// True if the resource tile was depleted this attempt.
        resource_depleted: bool,
    },
    /// The attempt roll failed — no yield, no depletion.
    Failed,
    /// Player lacks the required tool in inventory or equipped slot.
    NoTool,
    /// No valid resource tile within view range.
    NothingNearby,
    /// Player manually interrupted the session.
    Interrupted,
    /// A random enemy encounter fired mid-session.
    EnemyEncounter,
}

/// Returns true if the player has at least one item with the given ToolTag,
/// either equipped as a weapon or present in the inventory.
pub fn has_valid_tool(player: &Player, tag: &ToolTag) -> bool {
    // Check equipped weapon
    if let Some(weapon) = &player.equipped_weapon {
        if weapon.tool_tag.as_ref() == Some(tag) {
            return true;
        }
    }
    // Check inventory — any item with a matching tag and quantity > 0
    let items = crate::items::get_items();
    for (&item_id, &qty) in &player.inventory {
        if qty == 0 {
            continue;
        }
        if let Some(item) = items.get(&item_id) {
            if item.tool_tag.as_ref() == Some(tag) {
                return true;
            }
        }
    }
    false
}

/// Find the nearest tile of `target_type` within the map's view radius,
/// searching outward from the player position using Manhattan distance.
/// Returns `Some((x, y))` or `None` if no such tile is visible.
pub fn find_nearest_tile(map: &Map, target: Tile) -> Option<(usize, usize)> {
    let px = map.player_x as isize;
    let py = map.player_y as isize;
    let r = map.view_radius as isize;

    let x_start = (px - r).max(0) as usize;
    let x_end = (px + r).min(map.width as isize - 1) as usize;
    let y_start = (py - r).max(0) as usize;
    let y_end = (py + r).min(map.height as isize - 1) as usize;

    let mut best: Option<(usize, usize, isize)> = None; // (x, y, manhattan_dist)

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            if map.tiles[y][x] == target {
                let dist = (x as isize - px).abs() + (y as isize - py).abs();
                if best.is_none() || dist < best.unwrap().2 {
                    best = Some((x, y, dist));
                }
            }
        }
    }

    best.map(|(x, y, _)| (x, y))
}

/// Check the four cardinal tiles adjacent to the player for a specific tile type.
/// Returns the coordinates of the first match found, or None.
pub fn find_adjacent_tile(map: &Map, target: Tile) -> Option<(usize, usize)> {
    let (px, py) = (map.player_x, map.player_y);
    let candidates = [
        (px, py.wrapping_sub(1)),
        (px, py + 1),
        (px.wrapping_sub(1), py),
        (px + 1, py),
    ];
    for (x, y) in candidates {
        if x < map.width && y < map.height && map.tiles[y][x] == target {
            return Some((x, y));
        }
    }
    None
}

/// Build movement weights biased toward a target tile at (tx, ty) from (px, py).
pub fn weights_toward(tx: usize, ty: usize, px: usize, py: usize) -> MovementWeights {
    let dx = tx as i32 - px as i32;
    let dy = ty as i32 - py as i32;
    MovementWeights {
        same_direction: 32,
        up:    if dy < 0 { 220 } else { 20 },
        down:  if dy > 0 { 220 } else { 20 },
        left:  if dx < 0 { 220 } else { 20 },
        right: if dx > 0 { 220 } else { 20 },
    }
}
