//! Rendering helpers
//! Core game rules and state transitions in `logic.rs`

use crate::logic::Card;
use minui::prelude::*;

/// Returns a short glyph string like `9󰣎` or `A󰋑`
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

/// Card foreground colors:
/// - Diamonds/Hearts: red
/// - Spades/Clubs: white
pub fn card_color(card: Card) -> ColorPair {
    match card.suit {
        'D' | 'H' => ColorPair::new(Color::LightRed, Color::Transparent),
        _ => ColorPair::new(Color::White, Color::Transparent),
    }
}

/// HP text color used for the status line
pub fn health_color(hp: i32) -> ColorPair {
    let fg = if hp > 10 {
        Color::Green
    } else if hp > 5 {
        Color::Yellow
    } else {
        Color::Red
    };
    ColorPair::new(fg, Color::Transparent)
}

/// Returns a fixed-width HP bar like `█████░░░░░` (clamped to `[0, max_hp]`)
pub fn health_bar(hp: i32, max_hp: i32) -> String {
    let max_hp = max_hp.max(0);
    let hp = hp.max(0).min(max_hp);

    let filled = "█".repeat(hp as usize);
    let empty = "░".repeat((max_hp - hp) as usize);
    format!("{filled}{empty}")
}

/// Formats a "health line" for UI display, e.g.:
/// `Health: 12/20 |████████████░░░░░░░░|`
pub fn health_line(hp: i32, max_hp: i32) -> String {
    format!("Health: {hp}/{max_hp} |{}|", health_bar(hp, max_hp))
}

/// Formats a weapon label, including the "must be < N" restriction when present
///
/// Example outputs:
/// - `Weapon: None`
/// - `Weapon: 7 (must be < 10)`
pub fn weapon_line(weapon: Option<Card>, last_monster_slain_with_weapon: Option<u8>) -> String {
    match weapon {
        None => "Weapon: None".to_string(),
        Some(w) => {
            let limit = last_monster_slain_with_weapon
                .map(|l| format!(" (must be < {l})"))
                .unwrap_or_default();
            format!("Weapon: {}{limit}", card_text(w))
        }
    }
}
