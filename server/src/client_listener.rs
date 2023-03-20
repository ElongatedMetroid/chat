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
    config: ServerConfig,
}

impl ClientListener {
    // Maybe later remove Box<dyn std::error::Error> for a custom error type, but is this even needed?
    pub fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("creating new client listener");
        Ok(Self {
            listener: TcpListener::bind("127.0.0.1:1234")?,
            pool: ThreadPoolBuilder::new().num_threads(20).build()?,
            config,
        })
    }
    pub fn run(self) {
        log::info!("listening for clients");
        let message_broadcaster = Arc::new(Mutex::new(Broadcaster::default().run()));

        let config = Arc::new(self.config);
        let mut key = config.system.key_start();
        for stream in self.listener.incoming() {
            log::info!("got connection");
            key += 1;
            match stream {
                Ok(write_stream) => {
                    // Get the address of the stream that connected
                    let mut read_addr = match write_stream.peer_addr() {
                        Ok(port) => port,
                        Err(error) => {
                            log::warn!("failed to get peer_addr: {error}");
                            return;
                        }
                    };
                    // Set the port to the reading port (the client has opened this port, and is waiting for a connection on it)
                    read_addr.set_port(config.net.read_port());
                    let read_stream = match TcpStream::connect(read_addr) {
                        Ok(stream) => stream,
                        Err(error) => {
                            log::warn!(
                                "error connecting to server read stream (client write): {error}"
                            );
                            return;
                        }
                    };
                    // Create a new ClientStreams enum with the Server variant
                    let client_streams = ClientStreams::Server(
                        Arc::new(Mutex::new(read_stream)),
                        Arc::new(Mutex::new(write_stream)),
                    );

                    log::info!("Estabilished Connection: {client_streams:?}");

                    self.pool.spawn({
                        let message_broadcaster = Arc::clone(&message_broadcaster);
                        let config = Arc::clone(&config);
                        move || Client::new(key, client_streams, message_broadcaster, config).run()
                    });
                }
                Err(error) => {
                    log::warn!("failed to accept connection: {error}")
                }
            }
        }
    }
}
