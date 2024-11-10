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

        for row in &self.tiles {
            let mut current_char = row[0].to_char();
            let mut count = 0;

            for &tile in row {
                if tile.to_char() == current_char {
                    count += 1;
                } else {
                    serialized.push_str(&format!("{}{}", current_char, count));
                    current_char = tile.to_char();
                    count = 1;
                }
            }
            serialized.push_str(&format!("{}{}", current_char, count));
            serialized.push('\n');
        }
        serialized
    }

    pub fn deserialize_map(width: usize, height: usize, data: &str) -> Self {
        let mut tiles = vec![vec![Tile::Empty; width]; height];
        let mut y = 0;

        for line in data.lines() {
            let mut x = 0;
            let mut chars = line.chars().peekable();

            while let Some(c) = chars.next() {
                if let Some(digit_char) = chars.peek() {
                    if digit_char.is_numeric() {
                        let mut count_str = String::new();
                        while let Some(digit) = chars.peek() {
                            if digit.is_numeric() {
                                count_str.push(*digit);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let count: usize = count_str.parse().unwrap_or(1);
                        for _ in 0..count {
                            if x < width && y < height {
                                tiles[y][x] = Tile::from_char(c);
                                x += 1;
                            }
                        }
                    }
                }
            }
            y += 1;
        }

        Self {
            width,
            height,
            tiles,
            player_x: 0, // Default value (set properly after loading if needed)
            player_y: 0, // Default value (set properly after loading if needed)
            view_radius: 15, // Default value
            campfire_x: 0, // Default value (set properly after loading if needed)
            campfire_y: 0, // Default value (set properly after loading if needed)
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
        if x < self.width && y < self.height {
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
