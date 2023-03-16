use std::process;

use chat_core::config::Config;
use server::{client_listener::ClientListener, config::ServerConfig};

fn main() {
    let config = match ServerConfig::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Failed to load conifg: {error}. Using defaults");
            ServerConfig::default()
        }
    };

    match ClientListener::new(config) {
        Ok(client_listener) => client_listener,
        Err(error) => {
            eprintln!("Failed to start client: {error}");
            process::exit(1);
        }
    }
    .run();
}
