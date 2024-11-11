// Import necessary modules and types
use rand::Rng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent};

// Ensure you have access to these structs and enums
use crate::map::{Map, Direction};
use crate::player::Player;

pub struct MovementWeights {
    pub same_direction: u32,
    pub away_from_campfire: u32,
    pub towards_campfire: u32,
    pub towards_tree: u32,
    pub towards_rock: u32,
    pub away_from_tree: u32,
    pub away_from_rock: u32,
    pub up: u32,
    pub down: u32,
    pub left: u32,
    pub right: u32,
}

pub fn faf(player: &mut Player, game_map: &mut Map) -> bool {
    // Initialize weights (adjust as needed)
    let weights = MovementWeights {
        same_direction: 128,
        up: 64,
        down: 64,
        left: 64,
        right: 64,
        towards_tree: 0,
        towards_rock: 0,
        away_from_tree: 0,
        away_from_rock: 0,
        towards_campfire: 0,
        away_from_campfire: 0,
    };
    let mut prev_direction = Direction::Up;
    let mut rng = rand::thread_rng();

    loop {
        // Clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
        
        // Render the viewport
        println!("{}", game_map.render_full());

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

        // Calculate the next direction using movement weights
        let direction = weighted_random_direction(&mut rng, &weights, prev_direction, game_map);

        // Move the player
        game_map.move_player(&direction);
        prev_direction = direction;

        // Check for enemy encounter
        if should_encounter_enemy(1) { // Adjust the encounter rate as needed
            println!("Enemy encountered! Stopping automatic movement.");
            player.in_combat = true;
            return true; // Indicate that combat should be initiated
        }

        // Delay to simulate time between movements
        thread::sleep(Duration::from_millis(500));

        // Optionally, update the view or display the map here
    }

    false // No enemy encountered
}

pub fn check_for_input() -> Option<String> {
    if event::poll(Duration::from_secs(0)).unwrap() {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
            if modifiers.is_empty() {
                match code {
                    KeyCode::Char('p') => return Some("p".to_string()),
                    KeyCode::Char('b') => return Some("b".to_string()),
                    KeyCode::Char('q') => return Some("q".to_string()),
                    _ => {}
                }
            }
        }
    }
    None
}

pub fn weighted_random_direction(
    rng: &mut impl Rng,
    weights: &MovementWeights,
    prev_direction: Direction,
    game_map: &Map,
) -> Direction {
    // Collect all possible directions with their associated weights
    let mut directions = vec![
        (Direction::Up, weights.up),
        (Direction::Down, weights.down),
        (Direction::Left, weights.left),
        (Direction::Right, weights.right),
    ];

    // Increase weight for the same direction
    for (dir, weight) in &mut directions {
        if *dir == prev_direction {
            *weight += weights.same_direction;
        }
    }

    // Calculate the total weight
    let total_weight: u32 = directions.iter().map(|(_, weight)| *weight).sum();

    // Generate a random number within the total weight
    let mut choice = rng.gen_range(0..total_weight);

    // Select the direction based on weighted probability
    for (dir, weight) in directions {
        if choice < weight {
            return dir;
        }
        choice -= weight;
    }

    // Default to previous direction if something goes wrong
    prev_direction
}

pub fn should_encounter_enemy(chance: u8) -> bool {
    let mut rng = rand::thread_rng();
    let encounter = rng.gen_range(0..100) < chance;
    encounter
}
