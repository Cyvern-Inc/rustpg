use std::fmt;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::player::Player;
use term_size; // Ensure you have the term_size crate in your Cargo.toml

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Tile {
    Empty,
    Player,
    Tree,
    Rock,
    Campfire,
}

impl Tile {
    pub fn render(&self) -> &str {
        match self {
            Tile::Empty => ".",
            Tile::Player => "P",
            Tile::Tree => "t",
            Tile::Rock => "r",
            Tile::Campfire => "#",
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Tile::Empty => '.',
            Tile::Player => 'P',
            Tile::Tree => 't',
            Tile::Rock => 'r',
            Tile::Campfire => '#',
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Empty,
            'P' => Tile::Player,
            't' => Tile::Tree,
            'r' => Tile::Rock,
            '#' => Tile::Campfire,
            _ => Tile::Empty, // Default to Empty for unknown chars
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
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = vec![vec![Tile::Empty; width]; height];

        // Randomly generate map with obstacles (tree and rock)
        let mut rng = rand::thread_rng();
        for y in 0..height {
            for x in 0..width {
                let roll: f32 = rng.gen();
                if roll < 0.05 {
                    tiles[y][x] = Tile::Rock;
                } else if roll < 0.15 {
                    tiles[y][x] = Tile::Tree;
                }
            }
        }

        // Place the player in the center of the initial chunk
        let chunk_size = 30;
        let chunk_center_x = chunk_size / 2;
        let chunk_center_y = chunk_size / 2;
        tiles[chunk_center_y][chunk_center_x] = Tile::Player;

        // Place campfire south of player
        let campfire_x = chunk_center_x;
        let campfire_y = chunk_center_y + 1;
        if campfire_y < height {
            tiles[campfire_y][campfire_x] = Tile::Campfire;
        }

        Self {
            width,
            height,
            tiles,
            player_x: chunk_center_x,
            player_y: chunk_center_y,
            view_radius: 15, // Default to 15 for a 30x30 view
            campfire_x,
            campfire_y,
        }
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

        if self.tiles[new_y][new_x] == Tile::Empty || self.tiles[new_y][new_x] == Tile::Campfire {
            self.tiles[self.player_y][self.player_x] = Tile::Empty;
            self.player_x = new_x;
            self.player_y = new_y;
            self.tiles[self.player_y][self.player_x] = Tile::Player;
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Define the boundaries of the viewport
        let half_radius = self.view_radius;
        let start_x = if self.player_x > half_radius {
            self.player_x - half_radius
        } else {
            0
        };
        let end_x = usize::min(self.player_x + half_radius, self.width - 1);
        let start_y = if self.player_y > half_radius {
            self.player_y - half_radius
        } else {
            0
        };
        let end_y = usize::min(self.player_y + half_radius, self.height - 1);

        // Iterate through the viewport area
        for y in start_y..=end_y {
            for x in start_x..=end_x {
                // Append the tile representation followed by a space for even spacing
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

    pub fn deserialize_map(width: usize, height: usize, data: &str) -> Self {
        let mut tiles = vec![vec![Tile::Empty; width]; height];
        // Parse 'data' to populate 'tiles'
        // Example parsing logic (adjust as needed)
        for (y, line) in data.lines().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if y < height && x < width {
                    tiles[y][x] = match ch {
                        'p' => Tile::Player,
                        '#' => Tile::Campfire,
                        't' => Tile::Tree,
                        'r' => Tile::Rock,
                        _ => Tile::Empty,
                    };
                }
            }
        }
        // Extract player and campfire positions
        let mut player_x = 0;
        let mut player_y = 0;
        let mut campfire_x = 0;
        let mut campfire_y = 0;
        
        for y in 0..height {
            for x in 0..width {
                match tiles[y][x] {
                    Tile::Player => {
                        player_x = x;
                        player_y = y;
                    }
                    Tile::Campfire => {
                        campfire_x = x;
                        campfire_y = y;
                    }
                    _ => {}
                }
            }
        }

        Self {
            width,
            height,
            tiles,
            player_x,
            player_y,
            campfire_x,
            campfire_y,
            view_radius: 15, // Example value
        }
    }

    pub fn interact(&self, player: &Player) -> Option<String> {
        let (x, y) = match player.facing {
            Direction::Up => (self.player_x, self.player_y.saturating_sub(1)),
            Direction::Down => (self.player_x, (self.player_y + 1).min(self.height - 1)),
            Direction::Left => (self.player_x.saturating_sub(1), self.player_y),
            Direction::Right => ((self.player_x + 1).min(self.width - 1), self.player_y),
        };
        
        if self.tiles[y][x] == Tile::Campfire {
            Some("Would you like to save the game? (y/n)".to_string())
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if y < self.height && x < self.width {
            self.tiles[y][x] = tile;
        }
    }

    pub fn is_adjacent_and_facing_campfire(&self, player: &Player) -> bool {
        let (player_x, player_y) = (self.player_x, self.player_y);
        let (adjacent_x, adjacent_y) = match player.facing {
            Direction::Up => (self.player_x, self.player_y.saturating_sub(1)),
            Direction::Down => (self.player_x, self.player_y + 1),
            Direction::Left => (self.player_x.saturating_sub(1), self.player_y),
            Direction::Right => (self.player_x + 1, self.player_y),
        };

        if adjacent_x < self.width && adjacent_y < self.height {
            if self.tiles[adjacent_y][adjacent_x] == Tile::Campfire {
                return true;
            }
        }
        false
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
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Tile::Empty => '.',
            Tile::Player => 'P',
            Tile::Tree => 't',
            Tile::Rock => 'r',
            Tile::Campfire => '#',
        };
        write!(f, "{}", symbol)
    }
}
