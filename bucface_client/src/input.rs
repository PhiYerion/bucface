use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, AppMode};

pub fn key_handler(app: &mut App) -> std::io::Result<()> {
    match app.mode {
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
                log::debug!("Esc or 'q' pressed. Returning to normal");
                app.mode = AppMode::Normal;
                return Ok(());
            }
            KeyCode::Enter => {
                log::debug!("Enter pressed");
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
                log::debug!("'q' pressed. Quitting");
                app.mode = AppMode::Quitting;
            }
            KeyCode::Char('e') => {
                log::debug!("'e' pressed. Enabling logging");
                app.mode = AppMode::Logging;
            }
            _ => {}
        }
    }

    Ok(())
}
