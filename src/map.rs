#[derive(Debug)]
pub struct Map {
    grid: Vec<Vec<char>>,
    player_position: (usize, usize),
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Map {
    pub fn new(rows: usize, cols: usize) -> Map {
        let mut grid = vec![vec!['.'; cols]; rows];
        let player_position = (0, 0);
        grid[player_position.0][player_position.1] = 'P';
        Map { grid, player_position }
    }

    pub fn move_player(&mut self, direction: Direction) {
        let (x, y) = self.player_position;
        self.grid[x][y] = '.'; // Clear current position

        match direction {
            Direction::Up if x > 0 => self.player_position.0 -= 1,
            Direction::Down if x < self.grid.len() - 1 => self.player_position.0 += 1,
            Direction::Left if y > 0 => self.player_position.1 -= 1,
            Direction::Right if y < self.grid[0].len() - 1 => self.player_position.1 += 1,
            _ => println!("You can't move in that direction!"),
        }

        let (new_x, new_y) = self.player_position;
        self.grid[new_x][new_y] = 'P'; // Update player position
    }

    pub fn print_map(&self) {
        for row in &self.grid {
            for tile in row {
                print!("{}", tile);
            }
            println!();
        }
    }

    pub fn get_player_position(&self) -> (usize, usize) {
        self.player_position
    }

    pub fn player_on_specific_tile(&self, position: (usize, usize)) -> bool {
        self.player_position == position
    }
}
