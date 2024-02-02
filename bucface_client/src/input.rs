use std::time::Duration;

use crossterm::event::{self, poll, Event, KeyCode, KeyEventKind};

use crate::app::{App, AppMode};

pub fn key_handler(app: &mut App) {
    if let Ok(true) = poll(Duration::from_millis(10)) {
        match app.mode {
            AppMode::Entry => logging_key_handler(app),
            AppMode::Logging => logging_key_handler(app),
            AppMode::Normal => normal_key_handler(app),
            AppMode::Quitting => {}
        }
    }
}

fn logging_key_handler(app: &mut App) {
    if let Ok(Event::Key(key)) = event::read() {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Normal;
            }
            KeyCode::Enter => {
                app.send_buf();
            }
            KeyCode::Backspace => {
                app.buf.pop();
            }
            KeyCode::Char(c) => {
                app.buf.push(c as u8);
            }
            _ => {}
        }
    }
}

fn normal_key_handler(app: &mut App) {
    if let Ok(Event::Key(key)) = event::read() {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                app.mode = AppMode::Quitting;
            }
            KeyCode::Char('e') => {
                app.mode = AppMode::Logging;
            }
            KeyCode::Char('s') => {
                app.mode = AppMode::Entry;
            }
            _ => {}
        }
    }
}
