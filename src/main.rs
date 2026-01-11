mod logic;
mod messages;
mod render;
mod ui;

use minui::prelude::*;
use std::time::Duration;

fn main() -> minui::Result<()> {
    let initial = ui::AppState::new();

    let mut app = App::new(initial)?.with_frame_rate(Duration::from_millis(16));

    // Avoid noisy mouse-move spam; click/drag events still work
    app.window_mut().mouse_mut().set_movement_tracking(false);

    app.run(ui::update, ui::draw)?;

    Ok(())
}
