use chrono::Local;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;

use super::centered_rect;

pub fn log_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(frame.size(), 60, 40);

    let popup_block = Block::default()
        .title(format!(
            "{}'s Log | {:?} | word count: {}",
            app.name,
            Local::now(),
            app.buf.len()
        ))
        .borders(Borders::ALL)
        .style(Style::default());

    let chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80)])
        .margin(2)
        .split(area);

    let input_text = Paragraph::new(Text::styled(
        app.buf.iter().map(|x| char::from(*x)).collect::<String>(),
        Style::default().fg(Color::LightGreen),
    ))
    .wrap(Wrap::default());

    frame.render_widget(popup_block, area);
    frame.render_widget(input_text, chunk[0]);
}
