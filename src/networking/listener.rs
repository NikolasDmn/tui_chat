use std::{
    borrow::Borrow,
    collections::{LinkedList, VecDeque},
    net::{TcpListener, TcpStream},
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
};

pub struct Listener {
    listener: Arc<Mutex<TcpListener>>,
    pending_connections: Arc<Mutex<LinkedList<TcpStream>>>,
    running: Arc<Mutex<bool>>,
}

impl Listener {
    pub fn new() -> Self {
        Self {
            listener: Arc::new(Mutex::new(TcpListener::bind("127.0.0.1:0").unwrap())),
            pending_connections: Arc::new(Mutex::new(LinkedList::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    pub fn setup_thread(&self) {
        let listener = Arc::clone(&self.listener);
        let running = Arc::clone(&self.running);
        let pending_connections = Arc::clone(&self.pending_connections);
        thread::spawn(move || {
            while *running.lock().unwrap() {
                let (stream, _) = listener.lock().unwrap().accept().unwrap();
                pending_connections.lock().unwrap().push_front(stream);
            }
        });
    }
    pub fn pop(&mut self) -> Option<TcpStream> {
        self.pending_connections.lock().unwrap().pop_back()
    }
    pub fn get_ip(&self) -> String {
        self.listener
            .lock()
            .unwrap()
            .local_addr()
            .unwrap()
            .to_string()
    }
}
