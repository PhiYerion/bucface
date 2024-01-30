use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::app::{App, AppMode};

pub async fn key_handler<'a>(app: &App<'a>) -> std::io::Result<()> {
    match app.mode {
        AppMode::Entry => logging_key_handler(app).await?,
        AppMode::Normal => normal_key_handler(app)?,
        AppMode::Logging => logging_key_handler(app).await?,
        AppMode::Quitting => {}
    }

    Ok(())
}

async fn logging_key_handler<'a>(app: &App<'a>) -> std::io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                *app.mode.lock().unwrap() = AppMode::Normal.into();
                return Ok(());
            }
            KeyCode::Enter => {
                return app.send_buf().await;
            }
            KeyCode::Backspace => {
                app.buf.lock().unwrap().pop();
            }
            KeyCode::Char(c) => {
                app.buf.lock().unwrap().push(c as u8);
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
