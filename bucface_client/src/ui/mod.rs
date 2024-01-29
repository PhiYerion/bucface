mod log_popup;
mod main_window;
mod entry;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use main_window::main_window;

use crate::app::{App, AppMode};

pub fn window_handler(frame: &mut Frame, app: &App) {
    main_window(frame, app);

    match app.mode {
        AppMode::Entry => entry::entry_popup(frame, app),
        AppMode::Logging => log_popup::log_popup(frame, app),
        _ => {}
    }
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
