use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, AppMode};

pub fn key_handler(app: &mut App) -> std::io::Result<()> {
    match app.mode {
        AppMode::Entry => logging_key_handler(app)?,
        AppMode::Normal => normal_key_handler(app)?,
        AppMode::Logging => logging_key_handler(app)?,
        AppMode::Quitting => {}
    }

    Ok(())
}

fn logging_key_handler(app: &mut App) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Normal;
                return Ok(());
            }
            KeyCode::Enter => {
                return app.send_buf();
            }
            KeyCode::Backspace => {
                app.buf.pop();
            }
            KeyCode::Char(c) => {
                app.buf.push(c);
            }
            _ => {}
        }
    }

    Ok(())
}

fn normal_key_handler(app: &mut App) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') => {
                app.mode = AppMode::Quitting;
            }
            KeyCode::Char('e') => {
                app.mode = AppMode::Logging;
            }
            _ => {}
        }
    }

    Ok(())
}
