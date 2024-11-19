pub mod ListConfig {
    use ratatui::{
        style::{Color, Style, Stylize},
        widgets::ListDirection,
    };

    // Setup editable conf file for this

    pub fn unselected_color() -> Style {
        Style::new().fg(Color::Gray)
    }
    pub fn selected_color() -> Style {
        Style::new().fg(Color::Yellow)
    }
    pub fn highlight() -> Style {
        Style::new().bold().italic()
    }

    pub fn direction() -> ListDirection {
        ListDirection::TopToBottom
    }
}

pub mod MessageConfig {
    use ratatui::style::{Color, Style, Stylize};

    pub fn username_style(is_from_client: bool) -> Style {
        if is_from_client {
            Style::new().fg(Color::LightYellow).bold()
        } else {
            Style::new().fg(Color::Yellow).bold()
        }
    }
    pub fn time_style() -> Style {
        Style::new().fg(Color::Gray).italic()
    }
    pub fn text_style() -> Style {
        Style::new().fg(Color::White)
    }
}
pub mod InputConfig {
    use ratatui::style::{Color, Style};

    pub fn unselected_color() -> Style {
        Style::new().fg(Color::Gray)
    }
    pub fn selected_color() -> Style {
        Style::new().fg(Color::Yellow)
    }
}
