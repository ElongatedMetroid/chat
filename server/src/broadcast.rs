use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use chat_core::{client_streams::ClientStreams, message::Message, write::ChatWriter};

pub enum BroadcastMessage {
    /// Broadcast a message to all connected clients
    ChatMessage(Message),
    /// Add client along with a corresponding key
    AddClient(ClientStreams, usize),
    /// Remove client with id
    RemoveClient(usize),
}

/// Recieves messages through the Sender<BroadcastMessage>
/// returned from Broadcaster::run(). Handles stuff involving all
/// clients such as, broadcasting messages to all clients.
#[derive(Default)]
pub struct Broadcaster {
    clients: HashMap<usize, ClientStreams>,
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
                BroadcastMessage::AddClient(client, key) => {
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

impl Broadcast for HashMap<usize, ClientStreams> {
    /// Broadcast a `Message` to all clients, if writing data to a client fails, the message will not be broadcasted to that
    /// client (the client is not removed).
    fn broadcast(&mut self, message: &Message) {
        for client in self.values_mut() {
            let _ = client.write_data(message);
        }
    }
}
