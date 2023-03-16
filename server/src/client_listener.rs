use std::{
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use chat_core::client_streams::ClientStreams;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::{broadcast::Broadcaster, client::Client, config::ServerConfig};

pub struct ClientListener {
    pool: ThreadPool,
    listener: TcpListener,
}

impl ClientListener {
    // Maybe later remove Box<dyn std::error::Error> for a custom error type, but is this even needed?
    pub fn new(_config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            listener: TcpListener::bind("127.0.0.1:1234")?,
            pool: ThreadPoolBuilder::new().num_threads(20).build()?,
        })
    }
    pub fn run(self) {
        let message_broadcaster = Arc::new(Mutex::new(Broadcaster::default().run()));

        let mut key = 0;
        for stream in self.listener.incoming() {
            key += 1;
            match stream {
                Ok(write_stream) => {
                    // Get the address of the stream that connected
                    let mut read_port = match write_stream.peer_addr() {
                        Ok(port) => port,
                        Err(error) => {
                            eprintln!("Failed to get peer_addr!: {error}");
                            return;
                        }
                    };
                    // Set the port to the reading port (the client has opened this port, and is waiting for a connection on it)
                    read_port.set_port(4321);
                    let read_stream = match TcpStream::connect(read_port) {
                        Ok(stream) => stream,
                        Err(error) => {
                            eprintln!(
                                "Error connecting to server read stream (client write): {error}"
                            );
                            return;
                        }
                    };
                    // Create a new ClientStreams enum with the Server variant
                    let client_streams = ClientStreams::Server(
                        Arc::new(Mutex::new(read_stream)),
                        Arc::new(Mutex::new(write_stream)),
                    );

                    eprintln!("Estabilished Connection: {client_streams:?}");

                    let mut client = Client::new(key, client_streams);

                    self.pool.spawn({
                        let message_broadcaster = Arc::clone(&message_broadcaster);
                        move || client.run(message_broadcaster)
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {e}");
                }
            }
        }
    }
}
