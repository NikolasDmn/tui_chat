use std::sync::{Arc, Mutex};

use ratatui::widgets::{Block, List, ListState};

use crate::{networking::Connection, tui::config::ListConfig};

use super::AppState;

pub struct ConnectionList<'a> {
    pub list: List<'a>,
    pub connections: Arc<Mutex<Vec<Connection>>>,
    pub list_state: ListState,
}
impl<'a> ConnectionList<'a> {
    pub fn new() -> ConnectionList<'a> {
        ConnectionList {
            list: List::new(Vec::<String>::new()),
            connections: Arc::new(Mutex::new(vec![])),
            list_state: ListState::default(),
        }
    }
    pub fn update(&mut self, selected: bool) {
        let connection_names: Vec<String> = self
            .connections
            .lock()
            .unwrap()
            .iter()
            .map(|c| c.get_name())
            .collect();
        let conn_len = connection_names.len();
        self.list = List::new(connection_names)
            .block(Block::bordered().title("Connections"))
            .style(if selected {
                ListConfig::selected_color()
            } else {
                ListConfig::unselected_color()
            })
            .highlight_style(ListConfig::highlight())
            .highlight_symbol(">>")
            .direction(ListConfig::direction());
        if let Some(index) = self.list_state.selected() {
            if index >= conn_len {
                self.list_state.select(Some(conn_len - 1));
            }
        }
    }

    pub fn iterate_selected(&mut self, step: i32) {
        if self.list_state.selected().is_none() {
            if step > 0 {
                self.list_state.select_first();
            } else {
                self.list_state.select_last();
            }
            return;
        }
        let conn_len = self.connections.lock().unwrap().len() as i32;
        let current_index = self.list_state.selected().unwrap();
        let mut index = (current_index as i32 + step) % conn_len;

        while index < 0 {
            index += conn_len
        }
        self.list_state.select(Some(index as usize));
    }
}
