use std::sync::{mpsc::Sender, Arc, Mutex};

use chat_core::{
    client_streams::ClientStreams, message::Message, read::ChatReader, request::Request,
    response::Response, user::User, value::Value, write::ChatWriter,
};

use crate::broadcast::BroadcastMessage;

use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_USER: User = User::builder().id(0).username("SERVER").build();
}

pub struct Client {
    pub key: usize,
    pub streams: ClientStreams,
}

impl Client {
    pub fn new(key: usize, streams: ClientStreams) -> Self {
        Self { key, streams }
    }
    pub fn key(&self) -> usize {
        self.key
    }
    pub fn run(&mut self, broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>) {
        /// This is strictly used only to make to sure that the
        /// client the corresponds to the key is removed from the
        /// HashMap on exiting this function. This is important
        /// because not removing a client will mean wasted memory
        /// and broadcasting messages would fail everytime.
        /// https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
        struct Exit {
            message_broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>,
            key: usize,
        }
        impl Drop for Exit {
            fn drop(&mut self) {
                print!("cleaning up client...  ");
                self.message_broadcaster
                    .lock()
                    .unwrap()
                    .send(BroadcastMessage::RemoveClient(self.key))
                    .unwrap();
                println!("removed client data");
            }
        }

        let _exit = {
            let message_broadcaster = Arc::clone(&broadcaster);

            Exit {
                message_broadcaster,
                key: self.key,
            }
        };

        broadcaster
            .lock()
            .unwrap()
            .send(BroadcastMessage::AddClient(
                self.streams.clone(),
                self.key(),
            ))
            .unwrap();

        // Create a user
        let mut user = {
            // Get peer_addresses
            let peer_addresses = self.streams.peer_addrs();
            let peer_addresses = match peer_addresses {
                (Ok(read), Ok(write)) => (read, write),
                _ => {
                    self.streams
                        .write_data(&Response::Error(
                            "Could not get your ip-addresses!".to_owned(),
                        ))
                        .ok();
                    return;
                }
            };

            // Build a user with a random name
            User::builder()
                .username(format!("anonymous-{}", User::random_name()))
                .id(self.key)
                .addresses(Some(peer_addresses))
                .build()
        };

        println!("New client connected: {user:?}");

        broadcaster
            .lock()
            .unwrap()
            .send(BroadcastMessage::ChatMessage(
                Message::builder()
                    .from(SERVER_USER.clone())
                    .payload(Value::String(format!("{user} has joined")))
                    .build(),
            ))
            .unwrap();

        loop {
            // Read request (Blocks thread until there is something to read)
            let request = self.streams.read_data::<Request>();

            let request = match request {
                Ok(request) => request,
                Err(error) => {
                    self.streams
                        .write_data(&Response::Error(format!("Bad request: {error}")))
                        .ok();
                    return;
                }
            };

            match request {
                Request::SendMessage(message) => {
                    let message = Message::builder()
                        .from(user.hide_addr())
                        .payload(message)
                        .build();

                    // Broadcast message to other clients
                    broadcaster
                        .lock()
                        .unwrap()
                        .send(BroadcastMessage::ChatMessage(message))
                        .unwrap();
                }
                Request::ChangeUserName(username) => {
                    broadcaster
                        .lock()
                        .unwrap()
                        .send(BroadcastMessage::ChatMessage(
                            Message::builder()
                                .from(SERVER_USER.clone())
                                .payload(Value::String(format!(
                                    "Requesting change username. {user} -> {username}"
                                )))
                                .build(),
                        ))
                        .unwrap();

                    user.set_username::<String>(match username.try_into() {
                        Ok(username) => username,
                        Err(error) => {
                            self.streams
                                .write_data(&format!(
                                    "Cannot change username to non-string data: {error}"
                                ))
                                .ok();
                            return;
                        }
                    });
                }
                Request::UserList => todo!(),
            }
        }
    }
}
