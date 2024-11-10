// Removed the unused random_range function from utils.rs.

use crate::map::{Direction, Map};
use crate::player::Player;
use rand::Rng;
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;

pub fn faf(player: &mut Player, game_map: &mut Map) {
    // Initialize weights (adjust as needed)
    let weights = MovementWeights {
        same_direction: 128,
        up: 64,
        down: 64,
        left: 64,
        right: 64,
        towards_tree: 0,
        towards_rock: 0,
        towards_campfire: 0,
        away_from_campfire: 0,
    };
    let mut prev_direction = Direction::Up;
    let mut rng = rand::thread_rng();

    loop {
        // Non-blocking input to check for 'p', 'b', or 'q'
        if let Some(input) = check_for_input() {
            match input.as_str() {
                "p" => {
                    println!("Paused. Press Enter to continue...");
                    let _ = io::stdin().read_line(&mut String::new());
                }
                "b" | "q" => {
                    break; // Exit faf mode
                }
                _ => {}
            }
        }

        // Adjust view radius based on terminal size
        if let Some((width, height)) = term_size::dimensions() {
            game_map.view_radius = std::cmp::min(width, height) / 2;
        }

        // Clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();

        // Render the viewport centered on the player
        println!("{}", game_map.render_full());

        // Move player in weighted random direction
        let direction = get_weighted_random_direction(&weights, prev_direction, player, game_map);
        game_map.move_player(&direction);
        prev_direction = direction;

        // Check for enemy encounter
        if rng.gen_range(0..100) < 20 {
            // Exit faf mode to start combat
            break;
        }

        // Wait for 0.2 seconds
        thread::sleep(Duration::from_millis(200));
    }
}

// Helper struct for movement weights
struct MovementWeights {
    same_direction: u8,
    up: u8,
    down: u8,
    left: u8,
    right: u8,
    towards_tree: u8,
    towards_rock: u8,
    towards_campfire: u8,
    away_from_campfire: u8,
}

// Function to get weighted random direction
fn get_weighted_random_direction(
    weights: &MovementWeights,
    prev_direction: Direction,
    player: &Player,
    game_map: &Map,
) -> Direction {
    let mut directions = vec![
        (Direction::Up, weights.up),
        (Direction::Down, weights.down),
        (Direction::Left, weights.left),
        (Direction::Right, weights.right),
    ];

    // Add weight for same direction
    directions.iter_mut().for_each(|(dir, weight)| {
        if *dir == prev_direction {
            *weight += weights.same_direction;
        }
    });

    // Calculate total weight
    let total_weight: u16 = directions.iter().map(|&(_, w)| w as u16).sum();

    // Choose random direction based on weights
    let mut rng = rand::thread_rng();
    let mut choice = rng.gen_range(0..total_weight);
    for (dir, weight) in directions {
        if choice < weight as u16 {
            return dir;
        }
        choice -= weight as u16;
    }

    // Default to up if something goes wrong
    Direction::Up
}

// Implement a function to handle non-blocking input
fn check_for_input() -> Option<String> {
    // Use a crate like `crossterm` or `termion` to handle non-blocking input
    None // Placeholder implementation
}
