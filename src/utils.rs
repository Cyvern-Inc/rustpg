use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::Duration;
use rand::Rng;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crate::actions::ActionEntry;
use crate::map::{Map, Direction};

/// Width of the health bar fill area (between `[` and `]`).
pub const HEALTH_BAR_W: usize = 36;

/// A HEALTH_BAR_W-wide colored progress bar.
/// Green █ for remaining health, red ░ for missing health.
pub fn health_bar(current: i32, max: i32) -> String {
    let ratio = (current.max(0) as f32 / max.max(1) as f32).clamp(0.0, 1.0);
    let filled = (ratio * HEALTH_BAR_W as f32).round() as usize;
    let empty = HEALTH_BAR_W.saturating_sub(filled);
    format!(
        "[\x1B[32m{}\x1B[31m{}\x1B[0m]",
        "█".repeat(filled),
        "░".repeat(empty),
    )
}

/// Compute safe `(h_radius, v_radius)` for any screen that renders a map
/// viewport. Pass the number of non-map lines that will be printed on the
/// same screen (below the map). The formula guarantees the total output never
/// exceeds the current terminal height, preventing any content from scrolling
/// off the top.
///
/// Rule: after the clear-escape, the cursor starts at row 1. Each `\r\n` or
/// `\n`/`println!` advances it by one. For no scrolling the cursor must reach
/// at most row `th` before the user's input line. Working backwards:
///   total newlines = (v_r×2+1) + v_overhead  ≤  th − 1
///   → v_r ≤ (th − v_overhead − 2) / 2
/// We use `saturating_sub` and clamp to [1, 40] so very small terminals
/// shrink the map gracefully rather than overflowing.
pub fn viewport_radii(v_overhead: usize) -> (usize, usize) {
    let (tw, th) = term_size::dimensions().unwrap_or((80, 24));
    let h_r = (tw.saturating_sub(52) / 4).clamp(1, 40);
    let v_r = (th.saturating_sub(v_overhead + 2) / 2).clamp(1, 40);
    (h_r, v_r)
}

pub struct MovementWeights {
    pub same_direction: u32,
    pub up: u32,
    pub down: u32,
    pub left: u32,
    pub right: u32,
}

/// Word-wrap `s` so that each returned line is at most `width` visible chars.
pub fn wrap_text(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![s.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(s.to_string());
    }
    lines
}

/// Build a complete `\r\n`-terminated screen buffer for automatic/continuous
/// modes (faf, gather wood, woodcutting). Displays the map viewport on the
/// left and the recent-actions feed on the right, just like the main view.
///
/// `header` is a single line shown at the top (e.g. mode name + key hints).
pub fn render_mode_frame(
    game_map: &Map,
    header: &str,
    recent_actions: &VecDeque<ActionEntry>,
) -> String {
    let (tw, _th) = term_size::dimensions().unwrap_or((80, 24));
    // 4 overhead lines below the map: blank + label + trailing \r\n + spare
    let (h_r, v_r) = viewport_radii(4);

    let map_str = game_map.render_viewport(h_r, v_r);
    let map_lines: Vec<&str> = map_str.lines().collect();
    let map_height = map_lines.len();
    let max_recent = if map_height > 1 { map_height - 1 } else { 0 };

    let sidebar_w = tw.saturating_sub((2 * h_r + 1) * 2 + 4).max(10);
    let mut all_lines: Vec<String> = Vec::new();
    for entry in recent_actions {
        for line in entry.format_for_sidebar() {
            all_lines.extend(wrap_text(&line, sidebar_w));
        }
    }
    let mut info_lines: Vec<String> = vec!["Recent Actions:".to_string()];
    let skip = all_lines.len().saturating_sub(max_recent);
    for line in &all_lines[skip..] {
        info_lines.push(line.clone());
    }
    if max_recent > 0 {
        while info_lines.len() <= max_recent {
            info_lines.push("----------".to_string());
        }
    }

    let map_width = map_lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let max_rows = map_lines.len().max(info_lines.len());

    let mut out = String::new();
    out.push_str("\x1B[2J\x1B[1;1H");
    for i in 0..max_rows {
        let map_part = if i < map_lines.len() { map_lines[i] } else { "" };
        let info_part = if i < info_lines.len() { info_lines[i].as_str() } else { "" };
        out.push_str(&format!(
            "{:<width$}    {}\r\n",
            map_part, info_part, width = map_width
        ));
    }
    // Header below the map so it stays visible regardless of terminal height
    out.push_str("\r\n");
    out.push_str(header);
    out.push_str("\r\n");
    out
}


/// Count the visible terminal columns a string occupies, ignoring ANSI
/// escape sequences of the form `ESC [ ... m`.
fn visible_len(s: &str) -> usize {
    let mut len = 0usize;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1B' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

/// Clear the screen and draw a centered titled box — no version header.
/// Used for in-game result/dialog screens (combat victory, defeat, etc.).
/// Returns the left-indent string for aligning a prompt below the box.
pub fn draw_in_game_box(title: &str, rows: &[String]) -> String {
    let (tw, th) = term_size::dimensions().unwrap_or((80, 24));

    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();

    // Box chrome: top border + title + separator + bottom border = 4 lines.
    // Reserve 1 line below box for a prompt/input, 1 spare = 6 total overhead.
    // Truncate content rows so the box never causes scrolling.
    let max_content = th.saturating_sub(6).max(1);
    let owned_truncated: Vec<String>;
    let rows: &[String] = if rows.len() > max_content {
        owned_truncated = {
            let mut v: Vec<String> = rows[..max_content.saturating_sub(1)].to_vec();
            v.push("  ...".to_string());
            v
        };
        &owned_truncated
    } else {
        rows
    };

    let content_max = rows.iter().map(|r| visible_len(r)).max().unwrap_or(0);
    let inner = (content_max + 4).max(title.len() + 4).max(32);

    let title_lpad = inner.saturating_sub(title.len()) / 2;
    let title_line = format!("{}{}", " ".repeat(title_lpad), title);

    // box_h: top border + title + separator + rows + bottom border
    let box_h = rows.len() + 4;
    let vert_free = th.saturating_sub(box_h + 2);
    let top_blank = vert_free / 2;

    for _ in 0..top_blank {
        println!();
    }

    let pad = " ".repeat(tw.saturating_sub(inner + 2) / 2);
    println!("{}╔{}╗", pad, "═".repeat(inner));
    println!("{}║{:<width$}║", pad, title_line, width = inner);
    println!("{}╠{}╣", pad, "═".repeat(inner));
    for row in rows {
        let trailing = " ".repeat(inner.saturating_sub(visible_len(row)));
        println!("{}║{}{}║", pad, row, trailing);
    }
    println!("{}╚{}╝", pad, "═".repeat(inner));

    pad
}

pub fn check_for_input() -> Option<String> {
    if event::poll(Duration::from_secs(0)).unwrap() {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
            if modifiers.is_empty() {
                match code {
                    KeyCode::Char('p') => return Some("p".to_string()),
                    KeyCode::Char('b') => return Some("b".to_string()),
                    KeyCode::Char('q') => return Some("q".to_string()),
                    KeyCode::Char('x') => return Some("x".to_string()),
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
    _game_map: &Map,
) -> Direction {
    let mut directions = vec![
        (Direction::Up, weights.up),
        (Direction::Down, weights.down),
        (Direction::Left, weights.left),
        (Direction::Right, weights.right),
    ];

    for (dir, weight) in &mut directions {
        if *dir == prev_direction {
            *weight += weights.same_direction;
        }
    }

    let total_weight: u32 = directions.iter().map(|(_, weight)| *weight).sum();
    let mut choice = rng.gen_range(0..total_weight);

    for (dir, weight) in directions {
        if choice < weight {
            return dir;
        }
        choice -= weight;
    }

    prev_direction
}

