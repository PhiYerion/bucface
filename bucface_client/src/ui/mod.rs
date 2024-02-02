mod entry_block;
mod log_block;
mod main_window;
use main_window::main_window;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::app::{App, AppMode};

pub fn window_handler<'a>(frame: &mut Frame<'a>, app: &App<'a>) {
    main_window(frame, app);
}
