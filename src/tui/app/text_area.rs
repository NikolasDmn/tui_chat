use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::tui::config::InputConfig;

pub struct TextArea<'a> {
    pub content: String,
    pub character_index: usize,
    widget: Paragraph<'a>,
    title: String,
}

impl TextArea<'_> {
    pub fn new(title: String) -> Self {
        Self {
            content: String::new(),
            character_index: 0,
            widget: Paragraph::new(vec![]),
            title,
        }
    }
    fn clamp_cursor(&self, index: usize) -> usize {
        index.clamp(0, self.content.chars().count())
    }
    pub fn move_cursor_left(&mut self) {
        self.character_index = self.clamp_cursor(self.character_index.saturating_sub(1));
    }

    pub fn move_cursor_right(&mut self) {
        self.character_index = self.clamp_cursor(self.character_index.saturating_add(1));
    }
    pub fn reset_cursor(&mut self) {
        self.character_index = 0;
    }
    fn byte_index(&self) -> usize {
        self.content
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.content.len())
    }
    pub fn enter_char(&mut self, chr: char) {
        let index = self.byte_index();
        self.content.insert(index, chr);
        self.move_cursor_right()
    }
    pub fn delete_current_char(&mut self) {
        if self.character_index == 0 {
            return;
        }
        let left_from_current_index = self.character_index - 1;
        let chars_before = self.content.chars().take(left_from_current_index);
        let chars_after = self.content.chars().skip(self.character_index);

        self.content = chars_before.chain(chars_after).collect();
        self.move_cursor_left();
    }

    pub fn clear_input(&mut self) {
        self.content.clear();
        self.reset_cursor();
    }

    pub fn get_widget(&self, writable: bool) -> Paragraph {
        Paragraph::new(self.content.as_str())
            .style(if writable {
                InputConfig::selected_color()
            } else {
                InputConfig::unselected_color()
            })
            .block(Block::bordered().title(self.title.clone()))
            .wrap(Wrap { trim: true })
    }
}
