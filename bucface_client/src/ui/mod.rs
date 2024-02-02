mod entry_block;
mod log_block;
mod main_window;
use main_window::main_window;
use ratatui::Frame;

use crate::app::App;

pub fn window_handler<'a>(frame: &mut Frame<'a>, app: &App<'a>) {
    main_window(frame, app);
}
