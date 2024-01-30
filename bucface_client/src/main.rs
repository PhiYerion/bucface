mod app;
mod input;
mod logging;
mod ui;

use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::borrow::{Borrow, BorrowMut};
use std::io::stdout;
use std::sync::Arc;
use tokio::sync::Mutex;

use self::app::App;
use self::input::key_handler;

use crate::logging::initialize_logging;
use crate::ui::window_handler;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    initialize_logging().unwrap();
    let mut terminal = setup(CrosstermBackend::new(stdout()))?;
    let app = Arc::new(App::default());

    {
        let _ = terminal.draw(|f| window_handler(f, &app));
    }

    {
        tokio::spawn(async move {
            loop {
                let _ = key_handler(app).await;
            }
        });
    }

    {
        let app = app.clone();
        tokio::spawn(async move {
            loop {
                let _ = app.update().await;
            }
        });
    }

    loop {
        if app.mode == app::AppMode::Quitting {
            break;
        };

        let _ = terminal.draw(|f| window_handler(f, &mut app));
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn setup<B: Backend>(backend: B) -> std::io::Result<Terminal<B>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    Ok(terminal)
}
