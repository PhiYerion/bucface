mod app;
mod input;
mod ui;
use std::io::stdout;

use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;

use self::app::App;
use self::input::key_handler;

#[allow(unreachable_code)]
fn main() -> std::io::Result<()> {
    env_logger::init();
    log::debug!("Starting up");
    let mut terminal = setup(CrosstermBackend::new(stdout()))?;
    let mut app = App::default();
    log::debug!("Created app state and terminal");

    loop {
        let _ = key_handler(&mut app);
        if app.mode == app::AppMode::Quitting {
            log::debug!("Quitting");
            break;
        };

        let _ = terminal.draw(|f| ui::ui(f, &app));
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
