use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use chat_core::{message::Message, write::ChatWriter};

pub enum BroadcastMessage {
    /// Broadcast a message to all connected clients
    ChatMessage(Message),
    /// Add client along with a corresponding key
    NewClient(Arc<Mutex<TcpStream>>, usize),
    /// Remove client with id
    RemoveClient(usize),
}

#[derive(Default)]
pub struct Broadcaster {
    clients: HashMap<usize, Arc<Mutex<TcpStream>>>,
}

impl Broadcaster {
    /// Start the broadcaster thread, returns a `Sender<BroadcastMessage>` to send data to its thread
    pub fn run(mut self) -> Sender<BroadcastMessage> {
        let (tx, rx): (Sender<BroadcastMessage>, Receiver<BroadcastMessage>) = mpsc::channel();

        thread::spawn(move || loop {
            let message = rx.recv().unwrap();

            match message {
                BroadcastMessage::ChatMessage(message) => {
                    self.clients.broadcast(&message);
                }
                BroadcastMessage::NewClient(client, key) => {
                    self.clients.insert(key, client);
                }
                BroadcastMessage::RemoveClient(key) => {
                    self.clients.remove(&key);
                }
            }
        });

        tx
    }
}

pub trait Broadcast {
    /// Broadcast a `Message` to all clients, cannot error (error should be handled inside)
    fn broadcast(&mut self, message: &Message);
}

impl Broadcast for HashMap<usize, Arc<Mutex<TcpStream>>> {
    /// Broadcast a `Message` to all clients, if writing data to a client fails, the message will not be broadcasted to that
    /// client (the client is not removed).
    fn broadcast(&mut self, message: &Message) {
        for client in self.values() {
            let _ = client.lock().unwrap().write_data(message);
        }
    }
}
