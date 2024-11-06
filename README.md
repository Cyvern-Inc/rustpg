Rust CLI RPG

Welcome to Rust CLI RPG, a command-line role-playing game built entirely in pure Rust! This game features a variety of skills to train, quests to complete, and enemies to fight, all within a simple text-based environment.

Features
- Tile-Based Map Navigation: Explore a 10x30 grid map with directional commands.
- Quests and Story: Engage in quests like retrieving the lost sword from a goblin camp.
- Combat System: Fight enemies, including goblins, using attack or defense options.
- Skills: Train various skills, such as Attack, Strength, and more, with a level-up system.
- Inventory System: Manage the items you collect during your adventures.
- Command History Navigation: Use the up and down arrow keys to navigate through your previous commands, similar to a shell environment.

Getting Started

Prerequisites
- Rust: Make sure you have Rust installed. You can install Rust using rustup (https://rustup.rs/).

Installing
1. Clone the repository:
   git clone [https://github.com/yourusername/rust_cli_rpg.git](https://github.com/Cyvern-Inc/rustpg)
   cd rust_cli_rpg

2. Build the project:
   cargo build

3. Run the game:
   cargo run

Controls
- Movement: Use w, a, s, d to move up, left, down, and right respectively.
- Inventory: Type i to check your inventory.
- Player Status: Type status to view your player stats.
- Train Skills: Type train <skill> to train a specific skill.
- Quit: Type q to quit the game.
- Command History: Use the up and down arrow keys to navigate through your recent commands.

Skills Overview
- Combat Skills: Train skills like Attack, Defense, and Magic to become a more formidable warrior.
- Gathering Skills: Mine ores, fish, or cut down trees to gather resources.
- Utility Skills: Use Thieving to pickpocket NPCs, or Sourceries for utility spells.

Example Gameplay
Upon starting the game, you'll be presented with a quest to find the lost sword. Navigate the map, face enemies like goblins, and train your skills to become stronger. The game will present options for movement, combat, and more through text-based commands.

Your current position: (0, 0)
P....
.....
.....
What will you do? (w/a/s/d to move, i for inventory, status to check player status, train <skill> to train a skill, q to quit)
> w
You moved up.

Contribution
Feel free to contribute to this project by forking the repository and creating a pull request. Any improvements, new features, or bug fixes are welcome!

1. Fork the repository
2. Create your feature branch (git checkout -b feature/AmazingFeature)
3. Commit your changes (git commit -m 'Add some AmazingFeature')
4. Push to the branch (git push origin feature/AmazingFeature)
5. Open a pull request

License
This project is licensed under the MIT License. See the LICENSE file for more details.

Acknowledgments
- Thanks to the Rust community for providing documentation and support.
- Special thanks to contributors who helped improve the game and add more exciting features.

Enjoy your adventure in the Rust CLI RPG!
