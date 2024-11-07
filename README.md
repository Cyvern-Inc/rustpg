# Rust CLI RPG

Welcome to Rust CLI RPG, a command-line role-playing game built entirely in pure Rust! This game features a variety of skills to train, quests to complete, enemies to fight, and loot to collect, all within a simple text-based environment.

## Features
- **Tile-Based Map Navigation**: Explore a large map with 30x30 visible chunks as you move through a 300x300 world.
- **Quests and Story**: Engage in quests like retrieving the lost sword from a goblin camp.
- **Combat System**: Fight enemies, including goblins, using regular and heavy attacks, as well as magic options.
- **Skills**: Train various skills, such as Attack, Strength, Magic, and more, with a level-up system.
- **Inventory System**: Manage the items you collect during your adventures, including coins, weapons, armor, and other resources. Items stack in your inventory, and types are categorized for easy reference.
- **Loot System**: Defeated enemies drop loot based on defined loot tables, which are added directly to your inventory.
- **Player Status**: View detailed player stats, including health, experience, level, skills, and inventory.
- **Command History Navigation**: Use the up and down arrow keys to navigate through your previous commands, similar to a shell environment.

## Getting Started

### Prerequisites
- **Rust**: Make sure you have Rust installed. You can install Rust using [rustup](https://rustup.rs/).

### Installing
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
- **Train Skills**: Type `train <skill>` to train a specific skill.
- **Quit**: Type `q` to quit the game.
- **Command History**: Use the up and down arrow keys to navigate through your recent commands.

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
Your current position: (0, 0)
. . . . . . . . . . . . .
. . . . P . . . . . . . .
. . . . . . . . . . . . .
What will you do? (w/a/s/d to move, i for inventory, status to check player status, train <skill> to train a skill, q to quit)
> w
You moved up.
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
This project is licensed under the MIT License. See the LICENSE file for more details.

## Acknowledgments
- Thanks to the Rust community for providing documentation and support.
- Special thanks to contributors who helped improve the game and add more exciting features.

Enjoy your adventure in the Rust CLI RPG!

