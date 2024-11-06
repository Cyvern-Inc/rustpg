mod player;
mod enemy;
mod map;
mod quest;
mod utils;
mod skill;

use player::Player;
use map::{Map, Direction};
use quest::Quest;
use enemy::Enemy;
use utils::random_range;
use std::io::{self, Write};
use std::collections::VecDeque;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{ClearType, Clear, EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    Result,
};
use std::panic;

fn main() -> Result<()> {
    let mut stdout = io::stdout();

    // Enter alternate screen and enable raw mode
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    // Set a panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let mut stdout = io::stdout();
        let _ = disable_raw_mode();
        let _ = execute!(stdout, LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Initialize game state
    let mut player = Player::new();
    let mut map = Map::new(10, 30); // 10x30 map
    let mut quests = vec![Quest::new(
        "Find the Lost Sword",
        "Retrieve the lost sword from the goblin camp.",
    )];
    let mut recent_actions: VecDeque<String> = VecDeque::new(); // Recent actions
    let mut command_history: VecDeque<String> = VecDeque::new(); // Command history
    let mut command_buffer = String::new();

    // Main game loop
    loop {
        // **Clear the Screen and Reset Cursor Position**
        execute!(
            stdout,
            Clear(ClearType::All),
                 MoveTo(0, 0)
        )?;

        // **Print Static Content Inside the Loop**
        execute!(
            stdout,
            Print("Welcome to the Rust CLI RPG!\n"),
                 Print(format!(
                     "Quest: {} - {}\n\n",
                     quests[0].name, quests[0].description
                 )),
                 Print(format!("Your current position: {:?}\n", map.get_player_position())),
        )?;

        // **Print Map**
        map.print_map(&mut stdout)?; // Now correctly accepts &mut stdout

        // **Display Recent Actions**
        execute!(stdout, Print("\nRecent Actions:\n"))?;
        for action in recent_actions.iter().rev() {
            execute!(stdout, Print(format!("{}\n", action)))?;
        }
        execute!(stdout, Print("\n"))?;

        // **Display Action Menu**
        execute!(
            stdout,
            Print("(w/a/s/d) Move | (status) Player Status | (train) Train a Skill\n"),
                 Print("(i) Inventory | (m) Menu | (q) Quit | (history) View Recent Commands\n"),
                 Print("> ")
        )?;
        stdout.flush()?;

        // **Prepare for User Input**
        command_buffer.clear();
        let mut history_index: Option<usize> = None;

        // **Handle User Input with Real-Time Key Detection**
        loop {
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Enter => {
                        execute!(stdout, Print("\n"))?;
                        if !command_buffer.trim().is_empty() {
                            command_history.push_back(command_buffer.clone());
                            if command_history.len() > 10 {
                                command_history.pop_front();
                            }
                        }
                        break; // Exit input loop
                    }
                    KeyCode::Up => {
                        if let Some(index) = history_index {
                            if index > 0 {
                                history_index = Some(index - 1);
                            }
                        } else if !command_history.is_empty() {
                            history_index = Some(command_history.len() - 1);
                        }

                        if let Some(index) = history_index {
                            command_buffer = command_history[index].clone();
                            // Clear current input line
                            execute!(
                                stdout,
                                MoveTo(0, cursor::position().unwrap().1),
                                     Clear(ClearType::CurrentLine)
                            )?;
                            execute!(stdout, Print(format!("> {}\n", command_buffer)))?;
                            stdout.flush()?;
                        }
                    }
                    KeyCode::Down => {
                        if let Some(index) = history_index {
                            if index + 1 < command_history.len() {
                                history_index = Some(index + 1);
                            } else {
                                history_index = None;
                                command_buffer.clear();
                            }
                        }

                        if let Some(index) = history_index {
                            command_buffer = command_history[index].clone();
                        } else {
                            command_buffer.clear();
                        }
                        // Clear current input line
                        execute!(
                            stdout,
                            MoveTo(0, cursor::position().unwrap().1),
                                 Clear(ClearType::CurrentLine)
                        )?;
                        execute!(stdout, Print(format!("> {}\n", command_buffer)))?;
                        stdout.flush()?;
                    }
                    KeyCode::Char(c) => {
                        command_buffer.push(c);
                        execute!(stdout, Print(c))?;
                        stdout.flush()?;
                    }
                    KeyCode::Backspace => {
                        if !command_buffer.is_empty() {
                            command_buffer.pop();
                            // Clear current input line
                            execute!(
                                stdout,
                                MoveTo(0, cursor::position().unwrap().1),
                                     Clear(ClearType::CurrentLine)
                            )?;
                            execute!(stdout, Print(format!("> {}\n", command_buffer)))?;
                            stdout.flush()?;
                        }
                    }
                    _ => {}
                }
            }
        }

        let action = command_buffer.trim().to_string();

        if action == "q" {
            execute!(stdout, Print("Thank you for playing!\n"))?;
            break;
        }

        // **Save the Action to Command History**
        if command_history.len() >= 10 {
            command_history.pop_front(); // Keep only the last 10 commands
        }
        command_history.push_back(action.clone());

        // **Process Action**
        let action_output = if action.starts_with("train ") {
            let skill_name = action.trim_start_matches("train ").to_string();
            let skill_name = skill_name[..1].to_uppercase() + &skill_name[1..].to_lowercase();

            if player.skills.contains_key(&skill_name) {
                let xp_gain = random_range(20, 50);
                player.gain_skill_xp(&skill_name, xp_gain);
                format!("You train {} and gain {} XP!", skill_name, xp_gain)
            } else {
                format!("Skill not found: {}", skill_name)
            }
        } else {
            match action.as_str() {
                "w" => {
                    map.move_player(Direction::Up);
                    "You moved up.".to_string()
                }
                "s" => {
                    map.move_player(Direction::Down);
                    "You moved down.".to_string()
                }
                "a" => {
                    map.move_player(Direction::Left);
                    "You moved left.".to_string()
                }
                "d" => {
                    map.move_player(Direction::Right);
                    "You moved right.".to_string()
                }
                "i" => {
                    player.print_inventory(&mut stdout)?;
                    "You checked your inventory.".to_string()
                }
                "status" => {
                    player.status(&mut stdout)?;
                    "You checked your status.".to_string()
                }
                _ => "Invalid action, please choose a valid option.".to_string(),
            }
        };

        // **Add Action Output to Recent Actions**
        if !action_output.is_empty() {
            if recent_actions.len() >= 3 {
                recent_actions.pop_front();
            }
            recent_actions.push_back(action_output);
        }

        // **Quest Completion Check**
        for quest in &mut quests {
            if !quest.completed && map.player_on_specific_tile((2, 2)) {
                let quest_output = format!(
                    "You found the lost sword!\nQuest Completed: {}",
                    quest.name
                );
                execute!(stdout, Print(format!("{}\n", quest_output)))?;
                quest.complete();
                player.add_item("Lost Sword".to_string());
                player.experience += 50;
                player.level_up();
                if recent_actions.len() >= 3 {
                    recent_actions.pop_front();
                }
                recent_actions.push_back(quest_output);
            }
        }

        // **Enemy Encounter**
        if map.player_on_specific_tile((1, 1)) {
            let mut enemy = Enemy::new("Goblin", 30, 5);
            execute!(stdout, Print(format!("You encountered a {}!\n", enemy.name)))?;

            while enemy.health > 0 && player.health > 0 {
                execute!(
                    stdout,
                    Print(format!(
                        "Your health: {} | {}'s health: {}\n",
                        player.health, enemy.name, enemy.health
                    )),
                    Print("What will you do? (1) Attack (2) Defend)\n"),
                         Print("> ")
                )?;
                stdout.flush()?;

                // Handle combat input
                let combat_action = loop {
                    if let Event::Key(KeyEvent { code, .. }) = read()? {
                        match code {
                            KeyCode::Enter => {
                                execute!(stdout, Print("\n"))?;
                                break String::new();
                            }
                            KeyCode::Char(c) => {
                                break c.to_string();
                            }
                            _ => {}
                        }
                    }
                };

                let combat_action = combat_action.trim();

                match combat_action {
                    "1" => {
                        let player_damage = random_range(10, 20);
                        execute!(
                            stdout,
                            Print(format!(
                                "You attack the {} for {} damage!\n",
                                enemy.name, player_damage
                            ))
                        )?;
                        enemy.take_damage(player_damage);
                        if enemy.is_defeated() {
                            execute!(
                                stdout,
                                Print(format!("You have defeated the {}!\n", enemy.name))
                            )?;
                            player.gain_skill_xp("Attack", 20);
                            player.level_up();
                            break;
                        }
                        let enemy_damage = enemy.attack + random_range(0, 5);
                        execute!(
                            stdout,
                            Print(format!(
                                "The {} hits you for {} damage!\n",
                                enemy.name, enemy_damage
                            ))
                        )?;
                        player.health -= enemy_damage;
                    }
                    "2" => {
                        let reduced_damage = random_range(1, 5);
                        execute!(
                            stdout,
                            Print(format!(
                                "You brace for the attack. The {} hits you for {} damage!\n",
                                enemy.name, reduced_damage
                            ))
                        )?;
                        player.health -= reduced_damage;
                    }
                    _ => {
                        execute!(
                            stdout,
                            Print("Invalid action, please choose 1 or 2.\n")
                        )?;
                    }
                }

                if player.health <= 0 {
                    execute!(
                        stdout,
                        Print(format!(
                            "You have been defeated by the {}. Game Over.\n",
                            enemy.name
                        ))
                    )?;
                    disable_raw_mode()?;
                    execute!(stdout, LeaveAlternateScreen)?;
                    return Ok(()); // Exit the game
                }
            }
        }
    }

    // **Exit Raw Mode and Leave Alternate Screen**
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}
