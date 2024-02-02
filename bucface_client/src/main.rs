#![feature(noop_waker)]
mod app;
mod input;
mod logging;
mod net;
mod ui;

use bucface_utils::Events;
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use futures::future::FutureExt;
use parking_lot::Mutex;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io::stdout;
use std::sync::Arc;

use self::app::App;
use self::input::key_handler;

use crate::logging::initialize_logging;
use crate::ui::window_handler;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    initialize_logging().unwrap();
    let mut terminal = setup(CrosstermBackend::new(stdout()))?;
    let mut app = App::default();

    let _ = terminal.draw(|f| window_handler(f, &app));
    let new_events: Arc<Mutex<Events>> = Arc::new(Mutex::new(Events { inner: Vec::new() }));
    get_new_events(&app, new_events.clone());

    loop {
        if app.mode == app::AppMode::Quitting {
            break;
        };
        if let Some(mut events) = new_events.try_lock() {
            if !events.inner.is_empty() {
                log::debug!("Got {} new events", events.inner.len());
                log::debug!("Current events: {:?}", app.events);
                app.events.extend_from_slice(&events.inner);
                events.inner.clear();
            } else {
                get_new_events(&app, new_events.clone());
            }
        }

        key_handler(&mut app);
        let _ = terminal.draw(|f| window_handler(f, &app));
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

fn get_new_events(app: &App, new_events: Arc<Mutex<Events>>) {
    let future = app.update_logs().then(|result| async move {
        if let Ok(events) = result {
            new_events.lock().inner = events.inner;
        }
    });
    tokio::spawn(future);
}

fn setup<B: Backend>(backend: B) -> std::io::Result<Terminal<B>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    Ok(terminal)
}
