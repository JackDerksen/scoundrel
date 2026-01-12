//! Game logic

use rand::seq::SliceRandom;
use std::collections::VecDeque;

use crate::messages as msg;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub suit: char, // 'S', 'C', 'D', 'H'
    pub value: u8,  // 2-14 (ace is 14)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    RoomChoice,
    CardSelection,
    /// Used for both "acknowledge" steps and weapon prompt
    CardInteraction,
    GameOver,
}

/// Result of an action that may require an explicit "continue" (Enter) acknowledgement
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveOutcome {
    /// No immediate resolution (e.g. waiting for y/n)
    None,
    /// Resolution is complete but should wait for user acknowledgement before advancing
    AwaitContinue,
}

/// The core game model
pub struct Game {
    pub deck: VecDeque<Card>,

    /// Stable room slots (always 4). `None` indicates an empty slot
    pub room_slots: [Option<Card>; 4],

    pub health: i32,
    pub max_health: i32,

    pub weapon: Option<Card>,
    pub last_monster_slain_with_weapon: Option<u8>,
    pub potion_used_this_room: bool,

    pub can_skip: bool,
    pub state: GameState,
    pub survived: bool,

    /// Primary message to display to the player (UI chooses where/how)
    pub message: String,

    /// "Previous input" echo line
    pub last_command_feedback: String,

    // Prompt state
    pub current_monster: Option<Card>,
    pub awaiting_weapon_choice: bool,

    /// After deciding to face a room, you get exactly 3 interactions
    pub interactions_left_in_room: u8,
}

impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            deck: VecDeque::new(),
            room_slots: [None, None, None, None],

            health: 20,
            max_health: 20,

            weapon: None,
            last_monster_slain_with_weapon: None,
            potion_used_this_room: false,

            can_skip: true,
            state: GameState::MainMenu,
            survived: false,

            message: String::new(),
            last_command_feedback: String::new(),

            current_monster: None,
            awaiting_weapon_choice: false,

            interactions_left_in_room: 0,
        };

        g.create_deck();
        g
    }

    /// Reset the game into a playable "in dungeon" state (RoomChoice + initial room filled)
    pub fn reset_to_playing(&mut self) {
        *self = Self::new();
        self.state = GameState::RoomChoice;
        self.fill_room();
        self.message = msg::ENTERED_DUNGEON.to_string();
    }

    pub fn create_deck(&mut self) {
        let mut cards = Vec::new();

        for suit in ['S', 'C', 'D', 'H'] {
            for value in 2..=14u8 {
                // Red aces and face cards removed, them's da rulez
                if (suit == 'D' || suit == 'H') && value >= 11 {
                    continue;
                }
                cards.push(Card { suit, value });
            }
        }

        let mut rng = rand::thread_rng();
        cards.shuffle(&mut rng);
        self.deck = VecDeque::from(cards);
    }

    pub fn room_is_empty(&self) -> bool {
        self.room_slots.iter().all(|c| c.is_none())
    }

    /// Fill empty room slots from the top of the deck, without shifting existing cards
    pub fn fill_room(&mut self) {
        for slot in self.room_slots.iter_mut() {
            if slot.is_none() {
                if let Some(card) = self.deck.pop_front() {
                    *slot = Some(card);
                }
            }
        }
    }

    pub fn face_room(&mut self) {
        self.potion_used_this_room = false;
        self.interactions_left_in_room = 3;
        self.state = GameState::CardSelection;
        self.message = msg::FACE_ROOM.to_string();
    }

    pub fn skip_room(&mut self) {
        if !self.can_skip {
            self.message = msg::NEED_FACE_ONLY.to_string();
            return;
        }

        // Put skipped room cards at bottom of deck, currently preserving slot order
        // TODO: This order should technically be randomized
        for slot in self.room_slots.iter_mut() {
            if let Some(card) = slot.take() {
                self.deck.push_back(card);
            }
        }

        self.can_skip = false;
        self.fill_room();

        if self.room_is_empty() && self.deck.is_empty() {
            self.survived = true;
            self.state = GameState::GameOver;
            self.message = msg::YOU_SURVIVED.to_string();
        } else {
            self.message = msg::SKIPPED_ROOM.to_string();
        }
    }

    pub fn can_use_weapon_on(&self, monster: Card) -> bool {
        if self.weapon.is_none() {
            return false;
        }
        match self.last_monster_slain_with_weapon {
            None => true,
            Some(last) => monster.value < last,
        }
    }

    pub fn handle_monster_with_weapon(&mut self, monster: Card) -> i32 {
        if let Some(w) = self.weapon {
            let dmg = (monster.value as i32 - w.value as i32).max(0);
            self.last_monster_slain_with_weapon = Some(monster.value);
            dmg
        } else {
            monster.value as i32
        }
    }

    pub fn handle_monster_without_weapon(&self, monster: Card) -> i32 {
        monster.value as i32
    }

    /// Play a card, perform the card effect and transition the state accordingly
    pub fn play_card_from_slot(&mut self, idx: usize) -> ResolveOutcome {
        if self.state != GameState::CardSelection {
            self.message = msg::MUST_FACE_FIRST.to_string();
            return ResolveOutcome::None;
        }
        if idx >= 4 {
            self.message = msg::INVALID_CARD_SELECTION.to_string();
            return ResolveOutcome::None;
        }

        let card = match self.room_slots[idx].take() {
            Some(c) => c,
            None => {
                self.message = msg::INVALID_CARD_SELECTION.to_string();
                return ResolveOutcome::None;
            }
        };

        match card.suit {
            // Monster
            'S' | 'C' => {
                self.current_monster = Some(card);

                if self.can_use_weapon_on(card) {
                    self.awaiting_weapon_choice = true;
                    self.state = GameState::CardInteraction;

                    let monster_txt = card_text(card);
                    let weapon_txt = self
                        .weapon
                        .map(card_text)
                        .unwrap_or_else(|| "?".to_string());
                    self.message =
                        format!("Monster {monster_txt} — use weapon {weapon_txt}? (y/n)");

                    ResolveOutcome::None
                } else {
                    let dmg = self.handle_monster_without_weapon(card);
                    self.health -= dmg;
                    self.state = GameState::CardInteraction;
                    self.message = format!("Fought monster! Took {dmg} damage.");
                    //ResolveOutcome::AwaitContinue
                    self.continue_after_interaction();
                    ResolveOutcome::None
                }
            }

            // Weapon
            'D' => {
                self.weapon = Some(card);
                self.last_monster_slain_with_weapon = None;
                self.state = GameState::CardInteraction;
                self.message = format!("Equipped {}!", card_text(card));
                //ResolveOutcome::AwaitContinue
                self.continue_after_interaction();
                ResolveOutcome::None
            }

            // Potion
            'H' => {
                self.state = GameState::CardInteraction;
                if !self.potion_used_this_room {
                    let heal = card.value as i32;
                    self.health = (self.health + heal).min(self.max_health);
                    self.potion_used_this_room = true;
                    self.message = format!("Healed for {heal} HP.");
                } else {
                    // This string isn't centralized in messages.rs, I don't think it really needs to be
                    self.message = "Potion wasted (only 1 per room).".to_string();
                }
                //ResolveOutcome::AwaitContinue
                self.continue_after_interaction();
                ResolveOutcome::None
            }

            _ => {
                self.state = GameState::CardInteraction;
                self.message = "Unknown card.".to_string();
                ResolveOutcome::None
            }
        }
    }

    /// Answer the current weapon prompt (y/n)
    pub fn answer_weapon_prompt(&mut self, use_weapon: bool) -> ResolveOutcome {
        if !self.awaiting_weapon_choice {
            return ResolveOutcome::None;
        }

        let monster = match self.current_monster.take() {
            Some(m) => m,
            None => {
                self.awaiting_weapon_choice = false;
                return ResolveOutcome::None;
            }
        };

        let dmg = if use_weapon {
            self.handle_monster_with_weapon(monster)
        } else {
            self.handle_monster_without_weapon(monster)
        };

        self.health -= dmg;
        self.awaiting_weapon_choice = false;

        self.message = if use_weapon {
            format!("Fought with weapon! Took {dmg} damage.")
        } else {
            format!("Fought monster! Took {dmg} damage.")
        };

        ResolveOutcome::AwaitContinue
    }

    /// Continue after an acknowledged interaction (Enter)
    pub fn continue_after_interaction(&mut self) {
        // Death check
        if self.health <= 0 {
            self.survived = false;
            self.state = GameState::GameOver;
            self.message = msg::YOU_DIED.to_string();
            return;
        }

        // Consume one interaction (only after "resolved" acknowledgement)
        if self.interactions_left_in_room > 0 {
            self.interactions_left_in_room -= 1;
        }

        // End-of-room window, advance to next room
        if self.interactions_left_in_room == 0 {
            self.can_skip = true;

            // Fill gaps for the next room without shifting existing cards
            self.fill_room();

            if self.room_is_empty() && self.deck.is_empty() {
                self.survived = true;
                self.state = GameState::GameOver;
                self.message = msg::YOU_SURVIVED.to_string();
            } else {
                self.state = GameState::RoomChoice;
                self.message = msg::ROOM_RESOLVED.to_string();
            }
            return;
        }

        // Still in the room interaction window
        if self.room_is_empty() && self.deck.is_empty() {
            self.survived = true;
            self.state = GameState::GameOver;
            self.message = msg::YOU_SURVIVED.to_string();
            return;
        }

        self.state = GameState::CardSelection;
    }

    pub fn remaining_summary_line(&self) -> String {
        let mut remaining: Vec<Card> = Vec::new();
        remaining.extend(self.room_slots.iter().copied().flatten());
        remaining.extend(self.deck.iter().copied());

        let monsters: Vec<Card> = remaining
            .iter()
            .copied()
            .filter(|c| c.suit == 'S' || c.suit == 'C')
            .collect();

        if monsters.is_empty() {
            return "No monsters remain. You defeated them all!".to_string();
        }

        let total_threat: i32 = monsters.iter().map(|m| m.value as i32).sum();
        format!("Remaining monsters total threat: -{total_threat}")
    }

    pub fn final_score(&self) -> i32 {
        if self.survived {
            self.health
        } else {
            let mut remaining: Vec<Card> = Vec::new();
            remaining.extend(self.room_slots.iter().copied().flatten());
            remaining.extend(self.deck.iter().copied());

            let sum: i32 = remaining
                .iter()
                .filter(|c| c.suit == 'S' || c.suit == 'C')
                .map(|c| c.value as i32)
                .sum();

            -sum
        }
    }
}

// ==============================
// Rendering helpers (pure)
// ==============================

pub fn card_text(card: Card) -> String {
    let v = match card.value {
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        14 => "A".to_string(),
        _ => card.value.to_string(),
    };

    let s = match card.suit {
        'S' => "󱢱",
        'C' => "󱢥",
        'D' => "󱢩",
        'H' => "󱢭",
        _ => "?",
    };

    format!("{v}{s}")
}
