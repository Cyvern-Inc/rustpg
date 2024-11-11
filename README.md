# Rust CLI RPG

Welcome to Rust CLI RPG, a command-line role-playing game built entirely in pure Rust! This game features a variety of skills to train, quests to complete, enemies to fight, and loot to collect, all within a simple text-based environment.

## Features
- **Tile-Based Map Navigation**: Explore a large map with direct user input and explore a dynamic world.
- **Quests and Story**: Engage in quests like retrieving the lost sword from a goblin camp.
- **Combat System**: Fight enemies, including goblins, using regular and heavy attacks, as well as magic options.
- **Skills**: Train various skills, such as Attack, Strength, Magic, and more, with a level-up system.
- **Inventory System**: Manage the items you collect during your adventures, including coins, weapons, armor, and other resources. Items stack in your inventory, and types are categorized for easy reference.
- **Loot System**: Defeated enemies drop loot based on defined loot tables, which are added directly to your inventory.
- **Player Status**: View detailed player stats, including health, experience, level, skills, and inventory.

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
- **Movement**: Use `w`, `a`, `s`, `d` to move up, left, down, and right respectively.
- **Inventory**: Type `i` to check your inventory.
- **Player Status**: Type `status` to view your player stats, including health, level, experience, and inventory.
- **Quit**: Type `q` to quit the game.

## Skills Overview
- **Combat Skills**: Train skills like Attack, Defense, and Magic to become a more formidable warrior.
- **Gathering Skills**: Mine ores, fish, or cut down trees to gather resources.
- **Utility Skills**: Use Thieving to pickpocket NPCs, or Sourceries for utility spells.

### Example Skills
- **Attack**: Increases damage dealt in melee combat.
- **Defense**: Increases resistance to enemy attacks.
- **Magic**: Grants access to new spells for combat and utility.
- **Fishing**: Catch fish for food to restore health.

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

