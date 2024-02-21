use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

pub fn entry(area: Rect, frame: &mut Frame, app: &App) {
    let popup_block = Block::default()
        .title("Login")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Rgb(12, 4, 4)));

    let chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .margin(2)
        .split(area);

    let prompt = Paragraph::new(Text::styled("Name: ", Style::default().fg(Color::White)))
        .alignment(Alignment::Right);

    let text = Paragraph::new(Text::styled(
        app.buf.iter().map(|x| char::from(*x)).collect::<String>(),
        Style::default().fg(Color::LightGreen),
    ))
    .wrap(Wrap::default());

    frame.render_widget(popup_block, area);
    frame.render_widget(prompt, chunk[0]);
    frame.render_widget(text, chunk[1]);
}
