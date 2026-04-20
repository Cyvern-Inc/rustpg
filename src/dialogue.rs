use std::collections::HashMap;
use std::io::{self, Write};

use crate::npc::Condition;
use crate::player::Player;

// ---------------------------------------------------------------------------
// Dialogue tree types
// ---------------------------------------------------------------------------

/// An effect that fires when a dialogue option is chosen.
#[derive(Clone)]
pub enum DialogueEffect {
    GiveQuest(u32),
    CompleteQuest(u32),
    GiveItem(u32, u32),  // item_id, qty
    TakeItem(u32, u32),
}

/// Where the dialogue goes after an option is chosen.
#[derive(Clone)]
pub enum NextNode {
    Node(&'static str),
    End,
}

/// A player-visible option within a BranchNode.
#[derive(Clone)]
pub struct DialogueOption {
    pub label: &'static str,
    pub condition: Option<Condition>,
    pub effects: Vec<DialogueEffect>,
    pub next: NextNode,
}

/// A visible dialogue node: shows NPC text and player-selectable options.
pub struct BranchNode {
    pub speaker: &'static str,
    pub text: &'static str,
    pub options: Vec<DialogueOption>,
}

/// An invisible redirect node: evaluates conditions and jumps to the first
/// matching branch without displaying anything to the player.
pub struct GatewayNode {
    /// (condition, target_node_id) pairs evaluated top-down.
    /// The first condition that passes (or the first `None`) wins.
    pub branches: Vec<(Option<Condition>, &'static str)>,
}

pub enum DialogueNode {
    Branch(BranchNode),
    Gateway(GatewayNode),
}

// ---------------------------------------------------------------------------
// Global dialogue registry
// ---------------------------------------------------------------------------

fn build_dialogue_tree() -> HashMap<&'static str, DialogueNode> {
    let mut nodes: HashMap<&'static str, DialogueNode> = HashMap::new();

    // -----------------------------------------------------------------------
    // Woodsman dialogue
    // -----------------------------------------------------------------------

    // Entry gateway — routes to the right conversation phase based on quest state
    nodes.insert("woodsman_gateway", DialogueNode::Gateway(GatewayNode {
        branches: vec![
            (Some(Condition::QuestCompleted(9)), "woodsman_postcomplete"),
            // Quest active + has goblin head → turn-in
            (Some(Condition::QuestActive(9)), "woodsman_check_head"),
            // Quest not yet given → intro
            (None, "woodsman_greeting"),
        ],
    }));

    // Gateway: if player has the goblin head route to turn-in, else in-progress
    nodes.insert("woodsman_check_head", DialogueNode::Gateway(GatewayNode {
        branches: vec![
            (Some(Condition::PlayerHasItem(100052)), "woodsman_turnin"),
            (None, "woodsman_inprogress"),
        ],
    }));

    nodes.insert("woodsman_greeting", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Ah, a traveller! I don't get many visitors out this way. \
               Name's Aldric. I'm a naturalist, studying the wildlife around these parts. \
               Been out here for months.",
        options: vec![
            DialogueOption {
                label: "Nice to meet you, Aldric.",
                condition: None,
                effects: vec![],
                next: NextNode::Node("woodsman_needhelp"),
            },
            DialogueOption {
                label: "Goodbye.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_needhelp", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Actually, since you're here, I wonder if you might be able to help \
               me with something? I'd make it worth your while.",
        options: vec![
            DialogueOption {
                label: "What do you need?",
                condition: None,
                effects: vec![],
                next: NextNode::Node("woodsman_offer"),
            },
            DialogueOption {
                label: "I'm a bit busy right now.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_offer", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "I've been trying to study the local goblin population. Fascinating \
               creatures, really. Dangerous, but fascinating. If you could bring me \
               back a goblin head, I could examine it up close. I promise I'd reward \
               you well for the trouble.",
        options: vec![
            DialogueOption {
                label: "I'll do it.",
                condition: None,
                effects: vec![DialogueEffect::GiveQuest(9)],
                next: NextNode::Node("woodsman_accepted"),
            },
            DialogueOption {
                label: "That sounds dangerous.",
                condition: None,
                effects: vec![],
                next: NextNode::Node("woodsman_softdecline"),
            },
            DialogueOption {
                label: "No thanks.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_softdecline", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Ha! Fair enough. I wouldn't expect anyone to walk into a goblin \
               camp without a good reason. The offer stands if you change your mind.",
        options: vec![
            DialogueOption {
                label: "Actually, I'll help.",
                condition: None,
                effects: vec![DialogueEffect::GiveQuest(9)],
                next: NextNode::Node("woodsman_accepted"),
            },
            DialogueOption {
                label: "Goodbye.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_accepted", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Wonderful! Be careful out there. Goblins travel in packs and \
               they're nastier than they look. Good luck.",
        options: vec![
            DialogueOption {
                label: "I'll be careful. Thanks.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_inprogress", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Ah, still out there hunting? Take your time. The goblins \
               aren't going anywhere. I'll be here when you get back.",
        options: vec![
            DialogueOption {
                label: "I'll keep at it.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_turnin", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Back already? Did you find anything useful out there?",
        options: vec![
            DialogueOption {
                label: "I found a goblin head.",
                condition: Some(Condition::PlayerHasItem(100052)),
                effects: vec![],
                next: NextNode::Node("woodsman_complete"),
            },
            DialogueOption {
                label: "Not yet, still looking.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_complete", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Remarkable... look at these teeth. The bone structure here is \
               extraordinary. This is incredibly helpful for my research. \
               And I had an idea. Let me fashion something from this for you. \
               It just might come in handy if you ever need to blend in with \
               the local goblin population.",
        options: vec![
            DialogueOption {
                label: "Thank you, Aldric.",
                condition: None,
                effects: vec![
                    DialogueEffect::TakeItem(100052, 1),
                    DialogueEffect::CompleteQuest(9),
                ],
                next: NextNode::End,
            },
        ],
    }));

    nodes.insert("woodsman_postcomplete", DialogueNode::Branch(BranchNode {
        speaker: "Woodsman",
        text: "Good to see you again. That mask should serve you well out there. \
               Goblins aren't the brightest, but they do recognise their own.",
        options: vec![
            DialogueOption {
                label: "Good to know. Goodbye.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    // -----------------------------------------------------------------------
    // Goblin dialogue (accessed when wearing the Goblin Mask)
    // -----------------------------------------------------------------------

    nodes.insert("goblin_neutral_greeting", DialogueNode::Branch(BranchNode {
        speaker: "Goblin",
        text: "Grrkk... you smell almost like kin. Almost. What you want?",
        options: vec![
            DialogueOption {
                label: "Nothing. Just passing through.",
                condition: None,
                effects: vec![],
                next: NextNode::End,
            },
        ],
    }));

    nodes
}

// ---------------------------------------------------------------------------
// Dialogue runner
// ---------------------------------------------------------------------------

/// Run a full dialogue session starting at `root_id`. Returns a short summary
/// string for the recent actions feed.
pub fn run_dialogue(player: &mut Player, root_id: &str) -> String {
    let nodes = build_dialogue_tree();

    let mut current_id = root_id.to_string();
    let mut last_speaker = "NPC".to_string();

    loop {
        let node = match nodes.get(current_id.as_str()) {
            Some(n) => n,
            None => {
                println!("\n[Dialogue error: unknown node '{}'. Press Enter.]\n", current_id);
                let _ = io::stdin().read_line(&mut String::new());
                break;
            }
        };

        match node {
            DialogueNode::Gateway(gw) => {
                // Find the first branch whose condition passes
                let mut jumped = false;
                for (condition, target) in &gw.branches {
                    let passes = match condition {
                        Some(cond) => cond.evaluate(player),
                        None => true,
                    };
                    if passes {
                        current_id = target.to_string();
                        jumped = true;
                        break;
                    }
                }
                if !jumped {
                    break; // no branch matched — shouldn't happen if trees are well-formed
                }
            }

            DialogueNode::Branch(branch) => {
                last_speaker = branch.speaker.to_string();

                // Filter options by condition
                let visible: Vec<(usize, &DialogueOption)> = branch
                    .options
                    .iter()
                    .enumerate()
                    .filter(|(_, opt)| {
                        opt.condition.as_ref().map(|c| c.evaluate(player)).unwrap_or(true)
                    })
                    .collect();

                // Render
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().unwrap();
                println!();
                println!("  [{}]", branch.speaker);
                println!();
                // Wrap long NPC text at ~70 chars
                for line in wrap_dialogue(branch.text, 70) {
                    println!("  {}", line);
                }
                println!();
                for (display_idx, (_, opt)) in visible.iter().enumerate() {
                    println!("  {}. {}", display_idx + 1, opt.label);
                }
                println!("  0. (leave)");
                println!();
                print!("> ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read input");
                let choice = input.trim();

                if choice == "0" || choice == "q" || choice == "x" {
                    break;
                }

                let picked = choice.parse::<usize>().ok().and_then(|n| {
                    if n >= 1 && n <= visible.len() {
                        Some(visible[n - 1].1)
                    } else {
                        None
                    }
                });

                match picked {
                    None => continue, // invalid input — re-show the same node
                    Some(opt) => {
                        // Fire effects in order
                        let mut quest_notifications: Vec<String> = Vec::new();
                        for effect in &opt.effects {
                            match effect {
                                DialogueEffect::GiveQuest(id) => {
                                    if player.give_quest(*id) {
                                        quest_notifications
                                            .push(format!("  New quest: {}", quest_name_for(*id)));
                                    }
                                }
                                DialogueEffect::CompleteQuest(id) => {
                                    let notes = player.complete_quest_via_dialogue(*id);
                                    quest_notifications.extend(notes);
                                }
                                DialogueEffect::GiveItem(item_id, qty) => {
                                    player.add_item_to_inventory(*item_id, *qty);
                                }
                                DialogueEffect::TakeItem(item_id, qty) => {
                                    player.remove_item(*item_id, *qty);
                                }
                            }
                        }

                        // Show any notifications before moving to the next node
                        if !quest_notifications.is_empty() {
                            println!();
                            for note in &quest_notifications {
                                println!("{}", note);
                            }
                            println!();
                            println!("  Press Enter to continue...");
                            let _ = io::stdin().read_line(&mut String::new());
                        }

                        match &opt.next {
                            NextNode::Node(id) => current_id = id.to_string(),
                            NextNode::End => break,
                        }
                    }
                }
            }
        }
    }

    format!("Spoke with {}.", last_speaker)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn wrap_dialogue(text: &str, width: usize) -> Vec<&str> {
    // Honour explicit \n\n paragraph breaks; otherwise wrap at word boundaries.
    // Simple approach: split on \n\n, then each paragraph as a single line
    // (the terminal will soft-wrap). This keeps the code minimal.
    text.split("\n\n").collect()
}

fn quest_name_for(quest_id: u32) -> &'static str {
    match quest_id {
        9 => "A Curious Request",
        _ => "Unknown Quest",
    }
}
