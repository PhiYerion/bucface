use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, AppMode};

pub fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    create_title(chunks[0], frame, app);
    create_events(chunks[1], frame, app);
    create_nav(chunks[2], frame, app);
}

fn create_title(chunk: Rect, frame: &mut Frame, app: &App) {
    let title_block = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        format!("{}@BucFace v0.1", app.name),
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, chunk);
}

fn create_events(chunk: Rect, frame: &mut Frame, app: &App) {
    let list_items = app.human_events.iter().map(|event| {
        let text = Text::styled(
            format!(
                "{:?} | {}@{}: {}",
                event.time, event.author, event.machine, event.event
            ),
            Style::default().fg(Color::LightGreen),
        );
        ListItem::new(text)
    });

    let list = List::new(list_items);
    frame.render_widget(list, chunk);
}

fn create_nav(chunk: Rect, frame: &mut Frame, app: &App) {
    let mode_text = Text::styled(
        app.mode.to_string(),
        Style::default().fg(match app.mode {
            AppMode::Normal => Color::LightGreen,
            AppMode::Logging => Color::Yellow,
            AppMode::Quitting => Color::Red,
        }),
    );
    let mode_block = Paragraph::new(mode_text)
        .block(Block::default().borders(Borders::TOP))
        .alignment(ratatui::layout::Alignment::Center);

    let help_text = Text::styled(
        "Press 'q' to quit, 'e' to start editing, 's' to send, 'c' to clear".to_string(),
        Style::default().fg(Color::LightGreen),
    );
    let help_block = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .alignment(ratatui::layout::Alignment::Center);

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunk);

    frame.render_widget(mode_block, footer_chunks[0]);
    frame.render_widget(help_block, footer_chunks[1]);
}
