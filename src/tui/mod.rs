use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    widgets::{Block, List, ListDirection, ListState, Paragraph},
    DefaultTerminal,
};
use std::{
    io,
    sync::{Arc, Mutex, MutexGuard},
};
mod config;
//use crate::networking::Connection;
use config::ListConfig;
mod app;
use app::{App, AppState};
pub fn start() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    while app.state != AppState::Closing {
        app.update_connection_list();
        terminal.draw(|frame| app.render(frame))?;
        if let event::Event::Key(key) = event::read()? {
            app.handle_input(&key);
        }
    }
    Ok(())
}
