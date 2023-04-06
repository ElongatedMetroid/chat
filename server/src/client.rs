use std::{sync::{mpsc::Sender, Arc, Mutex}, net::TcpStream, io};

use chat_core::{
    guidelines::AgainstGuidelines,
    message::Message,
    read::ChatReader,
    read_write_streams::{ReadWriteStreams, ConnectPeerStream},
    request::{Request, RequestError},
    response::Response,
    user::{User, Username},
    value::Value,
    write::ChatWriter,
};

use crate::{broadcast::BroadcastMessage, config::ServerConfig};

use lazy_static::lazy_static;

lazy_static! {
    static ref SERVER_USER: User = User::builder()
        .id(0)
        .username(Username::new("SERVER"))
        .build();
}

pub struct Client {
    pub key: usize,
    pub streams: ReadWriteStreams,
    pub broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>,
    pub config: Arc<ServerConfig>,
}

impl Client {
    pub fn make_connection(
        key: usize,
        stream: TcpStream,
        broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>,
        config: Arc<ServerConfig>,
    ) -> Result<Self, io::Error> {
        let streams = stream.connect_peer_stream(config.net.read_port())?;

        Ok(Self {
            key,
            streams,
            broadcaster,
            config,
        })
    }
    pub fn key(&self) -> usize {
        self.key
    }
    /// Initial client code that is only ran once, very messy in how it works now but will be fixed later
    pub fn initial_connect(&mut self) -> Result<User, ()> {
        log::debug!("broadcasting add client message");
        self.broadcaster
            .lock()
            .unwrap()
            .send(BroadcastMessage::AddClient(
                self.streams.clone(),
                self.key(),
            ))
            .unwrap();

        log::info!("client added to chat broadcaster");

        // Create a user
        let user = {
            // Get peer_addresses
            let peer_addresses = self.streams.peer_addrs();
            let peer_addresses = match peer_addresses {
                (Ok(read), Ok(write)) => (read, write),
                _ => {
                    log::warn!("getting peer_addrs() of client failed");
                    self.streams
                        .write_data(&Response::Err(RequestError::Ip))
                        .ok();
                    return Err(());
                }
            };

            // Build a user with a random name
            User::builder()
                .username(Username::new(Value::String(format!(
                    "anonymous-{}",
                    User::random_name()
                ))))
                .id(self.key)
                .addresses(Some(peer_addresses))
                .build()
        };

        log::info!("new client connected: {user:?}");

        self.broadcaster
            .lock()
            .unwrap()
            .send(BroadcastMessage::ChatMessage(
                Message::builder()
                    .from_who(SERVER_USER.clone())
                    .payload(Value::String(format!("{user} has joined")))
                    .build(),
            ))
            .unwrap();

        Ok(user)
    }
    pub fn run(&mut self) {
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
                log::info!("cleaning up client...  ");
                self.message_broadcaster
                    .lock()
                    .unwrap()
                    .send(BroadcastMessage::RemoveClient(self.key))
                    .unwrap();
                log::info!("client removed");
            }
        }

        let _exit = {
            let message_broadcaster = Arc::clone(&self.broadcaster);

            Exit {
                message_broadcaster,
                key: self.key,
            }
        };

        let mut user = match self.initial_connect() {
            Ok(user) => user,
            // THIS METHOD HANDLES RETURNING AN ERROR RESPONSE!
            Err(_) => return,
        };

        loop {
            // Read request (Blocks thread until there is something to read)
            let request = self.streams.read_data::<Request>();
            log::debug!("got request: {request:?}");

            let request = match request {
                Ok(request) => request,
                Err(error) => {
                    log::debug!("bad request: {error}");
                    self.streams
                        .write_data(&Response::Err(RequestError::Bad(error.to_string())))
                        .ok();
                    return;
                }
            };

            match request {
                Request::SendMessage(message) => {
                    let message = match Message::builder()
                        .from_who(user.hide_addr())
                        .payload(message)
                        .build()
                        .against_guidelines(&self.config.message_guidelines)
                    {
                        Ok(message) => message,
                        Err(error) => {
                            log::info!("message did not follow guidelines");
                            self.streams
                                .write_data(&Response::Err(RequestError::Message(error)))
                                .ok();
                            return;
                        }
                    };

                    // Broadcast message to other clients
                    self.broadcaster
                        .lock()
                        .unwrap()
                        .send(BroadcastMessage::ChatMessage(message))
                        .unwrap();
                }
                Request::ChangeUserName(username) => {
                    self.broadcaster
                        .lock()
                        .unwrap()
                        .send(BroadcastMessage::ChatMessage(
                            Message::builder()
                                .from_who(SERVER_USER.clone())
                                .payload(Value::String(format!(
                                    "Requesting change username. {user} -> {username}"
                                )))
                                .build(),
                        ))
                        .unwrap();

                    user.set_username(
                        match Username::new(username)
                            .against_guidelines(&self.config.username_guidelines)
                        {
                            Ok(name) => name,
                            Err(error) => {
                                self.streams
                                    .write_data(&Response::Err(RequestError::Username(error)))
                                    .ok();
                                return;
                            }
                        },
                    );
                }
                Request::UserList => todo!(),
            }
        }
    }
}
