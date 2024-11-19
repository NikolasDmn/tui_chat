use std::io::{Read, Write};
pub mod listener;

use std::u8;
use std::{
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Mutex,
    },
    thread, time,
    time::SystemTime,
};

const SELF_NAME: &str = "Me";

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MessageType {
    Text = 0,
    NameChange = 1,
    Encryption = 2,
    Error = 3,
}

impl MessageType {
    // Custom function to map a byte (u8) to the corresponding MessageType enum
    fn from_u8(byte: u8) -> Option<MessageType> {
        match byte {
            0 => Some(MessageType::Text),
            1 => Some(MessageType::NameChange),
            2 => Some(MessageType::Encryption),
            3 => Some(MessageType::Error),
            _ => None, // Return None if the byte doesn’t match any known MessageType
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub time: SystemTime,
    pub sent_by_self: bool,
    pub sender_name: String,
    pub message_type: MessageType,
    pub content: String,
}

pub struct Connection {
    pub name: Arc<Mutex<String>>,
    stream: Arc<Mutex<TcpStream>>,
    is_alive: Arc<AtomicBool>,
    pub messages: Arc<Mutex<Vec<Message>>>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        // Setup name as the IP to start
        let ip: String;
        if let Ok(ipv4) = stream.local_addr() {
            ip = format!("{:?}", ipv4.ip());
        } else {
            ip = "Err".to_string();
        }
        let name = Arc::new(Mutex::new(ip));
        let arc_stream = Arc::new(Mutex::new(stream));
        let is_alive = Arc::new(AtomicBool::new(true));
        let messages = Arc::new(Mutex::new(vec![]));
        Connection {
            name,
            stream: arc_stream,
            is_alive,
            messages,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.lock().unwrap().clone()
    }
    pub fn register_listener(connection: Arc<Mutex<Self>>) {
        let mut conn = Arc::clone(&connection);

        thread::spawn(move || {
            let mut buffer = [0; 512];
            while conn.lock().unwrap().is_alive.load(Relaxed) {
                dbg!("Incoming message...");
                let mut stream = conn
                    .lock()
                    .unwrap()
                    .stream
                    .clone()
                    .lock()
                    .unwrap()
                    .try_clone()
                    .unwrap();
                match stream.read(&mut buffer) {
                    Ok(0) => {
                        conn.lock().unwrap().is_alive.store(false, Relaxed);
                    }
                    Ok(n) => {
                        conn.lock()
                            .unwrap()
                            .handle_incoming_data(Vec::from(&buffer[..n]));
                    }
                    Err(e) => conn
                        .lock()
                        .unwrap()
                        .register_incoming_message(format!("{}", e), MessageType::Error),
                }
            }
        });
    }

    fn handle_incoming_data(&mut self, data: Vec<u8>) {
        let message_type_byte = data[0];
        if let Some(message_type) = MessageType::from_u8(message_type_byte) {
            match message_type {
                MessageType::Text | MessageType::Error => {
                    let message = String::from_utf8_lossy(&data[1..]);
                    self.register_incoming_message(message.to_string(), message_type);
                }
                MessageType::NameChange => {
                    let new_name = String::from_utf8_lossy(&data[1..]);
                    let mut name_guard = self.name.lock().unwrap();
                    *name_guard = new_name.to_string();
                }
                MessageType::Encryption => {
                    println!("Encryption not yet implemented");
                }
            }
        }
    }

    fn register_incoming_message(&mut self, message: String, message_type: MessageType) {
        let mut messages = self.messages.lock().unwrap();
        let name = self.name.lock().unwrap().clone();
        messages.push(Message {
            time: SystemTime::now(),
            sent_by_self: false,
            sender_name: name,
            message_type,
            content: message,
        });
    }

    pub fn disconnect(&mut self) {
        self.is_alive
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn send_message(&mut self, message: String, message_type: MessageType) {
        if !self.is_alive.load(Relaxed) {
            return;
        }
        let mut stream = self.stream.lock().unwrap();
        let mut buffer = vec![message_type as u8];
        buffer.extend(message.bytes());
        let mut messages = self.messages.lock().unwrap();
        if let Err(e) = stream.write(&buffer) {
            messages.push(Message {
                time: SystemTime::now(),
                sent_by_self: true,
                sender_name: SELF_NAME.to_string(),
                message_type: MessageType::Error,
                content: format!("{}", e),
            })
        } else {
            messages.push(Message {
                time: SystemTime::now(),
                sent_by_self: true,
                sender_name: SELF_NAME.to_string(),
                message_type: MessageType::Text,
                content: message,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use std::net::{Ipv4Addr, SocketAddr, TcpListener};

    use super::*;

    fn mock_tcpstream() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().expect("").port();

        dbg!(port);
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();

        let (stream2, _) = listener.accept().unwrap();

        (stream, stream2)
    }
    #[test]
    fn test_new_connection_sets_defaults_correctly() {
        // Arrange
        let (stream1, stream2) = mock_tcpstream(); // Mock a TcpStream

        // Act
        let conn = Connection::new(stream1);

        // Assert
        // Check if the name was set to the IP address
        let name = conn.name.lock().unwrap();
        assert_eq!(*name, "127.0.0.1");

        // Check if the stream exists and is accessible
        assert!(conn.stream.lock().is_ok());

        // Check if is_alive is set to true
        assert_eq!(conn.is_alive.load(Relaxed), true);

        // Ensure the messages list is empty at the start
        let messages = conn.messages.lock().unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_connections() {
        let (stream1, stream2) = mock_tcpstream();
        let conn1 = Connection::new(stream1);
        let conn2 = Connection::new(stream2);
        let conn1_arc = Arc::new(Mutex::new(conn1));
        let conn2_arc = Arc::new(Mutex::new(conn2));
        Connection::register_listener(Arc::clone(&conn1_arc));
        Connection::register_listener(Arc::clone(&conn2_arc));

        conn1_arc
            .lock()
            .unwrap()
            .send_message("nigga".to_string(), MessageType::Text);

        thread::sleep(time::Duration::from_millis(100));

        let msg1: Vec<Message> = conn1_arc
            .lock()
            .unwrap()
            .messages
            .lock()
            .unwrap()
            .iter()
            .map(|m| m.clone())
            .collect();

        let msg2: Vec<Message> = conn2_arc
            .lock()
            .unwrap()
            .messages
            .lock()
            .unwrap()
            .iter()
            .map(|m| m.clone())
            .collect();
        assert_eq!(msg1.len(), 1);
        assert_eq!(msg2.len(), 1);
        assert_eq!(msg2[0].message_type, MessageType::Text);
        assert_eq!(msg2[0].content, "nigga");
    }
}
