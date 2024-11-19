use std::time::SystemTime;

use chrono::{DateTime, Local};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{networking::Message, tui::config::MessageConfig};
pub struct MessageBox {}

impl MessageBox {
    pub fn new(messages: &Vec<Message>) -> Paragraph {
        Paragraph::new(
            messages
                .into_iter()
                .map(|m| MessageBox::get_line(m))
                .collect::<Vec<Line>>()
                .clone(),
        )
        .wrap(Wrap { trim: true })
        .block(Block::bordered().title("Messages"))
    }
    fn get_line(message: &Message) -> Line {
        Line::from(vec![
            Span::styled(
                format!("[{}] ", MessageBox::time_format(message.time)),
                MessageConfig::time_style(),
            ),
            Span::styled(
                message.sender_name.clone(),
                MessageConfig::username_style(message.sent_by_self),
            ),
            Span::styled(
                format!(" :  {}", message.content.clone()),
                MessageConfig::text_style(),
            ),
        ])
    }

    fn time_format(time: SystemTime) -> String {
        let date_time: DateTime<Local> = time.into();
        date_time.format("%H:%M:%S").to_string()
    }
}
