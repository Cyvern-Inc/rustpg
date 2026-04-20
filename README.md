# Rust CLI RPG

Welcome to Rust CLI RPG, a command-line role-playing game built entirely in pure Rust! This game features a variety of skills to train, quests to complete, enemies to fight, and loot to collect, all within a simple text-based environment.

## Features
- **Advanced TUI**: A responsive, centered terminal interface with dynamic resizing, sidebar action feeds, and multi-line command input.
- **Dynamic NPC System**: Overworld entities with roaming behaviors, home-camp logic, and a branching dialogue system.
- **Automated Combat (FAF)**: A "Find and Fight" automation loop with BFS pathfinding, customizable attack styles, and smart resource management.
- **Robust Save Management**: Slot-based save system allowing for character duplication, deletion, and detailed metadata tracking (level, last played).
- **Data-Driven Content**: Enemies, items, and loot tables are managed via external JSON files for easy balancing and expansion.
- **Action Feed**: A persistent sidebar that merges consecutive events (e.g., multiple kills of the same enemy) to keep the combat log clean and readable.

# Planned Features
- **Overworld Enemies**: Implement more intelligent enemy behaviors, such as overworld enemies attacking players, and smarter situational combat logic.
- **Crafting System**: Enable players to craft items from gathered resources.
- **Expanded Skills**: Add more skills and deeper progression.
- **Enhanced Storyline**: Develop a more intricate and engaging narrative with multiple quests and story arcs.


## Getting Started

### Installing from executable/precompiled package

Visit the [Releases](https://github.com/Cyvern-Inc/rustpg/releases) page and download the latest release for your oporating system.

NOTE: While this game is in early development, precompiled releases will be few and far between. It is recomended to instead compile from source by following the instructions bellow.

### Prerequisites
- **Rust**: Make sure you have Rust installed. You can install Rust using [rustup](https://rustup.rs/).

### Installing from source

1. **Clone the repository**:
   ```sh
   git clone https://github.com/Cyvern-Inc/rustpg
   cd rustpg
   ```

2. **Build the project**:
   ```sh
   cargo build
   ```

3. **Run the game**:
   ```sh
   cargo run
   ```

## Controls
- **Movement**: Use `w`, `a`, `s`, `d` to move.
- **Command Input**: Press `Enter` to open the command box.
- **Submit Command**: Type your command and press `Enter` again.
- **Cancel Input**: Press `Esc` to clear the buffer and return to walk mode.
- **Quit**: Type `q` or `quit` in the command box to save and exit.

## Common Commands
- `status`: View detailed player statistics and skill levels.
- `i` or `inventory`: Manage your items and equipment.
- `quests`: Check active quest progress and objectives.
- `cut` / `gather wood`: Interact with nearby trees.
- `fish` / `gather fish`: Interact with nearby water sources.
- `talk`: Initiate dialogue with adjacent NPCs.
- `faf`: Begin the "Find and Fight" auto-combat loop (use `faf spell` or `faf charged` for different styles).

## Skills Overview
- **Combat**: Train Attack, Defense, and Magic to survive encounters with tougher enemies.
- **Woodcutting**: Harvest trees for logs; includes stump regeneration logic and specialized gathering tools.
- **Fishing**: Catch various types of fish from water tiles to be used as food.
- **Cooking**: Prepare raw ingredients into life-saving consumables.

## Loot System and Inventory Management
- **Loot Tables**: Enemies drop loot based on defined loot tables. For example, goblins may drop items like coins, weapons, and consumables.
- **Item Types**: Items are categorized into currency, combat items, consumables, and miscellaneous items. Loot is added directly to the player's inventory, and items of the same type will stack.
- **Example Items**:
  - **Currency**: Gold Coins, Silver Coins, Copper Coins.
  - **Combat**: Bronze Dagger, Leather Armor.
  - **Consumables**: Raw Shrimp, Healing Potions.
  - **Miscellaneous**: Leather Scraps, Small Bones.

## Example Gameplay
Upon starting the game, you'll be presented with a quest to find the lost sword. Navigate the map, face enemies like goblins, and train your skills to become stronger. The game will present options for movement, combat, and more through text-based commands.

**Example Output**:
```
(w/a/s/d) move | (status) player status | (quests) view quests
(i) inventory | (m) menu | (q) quit

. . . . . . . . r . . . . r t . . . t . .     Recent Actions:
. . . . . . . . . . . . r . . . . . . t r     ----------
. . . . . . . t . . . . . . . t t . . . .     ----------
r . . . . . . . . . . . . . . . . . . . .     ----------
. . . . . t t . t r t . t r . . . . . r .     ----------
. . . . . r . . . . . . . . . . . . . . .     ----------
. . . . . . . . . . . . . . t . . . . . .     ----------
t . . . . . . r t . . . . . . . . . . . .     ----------
. . r . . . t . . . . . . . . r . . . . .     ----------
. . . . . . . . . . . . . . . . . . . . .     ----------
. . . . . . . . . . P t . . . . . . . . .     ----------
t . . . r . . r . . # . . . . . . . t . .     ----------
t . . . r t . . . . . . t . . . . . . r .     ----------
. . . . . . . . t . r . . . t . . . . . .     ----------
. . . . . . . . . . . . . t . . . t . . r     ----------
. . t t . . . t . r . . t . t . . . . . .     ----------
. . . . r . . r . . . . . . . . . . . . .     ----------
. r . . t . . . . . . . t . . . . . . . .     ----------
. . . . . . . . . . . t t . . . . . t . .     ----------
. . . r . . . . t t . t t . . . . . . . .     ----------
r . . . . . . . . . . . . t . . . . . . .     ----------

What would you like to do?...

```

When you defeat an enemy, you may see a message like:
```
Defeated a Goblin | +10xp | Looted: (3) Feathers, (1) Leather Scrap, (2) Copper Coins
```

## Contribution
Feel free to contribute to this project by forking the repository and creating a pull request. Any improvements, new features, or bug fixes are welcome!

1. **Fork the repository**
2. **Create your feature branch** (`git checkout -b feature/AmazingFeature`)
3. **Commit your changes** (`git commit -m 'Add some AmazingFeature'`)
4. **Push to the branch** (`git push origin feature/AmazingFeature`)
5. **Open a pull request**

## License
This project is licensed under the GNU GENERAL PUBLIC LICENSE Version 3 (GNUGPL v3). See the LICENSE file for more details.

## Acknowledgments
- Thanks to the Rust community for providing documentation and support.
- Special thanks to contributors who helped improve the game and add more exciting features.

Enjoy your adventure in the Rust CLI RPG!
