use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

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
                Ok(stream) => {
                    self.pool.spawn({
                        let message_broadcaster = Arc::clone(&message_broadcaster);
                        let config = Arc::clone(&config);
                        move || {
                            match Client::make_connection(
                                key,
                                stream,
                                message_broadcaster,
                                config,
                            ) {
                                Ok(client) => client,
                                Err(error) => {
                                    log::error!("failed to create client: {error}");
                                    return;
                                },
                            }
                            .run()
                        }
                    });
                }
                Err(error) => {
                    log::warn!("failed to accept connection: {error}")
                }
            }
        }
    }
}
