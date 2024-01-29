use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

use super::centered_rect;

pub fn entry_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(frame.size(), 80, 60);

    let popup_block = Block::default()
        .title("Login")
        .borders(Borders::ALL)
        .style(Style::default());

    let chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .margin(2)
        .split(area);

    let prompt = Paragraph::new(Text::styled(
        "Name: ",
        Style::default().fg(Color::White),
    )).alignment(Alignment::Right);

    let text = Paragraph::new(Text::styled(
        app.buf.iter().collect::<String>(),
        Style::default().fg(Color::LightGreen),
    ))
    .wrap(Wrap::default());

    frame.render_widget(popup_block, area);
    frame.render_widget(prompt, chunk[0]);
    frame.render_widget(text, chunk[1]);
}
