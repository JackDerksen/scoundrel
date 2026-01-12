//! Shared user-facing strings

/// Hint/help lines shown in the Message panel (top line)
pub const HINT_MAIN: &str = "Main menu: type 'start' then 'enter'.";
pub const HINT_ROOM_CHOICE_CAN_SKIP: &str = "Room: 'f' to face • 's' to skip.";
pub const HINT_ROOM_CHOICE_NO_SKIP: &str = "Room: 'f' to face (skip already used).";
pub const HINT_CARD_SELECTION: &str = "Select: click a card, or type 1-4 then 'enter'.";
pub const HINT_PROMPT_WEAPON: &str = "Prompt: type 'y' or 'n' then 'enter'.";
pub const HINT_INTERACTION_ACK: &str = "Battle won. Press 'enter' to continue.";
pub const HINT_GAME_OVER: &str = "Game over: type 'restart' to play again, or Ctrl+Q to quit.";

/// Common state/status messages
pub const ENTERED_DUNGEON: &str = "Entered the dungeon.";
pub const FACE_ROOM: &str = "Facing the room. Choose a card.";
pub const SKIPPED_ROOM: &str = "Skipped the room.";
pub const ROOM_RESOLVED: &str = "Room resolved. Face or skip the next room.";
pub const YOU_SURVIVED: &str = "You survived the dungeon!";
pub const YOU_DIED: &str = "You succumbed to the dungeon's monsters.";

/// Validation / guidance messages
pub const NEED_START: &str = "Type 'start' then 'enter'.";
pub const NEED_FACE_OR_SKIP: &str = "Type 'f' to face, or 's' to skip.";
pub const NEED_FACE_ONLY: &str = "Type 'f' to face (skip already used).";
pub const NEED_SELECT_CARD: &str = "Type 1-4 to select a card, or click a card.";
pub const INVALID_CARD_SELECTION: &str = "Invalid card selection.";
pub const MUST_FACE_FIRST: &str = "You must face the room ('f') before selecting.";
pub const NEED_Y_OR_N: &str = "Type 'y' or 'n' then 'enter'.";
pub const RESTART_HELP: &str = "Type 'restart' to play again, 'exit' to quit, or Ctrl+Q.";

pub const CMD_PREFIX: &str = "> ";
