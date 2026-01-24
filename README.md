# Scoundrel ⚔️
## Built with MinUI

This is a terminal-based implementation of the solo dungeon-crawling card game. Face monsters, equip weapons, and drink potions as you navigate through a deadly deck of cards.

<p align="center">
    <img width="1720" height="1854" alt="CleanShot 2026-01-23 at 22 15 40@2x" src="https://github.com/user-attachments/assets/2d931c03-6bd8-4b22-86e8-8b67242006b6" />
</p>

## About the Game
Scoundrel is a single-player card game where you delve into a dungeon represented by a modified deck of cards. Each room presents difficult choices: face the dangers head-on or avoid them for now? Manage your health, weapons, and limited healing to survive until the final card.

## How to Play
### Setup
The game uses a modified 52-card deck:

- **Remove**: All Jokers, all red face cards (♥/♦ J, Q, K), and both red Aces (♥A, ♦A)
- **What remains** (40 cards total):

    - ♠ Spades & ♣ Clubs: Monsters (2-A, 13 cards each = 26 total)
    - ♦ Diamonds: Weapons (2-10 only = 9 cards)
    - ♥ Hearts: Health Potions (2-10 only = 5 cards)

You start with 20 Health Points (HP).

### The Room
Each turn, 4 cards are dealt face-up to form a "room." You have two choices:

1. **Face the Room** (f): Interact with exactly 3 of the 4 cards, one at a time. The 4th card remains and carries over to the next room.
2. **Skip the Room** (s): Place all 4 cards at the bottom of the deck and deal 4 new ones. **NOTE**: You cannot skip two rooms in a row!

### Card Types & Interactions
**🧌 Monsters (♠ Spades / ♣ Clubs)**

Monster strength equals card value (J=11, Q=12, K=13, A=14).

- **Fighting bare-handed**: Take damage equal to the monster's full strength
- **Fighting with a weapon**: Take damage equal to (Monster Strength - Weapon Value), minimum 0

**Weapon Degradation**: After slaying a monster with a weapon, that weapon can only be used against weaker monsters (lower value than the one just defeated).

**⚔️ Weapons (♦ Diamonds)**

Equip a weapon by selecting it. Your new weapon replaces any previously equipped weapon.

**🧪 Potions (♥ Hearts)**

Heals you for the card's value, up to a maximum of 20 HP.
**Limit**: You can only use one potion per room. Selecting a second potion in the same room wastes it.

### Winning & Losing

- **Victory**: Survive through the entire deck. Your score is your remaining HP.
- **Defeat**: Your HP reaches 0. Your score is a negative number equal to the total strength of all remaining monsters.

## Controls
**Main Menu**

- `start` - Begin a new game
- `exit` / `quit` - Exit the game
- `Ctrl+Q` - Quick exit

**During Gameplay**

- `f` - Face the current room
- `s` - Skip the current room (once per two rooms)
- `1-4` - Select a card by number
    - Can also click a card to select it
- `y` / `n` - Answer weapon usage prompts
- `Enter` - Continue after card resolution
- `restart` - Start a new game at any time

## Game Strategy Tips

- **Weapon management**: Try to upgrade weapons progressively. A degraded high-value weapon becomes less useful. Also, consider not using your weapon on a low-value monster to save it for a more challenging fight.
- **Potion timing**: Save potions for rooms with multiple monsters or when HP is critically low.
- **Skip wisely**: Skipping puts dangerous rooms at the bottom of the deck, but you'll face them eventually. Use skips to delay unavoidable threats until you're better equipped.
- **Card counting**: Track which high-value monsters remain. If you have a strong weapon, save it for the toughest fights.
