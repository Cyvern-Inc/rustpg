use std::fmt;
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Player,
    Tree,
    Rock,
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub player_x: usize,
    pub player_y: usize,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
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

        Map {
            width,
            height,
            tiles,
            player_x: chunk_center_x,
            player_y: chunk_center_y,
        }
    }

    // Render the current 30x30 chunk that the player is in
    pub fn render(&self) -> String {
        let chunk_size = 30;
        let chunk_x = (self.player_x / chunk_size) * chunk_size;
        let chunk_y = (self.player_y / chunk_size) * chunk_size;

        let start_x = chunk_x;
        let start_y = chunk_y;
        let end_x = (start_x + chunk_size).min(self.width);
        let end_y = (start_y + chunk_size).min(self.height);

        let mut rendered_view = String::new();
        for y in start_y..end_y {
            for x in start_x..end_x {
                match self.tiles[y][x] {
                    Tile::Empty => rendered_view.push('.'),
                    Tile::Player => rendered_view.push('P'),
                    Tile::Tree => rendered_view.push('t'),
                    Tile::Rock => rendered_view.push('r'),
                }
                rendered_view.push(' '); // Add space between tiles for better readability
            }
            rendered_view.push('\n');
        }
        rendered_view
    }

    // Move the player in the specified direction
    pub fn move_player(&mut self, direction: Direction) {
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
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
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
