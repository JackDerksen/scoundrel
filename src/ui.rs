//! UI module
//!
//! Responsibilities:
//! - Own the MinUI-facing app state (`UiScene`, `TextInputState`, mouse drag flags)
//! - Route keyboard + mouse input into game actions
//! - Render the game as nested `Container`s
//! - Register clickable hitboxes for card slots via `InteractionCache::register`

use std::time::Duration;

use minui::Window;
use minui::prelude::*;
use minui::ui::UiScene;
use minui::widgets::{ContainerPadding, TextInput, TextInputState, Tooltip, WidgetArea};

use crate::logic::{Game, GameState};
use crate::messages as msg;
use crate::render::{card_color, card_text, health_color, health_line, weapon_line};

fn command_placeholder(game: &Game) -> String {
    // Keep these always-available commands last, since they're "meta" actions
    let mut parts: Vec<&'static str> = Vec::new();

    match game.state {
        GameState::MainMenu => {
            parts.push("start");
        }
        GameState::RoomChoice => {
            parts.push("f");
            if game.can_skip {
                parts.push("s");
            }
        }
        GameState::CardSelection => {
            parts.push("1..4");
        }
        GameState::CardInteraction => {
            if game.awaiting_weapon_choice {
                parts.push("y/n");
            } else {
                parts.push("(Enter)");
            }
        }
        GameState::GameOver => {
            parts.push("restart");
        }
    }

    // Global commands (always valid options)
    parts.push("restart");
    parts.push("exit");

    parts.join(" | ")
}

// ==============================
// Interaction IDs
// ==============================

pub const ID_INPUT: InteractionId = 1;
pub const ID_CARD_1: InteractionId = 101;
pub const ID_CARD_2: InteractionId = 102;
pub const ID_CARD_3: InteractionId = 103;
pub const ID_CARD_4: InteractionId = 104;

// ==============================
// AppState
// ==============================

pub struct AppState {
    pub game: Game,

    pub ui: UiScene,
    pub input: TextInputState,

    pub mouse_down: bool,
    pub dragging: bool,

    pub should_quit: bool,
    pub mouse_pos: (u16, u16),
    pub card_hovers: [HoverTracker; 4],
}

impl AppState {
    pub fn new() -> Self {
        let mut input = TextInputState::new();
        input.set_focused(true);

        Self {
            game: Game::new(),
            ui: UiScene::new(),
            input,
            mouse_down: false,
            dragging: false,
            should_quit: false,
            mouse_pos: (0, 0),
            card_hovers: [
                HoverTracker::new(),
                HoverTracker::new(),
                HoverTracker::new(),
                HoverTracker::new(),
            ],
        }
    }

    fn set_last_command_feedback(&mut self, cmd: &str) {
        self.game.last_command_feedback = format!("{}{}", msg::CMD_PREFIX, cmd);
    }
}

// ==============================
// Update
// ==============================

pub fn update(state: &mut AppState, event: Event) -> bool {
    if state.should_quit {
        return false;
    }

    // Quit (Ctrl+Q only)
    if let Event::KeyWithModifiers(k) = event {
        if matches!(k.key, KeyKind::Char('q')) && k.mods.ctrl {
            return false;
        }
    }

    // Apply scene policies (focus/capture bookkeeping)
    let _effects = state.ui.apply_policies(&event);

    // Mouse events: click-to-focus input / click-to-select cards / drag selection in input
    match event {
        Event::MouseMove { x, y } => {
            state.mouse_pos = (x, y);

            // Check which card slots are being hovered
            let hit = state.ui.hit_test_id(x, y);
            for i in 0..4 {
                let card_id = match i {
                    0 => ID_CARD_1,
                    1 => ID_CARD_2,
                    2 => ID_CARD_3,
                    _ => ID_CARD_4,
                };

                if hit == Some(card_id) {
                    state.card_hovers[i].start_hover();
                } else {
                    state.card_hovers[i].end_hover();
                }
            }
        }
        Event::MouseClick { x, y, button: _ } => {
            state.mouse_down = true;
            state.dragging = false;

            let hit = state.ui.hit_test_id(x, y);
            match hit {
                Some(ID_INPUT) => {
                    state.input.set_focused(true);
                    state.input.click_set_cursor(x);
                    return true;
                }
                Some(ID_CARD_1) => {
                    // Only allow clicking cards when we're actually in the selection state.
                    // If not, show state-appropriate guidance (avoid stale/incorrect MUST_FACE_FIRST).
                    if state.game.state == GameState::CardSelection {
                        let _ = state.game.play_card_from_slot(0);
                    } else {
                        state.game.message = match state.game.state {
                            GameState::RoomChoice => msg::NEED_FACE_OR_SKIP.to_string(),
                            GameState::CardInteraction => {
                                if state.game.awaiting_weapon_choice {
                                    msg::NEED_Y_OR_N.to_string()
                                } else {
                                    msg::HINT_INTERACTION_ACK.to_string()
                                }
                            }
                            GameState::MainMenu => msg::NEED_START.to_string(),
                            GameState::GameOver => msg::RESTART_HELP.to_string(),
                            GameState::CardSelection => msg::NEED_SELECT_CARD.to_string(),
                        };
                    }
                    return true;
                }
                Some(ID_CARD_2) => {
                    // Only allow clicking cards when we're actually in the selection state.
                    // If not, show state-appropriate guidance (avoid stale/incorrect MUST_FACE_FIRST).
                    if state.game.state == GameState::CardSelection {
                        let _ = state.game.play_card_from_slot(1);
                    } else {
                        state.game.message = match state.game.state {
                            GameState::RoomChoice => msg::NEED_FACE_OR_SKIP.to_string(),
                            GameState::CardInteraction => {
                                if state.game.awaiting_weapon_choice {
                                    msg::NEED_Y_OR_N.to_string()
                                } else {
                                    msg::HINT_INTERACTION_ACK.to_string()
                                }
                            }
                            GameState::MainMenu => msg::NEED_START.to_string(),
                            GameState::GameOver => msg::RESTART_HELP.to_string(),
                            GameState::CardSelection => msg::NEED_SELECT_CARD.to_string(),
                        };
                    }
                    return true;
                }
                Some(ID_CARD_3) => {
                    // Only allow clicking cards when we're actually in the selection state.
                    // If not, show state-appropriate guidance (avoid stale/incorrect MUST_FACE_FIRST).
                    if state.game.state == GameState::CardSelection {
                        let _ = state.game.play_card_from_slot(2);
                    } else {
                        state.game.message = match state.game.state {
                            GameState::RoomChoice => msg::NEED_FACE_OR_SKIP.to_string(),
                            GameState::CardInteraction => {
                                if state.game.awaiting_weapon_choice {
                                    msg::NEED_Y_OR_N.to_string()
                                } else {
                                    msg::HINT_INTERACTION_ACK.to_string()
                                }
                            }
                            GameState::MainMenu => msg::NEED_START.to_string(),
                            GameState::GameOver => msg::RESTART_HELP.to_string(),
                            GameState::CardSelection => msg::NEED_SELECT_CARD.to_string(),
                        };
                    }
                    return true;
                }
                Some(ID_CARD_4) => {
                    // Only allow clicking cards when we're actually in the selection state.
                    // If not, show state-appropriate guidance (avoid stale/incorrect MUST_FACE_FIRST).
                    if state.game.state == GameState::CardSelection {
                        let _ = state.game.play_card_from_slot(3);
                    } else {
                        state.game.message = match state.game.state {
                            GameState::RoomChoice => msg::NEED_FACE_OR_SKIP.to_string(),
                            GameState::CardInteraction => {
                                if state.game.awaiting_weapon_choice {
                                    msg::NEED_Y_OR_N.to_string()
                                } else {
                                    msg::HINT_INTERACTION_ACK.to_string()
                                }
                            }
                            GameState::MainMenu => msg::NEED_START.to_string(),
                            GameState::GameOver => msg::RESTART_HELP.to_string(),
                            GameState::CardSelection => msg::NEED_SELECT_CARD.to_string(),
                        };
                    }
                    return true;
                }
                _ => {
                    // click outside: stop drag
                    state.mouse_down = false;
                    state.dragging = false;
                    return true;
                }
            }
        }
        Event::MouseDrag { x, y: _, button: _ } => {
            if !state.mouse_down {
                return true;
            }
            state.dragging = true;
            if state.input.is_focused() {
                state.input.drag_select_to(x);
            }
            return true;
        }
        Event::MouseRelease { x, y: _, button: _ } => {
            if state.mouse_down && state.dragging && state.input.is_focused() {
                state.input.drag_select_to(x);
            }
            state.mouse_down = false;
            state.dragging = false;
            return true;
        }
        _ => {}
    }

    // Enter submits the command (modifier-aware + legacy)
    if let Event::KeyWithModifiers(k) = event {
        if matches!(k.key, KeyKind::Enter) {
            submit_command(state);
            return true;
        }
    }
    if matches!(event, Event::Enter) {
        submit_command(state);
        return true;
    }

    // Let TextInput consume typing/editing
    if state.input.handle_event(event) {
        return true;
    }

    true
}

fn submit_command(state: &mut AppState) {
    let raw = state.input.text().trim().to_string();

    // Empty Enter:
    // - Only continues in CardInteraction when NOT awaiting weapon choice
    // - Otherwise it's a no-op to avoid accidental actions
    if raw.is_empty() {
        state.input.set_text("");
        if state.game.state == GameState::CardInteraction && !state.game.awaiting_weapon_choice {
            state.game.continue_after_interaction();
        }
        return;
    }

    let cmd = raw;
    state.set_last_command_feedback(&cmd);
    state.input.set_text("");

    // Global exit/restart
    if cmd.eq_ignore_ascii_case("exit") || cmd.eq_ignore_ascii_case("quit") {
        state.should_quit = true;
        return;
    }
    if cmd.eq_ignore_ascii_case("restart") {
        state.game.reset_to_playing();
        return;
    }

    match state.game.state {
        GameState::MainMenu => {
            if cmd.eq_ignore_ascii_case("start") || cmd.eq_ignore_ascii_case("s") {
                state.game.state = GameState::RoomChoice;
                state.game.fill_room();
                state.game.message = msg::ENTERED_DUNGEON.to_string();
            } else {
                state.game.message = msg::NEED_START.to_string();
            }
        }

        GameState::RoomChoice => {
            // Accept either the short forms (f/s) or the clearer words (face/skip)
            if cmd.eq_ignore_ascii_case("f") || cmd.eq_ignore_ascii_case("face") {
                state.game.face_room();
            } else if cmd.eq_ignore_ascii_case("s") || cmd.eq_ignore_ascii_case("skip") {
                state.game.skip_room();
            } else if state.game.can_skip {
                state.game.message = msg::NEED_FACE_OR_SKIP.to_string();
            } else {
                state.game.message = msg::NEED_FACE_ONLY.to_string();
            }
        }

        GameState::CardSelection => {
            if let Ok(n) = cmd.parse::<usize>() {
                let idx = n.saturating_sub(1);
                let _ = state.game.play_card_from_slot(idx);
            } else {
                state.game.message = msg::NEED_SELECT_CARD.to_string();
            }
        }

        GameState::CardInteraction => {
            if state.game.awaiting_weapon_choice {
                if cmd.eq_ignore_ascii_case("y") {
                    let _ = state.game.answer_weapon_prompt(true);
                } else if cmd.eq_ignore_ascii_case("n") {
                    let _ = state.game.answer_weapon_prompt(false);
                } else {
                    state.game.message = msg::NEED_Y_OR_N.to_string();
                }
            } else if cmd.eq_ignore_ascii_case("ok") {
                state.game.continue_after_interaction();
            } else {
                // Ignore other commands during acknowledgement step
            }
        }

        GameState::GameOver => {
            // Non-global commands in GameOver just show help
            state.game.message = msg::RESTART_HELP.to_string();
        }
    }

    // Death check safeguard (some sequences may reduce HP outside continue)
    if state.game.health <= 0 && state.game.state != GameState::GameOver {
        state.game.survived = false;
        state.game.state = GameState::GameOver;
        state.game.message = msg::YOU_DIED.to_string();
    }
}

// ==============================
// Draw
// ==============================

pub fn draw(state: &mut AppState, window: &mut dyn Window) -> minui::Result<()> {
    let (w, h) = window.get_size();

    // New immediate-mode scene frame: clears registrations
    state.ui.begin_frame();

    // Cursor is applied at end_frame
    window.clear_cursor_request();

    // Clear full screen
    if h > 0 && w > 0 {
        window.clear_area(0, 0, h.saturating_sub(1), w.saturating_sub(1))?;
    }

    // Root container (whole game UI)
    let margin: u16 = 1;
    let root_x = margin;
    let root_y = margin;
    let root_w = w.saturating_sub(margin * 2).max(1);
    let root_h = h.saturating_sub(margin * 2).max(1);

    let root_panel = Container::new()
        .with_position_and_size(root_x, root_y, root_w, root_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::double_line())
        .with_border_color(ColorPair::new(Color::White, Color::Transparent))
        .with_title("Scoundrel")
        .with_title_alignment(TitleAlignment::Center)
        .with_padding(ContainerPadding::uniform(0));
    root_panel.draw(window)?;

    // Inner content area (inside root border)
    let inner_x = root_x + 1;
    let inner_y = root_y + 1;
    let inner_w = root_w.saturating_sub(2).max(1);

    // Fixed panel heights (stable layout)
    let status_h: u16 = 5;
    let room_h: u16 = 6;
    let msg_h: u16 = 5;
    let cmd_h: u16 = 3;

    // Shared geometry
    let content_x = inner_x + 1;

    // ==============================
    // Status panel
    // ==============================
    let status_y = inner_y;

    Container::new()
        .with_position_and_size(inner_x, status_y, inner_w, status_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::DarkGray, Color::Transparent))
        .with_title("Status")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0))
        .draw(window)?;

    // Health line + color
    let hp_line = health_line(state.game.health, state.game.max_health);
    window.write_str_colored(
        status_y + 1,
        content_x,
        &hp_line,
        health_color(state.game.health),
    )?;

    // Weapon + deck lines
    let weapon = weapon_line(state.game.weapon, state.game.last_monster_slain_with_weapon);
    window.write_str(status_y + 2, content_x, &weapon)?;

    let deck_line = format!("Cards left in Dungeon: {}", state.game.deck.len());
    window.write_str(status_y + 3, content_x, &deck_line)?;

    // ==============================
    // Dungeon room panel
    // ==============================
    let room_y = status_y + status_h + 1;

    Container::new()
        .with_position_and_size(inner_x, room_y, inner_w, room_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::LightBlue, Color::Transparent))
        .with_title("Dungeon Room")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0))
        .draw(window)?;

    // Cards (stable slots)
    let card_area_x = inner_x + 1;
    let card_area_y = room_y + 1;

    let card_w: u16 = ((inner_w.saturating_sub(5)) / 4).max(10);
    let card_h: u16 = 3;
    let gap: u16 = 1;

    for i in 0..4usize {
        let x = card_area_x + (card_w + gap) * (i as u16);
        let y0 = card_area_y;

        let id = match i {
            0 => ID_CARD_1,
            1 => ID_CARD_2,
            2 => ID_CARD_3,
            _ => ID_CARD_4,
        };

        Container::new()
            .with_position_and_size(x, y0, card_w, card_h)
            .with_layout_direction(LayoutDirection::Vertical)
            .with_border()
            .with_border_chars(BorderChars::single_line())
            .with_border_color(ColorPair::new(Color::DarkGray, Color::Transparent))
            .with_padding(ContainerPadding::uniform(0))
            .draw(window)?;

        let (label, colors) = match state.game.room_slots[i] {
            Some(c) => (format!("[{}] {}", i + 1, card_text(c)), card_color(c)),
            None => (
                "[ ] empty".to_string(),
                ColorPair::new(Color::DarkGray, Color::Transparent),
            ),
        };

        window.write_str_colored(y0 + 1, x + 1, &label, colors)?;

        // Click hitbox
        state.ui.cache_mut().register(
            id,
            WidgetArea {
                x,
                y: y0,
                width: card_w,
                height: card_h,
            },
        );
    }

    // Room footer
    let footer = match state.game.state {
        GameState::CardSelection => Some(format!(
            "Interactions left in this room: {}",
            state.game.interactions_left_in_room
        )),
        _ => None,
    };

    if let Some(footer) = footer {
        window.write_str_colored(
            room_y + 4,
            content_x,
            &footer,
            ColorPair::new(Color::DarkGray, Color::Transparent),
        )?;
    }

    // ==============================
    // Message panel (hint + message + previous input / score)
    // ==============================
    let msg_y = room_y + room_h + 1;

    Container::new()
        .with_position_and_size(inner_x, msg_y, inner_w, msg_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::DarkGray, Color::Transparent))
        .with_title("Message")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0))
        .draw(window)?;

    // Hint line in message box
    let hint = state_hint(&state.game);
    window.write_str_colored(
        msg_y + 1,
        content_x,
        hint,
        ColorPair::new(Color::DarkGray, Color::Transparent),
    )?;

    let message = if state.game.message.is_empty() {
        match state.game.state {
            GameState::MainMenu => "Welcome, Scoundrel.".to_string(),
            GameState::RoomChoice => msg::NEED_FACE_OR_SKIP.to_string(),
            GameState::CardSelection => "Choose a card.".to_string(),
            GameState::CardInteraction => {
                if state.game.awaiting_weapon_choice {
                    msg::NEED_Y_OR_N.to_string()
                } else {
                    msg::HINT_INTERACTION_ACK.to_string()
                }
            }
            GameState::GameOver => state.game.remaining_summary_line(),
        }
    } else {
        state.game.message.clone()
    };

    window.write_str(msg_y + 2, content_x, &message)?;

    // Previous input / score line directly under message (no extra blank line)
    if state.game.state == GameState::GameOver {
        let score_line = format!("FINAL SCORE: {}", state.game.final_score());
        window.write_str_colored(
            msg_y + 3,
            content_x,
            &score_line,
            ColorPair::new(Color::White, Color::Transparent),
        )?;
    } else if !state.game.last_command_feedback.is_empty() {
        window.write_str_colored(
            msg_y + 3,
            content_x,
            &state.game.last_command_feedback,
            ColorPair::new(Color::DarkGray, Color::Transparent),
        )?;
    }

    // ==============================
    // Command panel + TextInput
    // ==============================
    let cmd_y = msg_y + msg_h + 1;

    Container::new()
        .with_position_and_size(inner_x, cmd_y, inner_w, cmd_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::White, Color::Transparent))
        .with_title("Command")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0))
        .draw(window)?;

    let input_x = content_x;
    let input_y = cmd_y + 1;
    let input_w = inner_w.saturating_sub(2).max(10);

    let input_widget = TextInput::new()
        .with_position(input_x, input_y)
        .with_width(input_w)
        .with_border(true)
        .with_placeholder(command_placeholder(&state.game));

    input_widget.draw_with_id(window, &mut state.input, state.ui.cache_mut(), ID_INPUT)?;

    // Draw tooltips (rendered last to appear on top. I'll add proper z-ordering to MinUI soon!)
    for i in 0..4usize {
        if let Some(card) = state.game.room_slots[i] {
            if state.card_hovers[i].should_show_tooltip(Duration::from_millis(300)) {
                let tooltip_text = card_tooltip_text(card, &state.game);
                let tooltip = Tooltip::new(&tooltip_text)
                    .with_delay(Duration::from_millis(200))
                    .with_color(ColorPair::new(Color::LightGray, Color::DarkGray));

                let (tooltip_x, tooltip_y) =
                    tooltip.position_near_mouse(state.mouse_pos.0, state.mouse_pos.1, w, h);

                tooltip.draw_at(window, tooltip_x, tooltip_y)?;
            }
        }
    }

    // End frame applies cursor request
    window.end_frame()?;
    Ok(())
}

fn state_hint(game: &Game) -> &'static str {
    match game.state {
        GameState::MainMenu => msg::HINT_MAIN,
        GameState::RoomChoice => {
            if game.can_skip {
                msg::HINT_ROOM_CHOICE_CAN_SKIP
            } else {
                msg::HINT_ROOM_CHOICE_NO_SKIP
            }
        }
        GameState::CardSelection => msg::HINT_CARD_SELECTION,
        GameState::CardInteraction => {
            if game.awaiting_weapon_choice {
                msg::HINT_PROMPT_WEAPON
            } else {
                msg::HINT_INTERACTION_ACK
            }
        }
        GameState::GameOver => msg::HINT_GAME_OVER,
    }
}

fn card_tooltip_text(card: crate::logic::Card, game: &Game) -> String {
    match card.suit {
        'S' | 'C' => {
            let base_damage = card.value as i32;

            if let Some(weapon) = game.weapon {
                if game.can_use_weapon_on(card) {
                    let weapon_value = weapon.value as i32;
                    let damage = (base_damage - weapon_value).max(0);
                    format!(
                        "Monster (ATK {}) - With weapon: {} damage",
                        base_damage, damage
                    )
                } else {
                    //let limit = game.last_monster_slain_with_weapon.unwrap_or(0);
                    format!(
                        "Monster (ATK {}) - Weapon degraded. Will take {} damage",
                        base_damage, base_damage
                    )
                }
            } else {
                format!("Monster (ATK {})", base_damage)
            }
        }
        'D' => {
            let weapon_value = card.value as i32;
            let limit_text = game
                .last_monster_slain_with_weapon
                .map(|l| format!(" (updates to < {})", l))
                .unwrap_or_else(|| " (no restriction)".to_string());

            format!("Weapon (ATK {}){}", weapon_value, limit_text)
        }
        'H' => {
            let heal_amount = card.value as i32;
            format!("Potion (Heal for {})", heal_amount)
        }
        _ => "Unknown card".to_string(),
    }
}
