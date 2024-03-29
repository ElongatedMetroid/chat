use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use chat_core::{
    message::Message, read_write_streams::ReadWriteStreams, response::Response, write::ChatWriter,
};

#[derive(Debug)]
pub enum BroadcastMessage {
    /// Broadcast a message to all connected clients
    ChatMessage(Message),
    /// Add client along with a corresponding key
    AddClient(ReadWriteStreams, usize),
    /// Remove client with id
    RemoveClient(usize),
}

/// Recieves messages through the Sender<BroadcastMessage>
/// returned from Broadcaster::run(). Handles stuff involving all
/// clients such as, broadcasting messages to all clients.
#[derive(Default)]
pub struct Broadcaster {
    clients: HashMap<usize, ReadWriteStreams>,
}

impl Broadcaster {
    /// Start the broadcaster thread, returns a `Sender<BroadcastMessage>` to send data to its thread
    pub fn run(mut self) -> Sender<BroadcastMessage> {
        log::info!("running broadcaster");

        let (tx, rx): (Sender<BroadcastMessage>, Receiver<BroadcastMessage>) = mpsc::channel();

        thread::spawn(move || loop {
            let message = rx.recv().unwrap();
            log::info!("recieved a BroadcastMessage");
            log::debug!("{message:?}");

            match message {
                BroadcastMessage::ChatMessage(message) => {
                    log::debug!("chat message broadcast recieved");
                    self.clients.broadcast(message);
                }
                BroadcastMessage::AddClient(client, key) => {
                    log::debug!("add client broadcast recieved");
                    self.clients.insert(key, client);
                }
                BroadcastMessage::RemoveClient(key) => {
                    log::debug!("remove client broadcast recieved");
                    self.clients.remove(&key);
                }
            }
        });

        log::info!("broadcaster thread started");

        tx
    }
}

pub trait Broadcast {
    /// Broadcast a `Message` to all clients, cannot error (error should be handled inside)
    fn broadcast(&mut self, message: Message);
}

impl Broadcast for HashMap<usize, ReadWriteStreams> {
    /// Broadcast a `Message` to all clients, if writing data to a client fails, the message will not be broadcasted to that
    /// client (the client is not removed).
    fn broadcast(&mut self, message: Message) {
        log::info!("broadcasting message");
        let response = Response::Ok(message);
        log::debug!("created response: {response:?}");

        for client in self.values_mut() {
            log::debug!("broadcasting to: {client:?}");
            // Error is ignored since the client handler should handle what happens if a client fails
            let _ = client.write_data(&response);
        }
    }
}
