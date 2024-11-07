use std::fmt;
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Tile {
    Empty,
    Player,
    Tree,
    Rock,
}

impl Tile {
    pub fn render(&self) -> &str {
        match self {
            Tile::Empty => ".",
            Tile::Player => "P",
            Tile::Tree => "t",
            Tile::Rock => "r",
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Tile::Empty => '.',
            Tile::Player => 'P',
            Tile::Tree => 't',
            Tile::Rock => 'r',
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            '.' => Tile::Empty,
            'P' => Tile::Player,
            't' => Tile::Tree,
            'r' => Tile::Rock,
            _ => Tile::Empty, // Default to Empty for unknown chars
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub player_x: usize,
    pub player_y: usize,
    pub view_radius: usize,
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

        Self {
            width,
            height,
            tiles,
            player_x: chunk_center_x,
            player_y: chunk_center_y,
            view_radius: 15, // Default to 15 for a 30x30 view
        }
    }

    pub fn move_player(&mut self, direction: &Direction) {
        let (new_x, new_y) = match direction {
            Direction::Up => (self.player_x, self.player_y.saturating_sub(1)),
            Direction::Down => (self.player_x, (self.player_y + 1).min(self.height - 1)),
            Direction::Left => (self.player_x.saturating_sub(1), self.player_y),
            Direction::Right => ((self.player_x + 1).min(self.width - 1), self.player_y),
        };

        // Check if the new position is a valid location (i.e., not an obstacle)
        if self.tiles[new_y][new_x] == Tile::Empty {
            // Update the player's position
            self.tiles[self.player_y][self.player_x] = Tile::Empty;
            self.player_x = new_x;
            self.player_y = new_y;
            self.tiles[self.player_y][self.player_x] = Tile::Player;
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let chunk_size = if self.view_radius == 15 { 30 } else if self.view_radius == 10 { 20 } else { 10 };
        let chunk_x = (self.player_x / chunk_size) * chunk_size;
        let chunk_y = (self.player_y / chunk_size) * chunk_size;

        let start_x = chunk_x;
        let start_y = chunk_y;
        let end_x = (start_x + chunk_size).min(self.width);
        let end_y = (start_y + chunk_size).min(self.height);

        for y in start_y..end_y {
            for x in start_x..end_x {
                match self.tiles[y][x] {
                    Tile::Player => output.push_str("\x1b[31mP\x1b[0m "), // Red player character 'P'
                    _ => output.push_str(&format!("{} ", self.tiles[y][x].render())),
                }
            }
            output.push('\n');
        }
        output
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
        };
        write!(f, "{}", symbol)
    }
}
