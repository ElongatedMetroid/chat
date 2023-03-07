use std::{
    io,
    net::TcpStream,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::Duration,
};

use chat_core::{message::Message, read::ChatReader, request::Request, user::User, value::Value};

use crate::broadcast::BroadcastMessage;

use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_USER: User = User::builder().id(0).username("SERVER").build();
}

#[derive(Clone)]
pub struct Client {
    pub key: usize,
    pub stream: Arc<Mutex<TcpStream>>,
}

impl Client {
    pub fn new(key: usize, stream: Arc<Mutex<TcpStream>>) -> Self {
        Self { key, stream }
    }
    pub fn key(&self) -> usize {
        self.key
    }
    pub fn run(&self, broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>) {
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
            .send(BroadcastMessage::AddClient(self.clone(), self.key()))
            .unwrap();

        // Create a user
        let mut user = {
            // Get peer_address
            let peer_address = self.stream.lock().unwrap().peer_addr();
            let peer_address = match peer_address {
                Ok(peer_address) => peer_address,
                Err(error) => {
                    eprintln!("Failed to get peer_address: {error}");
                    return;
                }
            };

            // Build a user with a random name
            User::builder()
                .username(format!("anonymous-{}", User::random_name()))
                .id(self.key)
                .address(Some(peer_address))
                .build()
        };

        // Set client to non-blocking, later maybe do this with async
        let result = self.stream.lock().unwrap().set_nonblocking(true);
        if let Err(error) = result {
            eprintln!("Failed to set non_blocking: {error}");
            return;
        }

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
            thread::sleep(Duration::from_millis(100));
            // Read request
            let request = self.stream.lock().unwrap().read_data::<Request>();

            let request = match request {
                Ok(request) => request,
                Err(error) => match *error {
                    // No data to be read,
                    bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        continue
                    }
                    _ => {
                        eprintln!("Bad request from client {:?}: {error}", user.addr());
                        return;
                    }
                },
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
                            eprintln!("Cannot change username to non-string data: {error}");
                            return;
                        }
                    });
                }
                Request::UserList => todo!(),
            }
        }
    }
}
