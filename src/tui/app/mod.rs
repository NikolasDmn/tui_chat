mod connection_list;
mod message_box;
mod text_area;
use std::{
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
    time::SystemTime,
};

use connection_list::ConnectionList;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use message_box::MessageBox;
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Position, Rect},
    style::{Color, Stylize},
    widgets::Clear,
    Frame,
};
use text_area::TextArea;

use crate::networking::{
    listener::{self, Listener},
    Connection, Message, MessageType,
};

#[derive(PartialEq, Eq)]
pub enum AppState {
    Normal,
    Writing,
    Closing,
    AddingConnection,
    ConfirmingConnection,
}

pub struct App<'a> {
    connection_list: ConnectionList<'a>,
    input_widget: TextArea<'a>,
    pub state: AppState,
    adding_connection_popup: TextArea<'a>,
    listener: Listener,
}

impl App<'_> {
    pub fn new() -> Self {
        let mut listener = Listener::new();
        listener.setup_thread();
        Self {
            connection_list: ConnectionList::new(),
            input_widget: TextArea::new("Message".to_string()),
            state: AppState::Normal,
            adding_connection_popup: TextArea::new(listener.get_ip().clone()),
            listener,
        }
    }
    pub fn update_connection_list(&mut self) {
        self.connection_list.update(self.state == AppState::Normal);
    }

    pub fn handle_input(&mut self, key: &KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match self.state {
            AppState::Normal => self.handle_normal_input(key),
            AppState::Writing => self.handle_writting_input(key),
            AppState::AddingConnection => self.handle_adding_connection_input(key),
            AppState::ConfirmingConnection => self.handle_confirm_connection_input(key),
            AppState::Closing => {}
        }
    }

    fn handle_confirm_connection_input(&mut self, key: &KeyEvent) {
        todo!()
    }
    fn handle_adding_connection_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state = AppState::Normal,
            KeyCode::Char(c) => self.adding_connection_popup.enter_char(c),
            KeyCode::Backspace => self.adding_connection_popup.delete_current_char(),
            KeyCode::Left => self.adding_connection_popup.move_cursor_left(),
            KeyCode::Right => self.adding_connection_popup.move_cursor_right(),
            KeyCode::Enter => self.handle_add_connection(),
            KeyCode::Delete => {
                self.adding_connection_popup.move_cursor_right();
                self.adding_connection_popup.delete_current_char()
            }
            _ => {}
        }
    }
    fn handle_add_connection(&mut self) {
        if let Some(connection) =
            TcpStream::connect(self.adding_connection_popup.content.clone()).ok()
        {
            self.connection_list
                .connections
                .lock()
                .unwrap()
                .push(Connection::new(connection));
            self.adding_connection_popup.clear_input();
            self.connection_list.list_state.select_last();
        }
    }
    fn handle_normal_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.closing_sequence(),
            KeyCode::Down | KeyCode::Char('j') => self.connection_list.iterate_selected(1),
            KeyCode::Up | KeyCode::Char('k') => self.connection_list.iterate_selected(-1),
            KeyCode::Char('c') => self.input_widget.clear_input(),
            KeyCode::Char('a') => self.state = AppState::AddingConnection,
            KeyCode::Char('i') | KeyCode::Tab | KeyCode::Enter => self.hanlde_select_connection(),
            _ => {}
        }
    }

    fn handle_create_connection(&mut self) {
        todo!()
    }

    fn hanlde_select_connection(&mut self) {
        if self.connection_list.list_state.selected().is_none() {
            return;
        }
        self.state = AppState::Writing
    }

    fn handle_writting_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => self.state = AppState::Normal,
            KeyCode::Char(c) => self.input_widget.enter_char(c),
            KeyCode::Backspace => self.input_widget.delete_current_char(),
            KeyCode::Left => self.input_widget.move_cursor_left(),
            KeyCode::Right => self.input_widget.move_cursor_right(),
            KeyCode::Enter => self.handle_message_send(),
            KeyCode::Delete => {
                self.input_widget.move_cursor_right();
                self.input_widget.delete_current_char()
            }
            _ => {}
        }
    }

    fn handle_message_send(&mut self) {
        todo!();
    }
    fn closing_sequence(&mut self) {
        //        self.connection_list
        //           .connections
        //          .lock()
        //         .unwrap()
        //        .iter()
        //       .for_each(|&c| c.disconnect());
        self.state = AppState::Closing
    }
    pub fn render(&mut self, frame: &mut Frame) {
        let list_constraint = if self.state == AppState::Writing {
            Constraint::Percentage(15)
        } else {
            Constraint::Percentage(30)
        };
        if let Some(stream) = self.listener.pop() {
            self.connection_list
                .connections
                .lock()
                .unwrap()
                .push(Connection::new(stream));
        }
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![list_constraint, Constraint::Fill(1)])
            .split(frame.area());
        frame.render_stateful_widget(
            self.connection_list.list.clone(),
            main_layout[0],
            &mut self.connection_list.list_state,
        );
        let text_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Min(6)])
            .split(main_layout[1]);
        frame.render_widget(
            self.input_widget
                .get_widget(self.state == AppState::Writing),
            text_layout[1],
        );
        if let Some(index) = self.connection_list.list_state.selected() {
            frame.render_widget(
                MessageBox::new(
                    &self
                        .connection_list
                        .connections
                        .lock()
                        .unwrap()
                        .get(index)
                        .unwrap()
                        .messages
                        .lock()
                        .unwrap(),
                ),
                text_layout[0],
            );
        }
        if self.state == AppState::Writing {
            frame.set_cursor_position(Position::new(
                text_layout[1].x + self.input_widget.character_index as u16 + 1,
                text_layout[1].y + 1,
            ));
        } else if self.state == AppState::AddingConnection {
            let area = App::centered_popup(
                frame.area(),
                Constraint::Percentage(60),
                Constraint::Length(3),
            );
            frame.render_widget(Clear, area);
            frame.render_widget(
                self.adding_connection_popup
                    .get_widget(true)
                    .bg(Color::Black),
                area,
            );
        }
    }
    fn centered_popup(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
        let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
        area
    }
}
