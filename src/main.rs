use minui::prelude::*;
use minui::ui::UiScene;
use minui::widgets::ContainerPadding;
use minui::widgets::WidgetArea;
use minui::widgets::{TextInput, TextInputState};
use minui::Window;
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::time::Duration;

// Interaction IDs
const ID_INPUT: InteractionId = 1;
const ID_CARD_1: InteractionId = 101;
const ID_CARD_2: InteractionId = 102;
const ID_CARD_3: InteractionId = 103;
const ID_CARD_4: InteractionId = 104;

// ==============================
// Game model
// ==============================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Card {
    suit: char, // 'S', 'C', 'D', 'H'
    value: u8,  // 2-14 (ace is 14)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    MainMenu,
    RoomChoice,
    CardSelection,
    CardInteraction,
    GameOver,
}

struct Game {
    // Core game state
    deck: VecDeque<Card>,
    room: Vec<Card>,
    health: i32,
    max_health: i32,
    weapon: Option<Card>,
    last_monster_slain_with_weapon: Option<u8>,
    potion_used_this_room: bool,
    can_skip: bool,
    state: GameState,
    survived: bool,

    // Interaction prompts
    message: String,
    current_monster: Option<Card>,
    awaiting_weapon_choice: bool,

    // Turn/room tracking
    interactions_left_in_room: u8,

    // UI state
    ui: UiScene,
    input: TextInputState,
    last_command_feedback: String,

    // Mouse drag selection (for TextInput)
    mouse_down: bool,
    dragging: bool,
}

impl Game {
    fn new() -> Self {
        let mut input = TextInputState::new();
        input.set_focused(true);

        let mut g = Game {
            deck: VecDeque::new(),
            room: Vec::new(),
            health: 20,
            max_health: 20,
            weapon: None,
            last_monster_slain_with_weapon: None,
            potion_used_this_room: false,
            can_skip: true,
            state: GameState::MainMenu,
            survived: false,

            message: String::new(),
            current_monster: None,
            awaiting_weapon_choice: false,

            interactions_left_in_room: 0,

            ui: UiScene::new(),
            input,
            last_command_feedback: String::new(),

            mouse_down: false,
            dragging: false,
        };

        g.create_deck();
        g
    }

    fn reset(&mut self) {
        *self = Game::new();
        self.state = GameState::RoomChoice;
        self.fill_room();
        self.message = "Entered the dungeon.".to_string();
    }

    fn create_deck(&mut self) {
        let mut cards = Vec::new();
        for suit in ['S', 'C', 'D', 'H'] {
            for value in 2..=14u8 {
                // Red aces and face cards removed from the deck because them's da rulez
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

    fn fill_room(&mut self) {
        while self.room.len() < 4 && !self.deck.is_empty() {
            if let Some(c) = self.deck.pop_front() {
                self.room.push(c);
            }
        }
    }

    fn card_text(card: Card) -> String {
        let v = match card.value {
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            14 => "A".to_string(),
            _ => card.value.to_string(),
        };
        let s = match card.suit {
            // Hears render weirdly in Ghostty, might try to fix later, idk
            'S' => "♠",
            'C' => "♣",
            'D' => "♦",
            'H' => "",
            _ => "?",
        };
        format!("{v}{s}")
    }

    fn card_color(card: Card) -> ColorPair {
        match card.suit {
            'D' | 'H' => ColorPair::new(Color::LightRed, Color::Transparent),
            _ => ColorPair::new(Color::White, Color::Transparent),
        }
    }

    fn health_color(&self) -> ColorPair {
        let fg = if self.health > 10 {
            Color::Green
        } else if self.health > 5 {
            Color::Yellow
        } else {
            Color::Red
        };
        ColorPair::new(fg, Color::Transparent)
    }

    fn draw_health_bar(&self) -> String {
        let hp = self.health.max(0).min(self.max_health);
        let filled = "█".repeat(hp as usize);
        let empty = "░".repeat((self.max_health - hp) as usize);
        format!("{filled}{empty}")
    }

    fn remaining_summary_line(&self) -> String {
        let remaining: Vec<Card> = self
            .room
            .iter()
            .copied()
            .chain(self.deck.iter().copied())
            .collect();

        let monsters: Vec<Card> = remaining
            .iter()
            .copied()
            .filter(|c| c.suit == 'S' || c.suit == 'C')
            .collect();

        if monsters.is_empty() {
            return "No monsters remained. You cleared them all!".to_string();
        }

        let total_threat: i32 = monsters.iter().map(|m| m.value as i32).sum();
        format!("Remaining monsters total threat: -{total_threat}")
    }

    fn final_score(&self) -> i32 {
        if self.survived {
            self.health
        } else {
            let remaining: Vec<Card> = self
                .room
                .iter()
                .copied()
                .chain(self.deck.iter().copied())
                .collect();
            let monsters_sum: i32 = remaining
                .iter()
                .filter(|c| c.suit == 'S' || c.suit == 'C')
                .map(|c| c.value as i32)
                .sum();
            -monsters_sum
        }
    }

    fn can_use_weapon_on(&self, monster: Card) -> bool {
        if self.weapon.is_none() {
            return false;
        }
        match self.last_monster_slain_with_weapon {
            None => true,
            Some(last) => monster.value < last,
        }
    }

    fn handle_monster_with_weapon(&mut self, monster: Card) -> i32 {
        if let Some(w) = self.weapon {
            let dmg = (monster.value as i32 - w.value as i32).max(0);
            self.last_monster_slain_with_weapon = Some(monster.value);
            dmg
        } else {
            monster.value as i32
        }
    }

    fn handle_monster_without_weapon(&self, monster: Card) -> i32 {
        monster.value as i32
    }

    fn apply_card_selection(&mut self, idx: usize) {
        if self.state != GameState::CardSelection {
            self.message = "You must face the room ('f') before selecting.".to_string();
            return;
        }
        if idx >= self.room.len() {
            self.message = "Invalid card selection.".to_string();
            return;
        }

        let card = self.room.remove(idx);

        match card.suit {
            'S' | 'C' => {
                // Monster
                self.current_monster = Some(card);
                if self.can_use_weapon_on(card) {
                    let w = self.weapon.unwrap();
                    self.message = format!("Use your weapon, {}? (y/n)", Game::card_text(w));
                    self.awaiting_weapon_choice = true;
                    self.state = GameState::CardInteraction;
                } else {
                    let dmg = self.handle_monster_without_weapon(card);
                    self.health -= dmg;
                    self.message = format!("Fought the monster! Took {dmg} damage.");
                    self.state = GameState::CardInteraction;
                    self.after_one_interaction_resolve();
                }
            }
            'D' => {
                // Weapon
                self.weapon = Some(card);
                self.last_monster_slain_with_weapon = None;
                self.message = format!("Equipped a {}!", Game::card_text(card));
                self.state = GameState::CardInteraction;
                self.after_one_interaction_resolve();
            }
            'H' => {
                // Potion
                if !self.potion_used_this_room {
                    let heal = card.value as i32;
                    self.health = (self.health + heal).min(self.max_health);
                    self.potion_used_this_room = true;
                    self.message = format!("Healed for {heal} HP.");
                } else {
                    self.message = "Potion wasted (only allowed 1 per room).".to_string();
                }
                self.state = GameState::CardInteraction;
                self.after_one_interaction_resolve();
            }
            _ => {
                self.message = "Unknown card.".to_string();
                self.state = GameState::CardInteraction;
                self.after_one_interaction_resolve();
            }
        }
    }

    fn after_one_interaction_resolve(&mut self) {
        if self.health <= 0 {
            self.survived = false;
            self.state = GameState::GameOver;
            self.message = "You succumbed to the dungeon's monsters...".to_string();
            return;
        }

        if self.interactions_left_in_room > 0 {
            self.interactions_left_in_room -= 1;
        }

        if self.interactions_left_in_room == 0 {
            // End of the faced-room interaction window
            self.can_skip = true;
            self.fill_room();
            if self.room.is_empty() && self.deck.is_empty() {
                self.survived = true;
                self.state = GameState::GameOver;
                self.message = "You survived the dungeon!".to_string();
            } else {
                self.state = GameState::RoomChoice;
                self.message = "Room resolved. Face or skip the next room.".to_string();
            }
            return;
        }

        if self.room.is_empty() && self.deck.is_empty() {
            self.survived = true;
            self.state = GameState::GameOver;
            self.message = "You survived the dungeon!".to_string();
            return;
        }

        // Continue selection within this room
        self.state = GameState::CardSelection;
    }

    fn face_room(&mut self) {
        self.can_skip = true;
        self.potion_used_this_room = false;
        self.interactions_left_in_room = 3;
        self.state = GameState::CardSelection;
        self.message = "Facing the room. Choose a card.".to_string();
    }

    fn skip_room(&mut self) {
        if !self.can_skip {
            self.message = "Skip already used; you must face this room.".to_string();
            return;
        }
        let drained: Vec<Card> = self.room.drain(..).collect();
        self.deck.extend(drained);
        self.can_skip = false;

        self.fill_room();
        if self.room.is_empty() && self.deck.is_empty() {
            self.survived = true;
            self.state = GameState::GameOver;
            self.message = "You survived the dungeon!".to_string();
        } else {
            self.message = "Skipped the room... for now.".to_string();
        }
    }

    fn continue_from_interaction(&mut self) {
        // Only used when we were waiting for an explicit "continue" step (weapon prompt handled elsewhere).
        // In this implementation, interactions auto-advance when resolved (except weapon prompt),
        // so this is mostly a no-op safety.
        if self.state == GameState::CardInteraction && !self.awaiting_weapon_choice {
            if self.health <= 0 {
                self.survived = false;
                self.state = GameState::GameOver;
            } else if self.state != GameState::GameOver {
                // If we somehow landed here, return to selection if still in room window.
                if self.interactions_left_in_room > 0 {
                    self.state = GameState::CardSelection;
                } else {
                    self.state = GameState::RoomChoice;
                }
            }
        }
    }
}

// ==============================
// App entry
// ==============================
fn main() -> minui::Result<()> {
    let initial = Game::new();

    // Use a frame rate so mouse/keyboard integration stays responsive on backends that emit Frame.
    let mut app = App::new(initial)?.with_frame_rate(Duration::from_millis(16));

    // Reduce noisy mouse-move events (optional).
    app.window_mut().mouse_mut().set_movement_tracking(true);

    app.run(update, draw)?;
    Ok(())
}

// ==============================
// Update loop
// ==============================
fn update(game: &mut Game, event: Event) -> bool {
    // Quit
    match event {
        Event::KeyWithModifiers(k) if matches!(k.key, KeyKind::Char('q')) => return false,
        Event::Character('q') => return false,
        _ => {}
    }

    // Apply scene policies (focus/capture bookkeeping)
    let _effects = game.ui.apply_policies(&event);

    // Mouse: click-to-focus input or click-to-select card
    match event {
        Event::MouseClick { x, y, button: _ } => {
            game.mouse_down = true;
            game.dragging = false;

            let hit = game.ui.hit_test_id(x, y);
            match hit {
                Some(ID_INPUT) => {
                    game.input.set_focused(true);
                    game.input.click_set_cursor(x);
                    return true;
                }
                Some(ID_CARD_1) => {
                    game.apply_card_selection(0);
                    return true;
                }
                Some(ID_CARD_2) => {
                    game.apply_card_selection(1);
                    return true;
                }
                Some(ID_CARD_3) => {
                    game.apply_card_selection(2);
                    return true;
                }
                Some(ID_CARD_4) => {
                    game.apply_card_selection(3);
                    return true;
                }
                _ => {
                    // Click outside: stop drag
                    game.mouse_down = false;
                    game.dragging = false;
                    return true;
                }
            }
        }
        Event::MouseDrag { x, y: _, button: _ } => {
            if !game.mouse_down {
                return true;
            }
            game.dragging = true;
            if game.input.is_focused() {
                game.input.drag_select_to(x);
            }
            return true;
        }
        Event::MouseRelease { x, y: _, button: _ } => {
            if game.mouse_down && game.dragging && game.input.is_focused() {
                game.input.drag_select_to(x);
            }
            game.mouse_down = false;
            game.dragging = false;
            return true;
        }
        _ => {}
    }

    // 'Enter' submits the command (modifier-aware and legacy fallbacks)
    if let Event::KeyWithModifiers(k) = event {
        if matches!(k.key, KeyKind::Enter) {
            submit_command(game);
            return true;
        }
    }
    if matches!(event, Event::Enter) {
        submit_command(game);
        return true;
    }

    // Let TextInput consume typing/editing
    let consumed = game.input.handle_event(event);
    if consumed {
        return true;
    }

    true
}

fn submit_command(game: &mut Game) {
    let raw = game.input.text().trim().to_string();
    let cmd = if raw.is_empty() {
        "ok".to_string()
    } else {
        raw
    };

    game.last_command_feedback = format!("> {}", cmd);
    game.input.set_text("");

    // Global commands
    if cmd.eq_ignore_ascii_case("restart") {
        game.reset();
        return;
    }

    match game.state {
        GameState::MainMenu => {
            if cmd.eq_ignore_ascii_case("start") || cmd.eq_ignore_ascii_case("s") {
                game.state = GameState::RoomChoice;
                game.fill_room();
                game.message = "Entered the dungeon.".to_string();
            } else {
                game.message = "Type 'start' then hit 'enter'.".to_string();
            }
        }

        GameState::RoomChoice => {
            if cmd.eq_ignore_ascii_case("f") {
                game.face_room();
            } else if cmd.eq_ignore_ascii_case("s") {
                game.skip_room();
            } else {
                if game.can_skip {
                    game.message = "Type 'f' to face, or 's' to skip.".to_string();
                } else {
                    game.message = "Type 'f' to face (skip already used).".to_string();
                }
            }
        }

        GameState::CardSelection => {
            if let Ok(n) = cmd.parse::<usize>() {
                let idx = n.saturating_sub(1);
                game.apply_card_selection(idx);
            } else {
                game.message = "Type 1-4 to select a card, or click a card.".to_string();
            }
        }

        GameState::CardInteraction => {
            if game.awaiting_weapon_choice {
                if cmd.eq_ignore_ascii_case("y") {
                    if let Some(monster) = game.current_monster {
                        let dmg = game.handle_monster_with_weapon(monster);
                        game.health -= dmg;
                        game.message = format!("Fought with weapon! Took {dmg} damage.");
                    }
                    game.awaiting_weapon_choice = false;
                    game.current_monster = None;
                    game.after_one_interaction_resolve();
                } else if cmd.eq_ignore_ascii_case("n") {
                    if let Some(monster) = game.current_monster {
                        let dmg = game.handle_monster_without_weapon(monster);
                        game.health -= dmg;
                        game.message = format!("Fought monster! Took {dmg} damage.");
                    }
                    game.awaiting_weapon_choice = false;
                    game.current_monster = None;
                    game.after_one_interaction_resolve();
                } else {
                    game.message = "Type 'y' or 'n' then 'enter'.".to_string();
                }
            } else {
                // Continue/ack
                game.continue_from_interaction();
            }
        }

        GameState::GameOver => {
            if cmd.eq_ignore_ascii_case("restart") {
                game.reset();
            } else {
                game.message = "Type 'restart' to play again, or press 'q' to quit.".to_string();
            }
        }
    }

    if game.health <= 0 && game.state != GameState::GameOver {
        game.survived = false;
        game.state = GameState::GameOver;
        game.message = "You succumbed to the dungeon's monsters.".to_string();
    }
}

// ==============================
// Draw
// ==============================
fn draw(game: &mut Game, window: &mut dyn Window) -> minui::Result<()> {
    let (w, h) = window.get_size();

    // New immediate-mode scene frame: clears registrations
    game.ui.begin_frame();

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

    // Absolute layout: we still need this because TextInput must be drawn with an explicit position,
    // and cards must register hitboxes for mouse selection.
    let status_h: u16 = 5;
    let room_h: u16 = 8;
    let msg_h: u16 = 5;
    let cmd_h: u16 = 3;

    // Root chrome via Container (nice title + border)
    let root_panel = Container::new()
        .with_position_and_size(root_x, root_y, root_w, root_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::double_line())
        .with_border_color(ColorPair::new(Color::LightGray, Color::Transparent))
        .with_title("Scoundrel")
        .with_title_alignment(TitleAlignment::Center)
        .with_padding(ContainerPadding::uniform(0));
    root_panel.draw(window)?;

    // Inner content area (inside root border)
    let inner_x = root_x.saturating_add(1);
    let inner_y = root_y.saturating_add(1);
    let inner_w = root_w.saturating_sub(2).max(1);
    let _inner_h = root_h.saturating_sub(2).max(1);

    let hint = match game.state {
        GameState::MainMenu => "Main menu: type 'start' then 'enter'.",
        GameState::RoomChoice => {
            if game.can_skip {
                "Room: 'f' face • 's' skip"
            } else {
                "Room: 'f' face (skip already used)"
            }
        }
        GameState::CardSelection => "Select: click a card, or type 1-4 then 'enter'.",
        GameState::CardInteraction => {
            if game.awaiting_weapon_choice {
                "Prompt: type 'y' or 'n' then 'enter'."
            } else {
                "Resolved: type 'ok' to continue"
            }
        }
        GameState::GameOver => "Game over: type 'restart' to play again, or 'q' to quit",
    };
    window.write_str_colored(
        inner_y,
        inner_x,
        hint,
        ColorPair::new(Color::DarkGray, Color::Transparent),
    )?;

    // Shared geometry
    let content_x = inner_x + 1;
    let content_w = inner_w.saturating_sub(2).max(1);

    // --- Status Container ---
    let status_y = inner_y;
    let status_panel = Container::new()
        .with_position_and_size(inner_x, status_y, inner_w, status_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::DarkGray, Color::Transparent))
        .with_title("Status")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0));
    status_panel.draw(window)?;

    let hp_bar = game.draw_health_bar();
    let hp_line = format!("Health: {}/{} |{}|", game.health, game.max_health, hp_bar);
    window.write_str_colored(
        status_y + 1,
        content_x,
        &fit_to_cells(&hp_line, content_w, TabPolicy::SingleCell, true),
        game.health_color(),
    )?;

    let weapon_line = if let Some(wpn) = game.weapon {
        let limit = if let Some(l) = game.last_monster_slain_with_weapon {
            format!(" (must be < {l})")
        } else {
            String::new()
        };
        format!("Weapon: {}{}", Game::card_text(wpn), limit)
    } else {
        "Weapon: None".to_string()
    };
    window.write_str(
        status_y + 2,
        content_x,
        &fit_to_cells(&weapon_line, content_w, TabPolicy::SingleCell, true),
    )?;

    let deck_line = format!("Cards left in dungeon: {}", game.deck.len());
    window.write_str(
        status_y + 3,
        content_x,
        &fit_to_cells(&deck_line, content_w, TabPolicy::SingleCell, true),
    )?;

    // --- Dungeon Room Container ---
    let room_y = status_y + status_h + 1;
    let room_panel = Container::new()
        .with_position_and_size(inner_x, room_y, inner_w, room_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::LightBlue, Color::Transparent))
        .with_title("Dungeon Room")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0));
    room_panel.draw(window)?;

    // Render 4 card "buttons" manually and register hitboxes for mouse selection.
    let card_area_x = inner_x + 1;
    let card_area_y = room_y + 1;

    let card_w: u16 = ((inner_w.saturating_sub(5)) / 4).max(8);
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

        let (title, colors) = if i < game.room.len() {
            let c = game.room[i];
            (
                format!("[{}] {}", i + 1, Game::card_text(c)),
                Game::card_color(c),
            )
        } else {
            (
                "[ ] empty".to_string(),
                ColorPair::new(Color::DarkGray, Color::Transparent),
            )
        };

        // Card chrome via Container (proper top border rendering)
        let card_panel = Container::new()
            .with_position_and_size(x, y0, card_w, card_h)
            .with_layout_direction(LayoutDirection::Vertical)
            .with_border()
            .with_border_chars(BorderChars::single_line())
            .with_border_color(ColorPair::new(Color::DarkGray, Color::Transparent))
            .with_padding(ContainerPadding::uniform(0));
        card_panel.draw(window)?;

        window.write_str_colored(
            y0 + 1,
            x + 1,
            &fit_to_cells(
                &title,
                card_w.saturating_sub(2),
                TabPolicy::SingleCell,
                true,
            ),
            colors,
        )?;

        // Register interaction rect so UiScene::hit_test_id(...) can resolve clicks.
        game.ui.cache_mut().register(
            id,
            WidgetArea {
                x,
                y: y0,
                width: card_w,
                height: card_h,
            },
        );
    }

    let room_footer = match game.state {
        GameState::CardSelection => format!(
            "Interactions left in this room: {}",
            game.interactions_left_in_room
        ),
        GameState::RoomChoice => {
            if game.can_skip {
                "Choose: f face • s skip".to_string()
            } else {
                "Choose: f face".to_string()
            }
        }
        _ => String::new(),
    };
    if !room_footer.is_empty() {
        window.write_str_colored(
            room_y + 5,
            content_x,
            &fit_to_cells(&room_footer, content_w, TabPolicy::SingleCell, true),
            ColorPair::new(Color::DarkGray, Color::Transparent),
        )?;
    }

    // --- Message Container ---
    let msg_y = room_y + room_h + 1;
    let msg_panel = Container::new()
        .with_position_and_size(inner_x, msg_y, inner_w, msg_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::single_line())
        .with_border_color(ColorPair::new(Color::DarkGray, Color::Transparent))
        .with_title("Message")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0));
    msg_panel.draw(window)?;

    let msg = if game.message.is_empty() {
        match game.state {
            GameState::MainMenu => "Welcome, Scoundrel. Type 'start' to begin.".to_string(),
            GameState::RoomChoice => "Type 'f' to face, or 's' to skip.".to_string(),
            GameState::CardSelection => "Choose a card.".to_string(),
            GameState::CardInteraction => "Resolve the interaction.".to_string(),
            GameState::GameOver => game.remaining_summary_line(),
        }
    } else {
        game.message.clone()
    };

    window.write_str(
        msg_y + 1,
        content_x,
        &fit_to_cells(&msg, content_w, TabPolicy::SingleCell, true),
    )?;

    if game.state == GameState::GameOver {
        let score_line = format!("FINAL SCORE: {}", game.final_score());
        window.write_str_colored(
            msg_y + 2,
            content_x,
            &fit_to_cells(&score_line, content_w, TabPolicy::SingleCell, true),
            ColorPair::new(Color::White, Color::Transparent),
        )?;
    } else if !game.last_command_feedback.is_empty() {
        window.write_str_colored(
            msg_y + 2,
            content_x,
            &fit_to_cells(
                &game.last_command_feedback,
                content_w,
                TabPolicy::SingleCell,
                true,
            ),
            ColorPair::new(Color::DarkGray, Color::Transparent),
        )?;
    }

    // --- Command Container + TextInput ---
    let cmd_y = msg_y + msg_h + 1;
    let cmd_panel = Container::new()
        .with_position_and_size(inner_x, cmd_y, inner_w, cmd_h)
        .with_layout_direction(LayoutDirection::Vertical)
        .with_border()
        .with_border_chars(BorderChars::double_line())
        .with_border_color(ColorPair::new(Color::White, Color::Transparent))
        .with_title("Command")
        .with_title_alignment(TitleAlignment::Left)
        .with_padding(ContainerPadding::uniform(0));
    cmd_panel.draw(window)?;

    let prefix = "Command:";
    window.write_str_colored(
        cmd_y + 1,
        content_x,
        prefix,
        ColorPair::new(Color::White, Color::Transparent),
    )?;

    let input_x = content_x + (prefix.len() as u16) + 1;
    let input_y = cmd_y + 1;
    let input_w = inner_w
        .saturating_sub(2)
        .saturating_sub(prefix.len() as u16 + 1)
        .max(10);

    let input_widget = TextInput::new()
        .with_position(input_x, input_y)
        .with_width(input_w)
        .with_border(true)
        .with_placeholder("start | f | s | 1..4 | y/n | ok | restart");

    input_widget.draw_with_id(window, &mut game.input, game.ui.cache_mut(), ID_INPUT)?;

    // Finish frame (applies cursor request from TextInput)
    window.end_frame()?;
    Ok(())
}

// ==============================
// Small helpers
// ==============================
//
// NOTE: We intentionally removed the manual `draw_box(...)` panel helper in favor of
// nested `Container` widgets so borders/titles are rendered consistently by MinUI.
//
// (Keeping this section in case you want future helpers.)
