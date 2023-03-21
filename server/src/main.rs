use std::process;

use chat_core::config::Config;
use log::LevelFilter;
use server::{client_listener::ClientListener, config::ServerConfig};
use simple_logger::SimpleLogger;

fn main() {
    let config = match ServerConfig::load() {
        Ok(config) => config,
        Err(error) => {
            log::warn!("failed to load conifg: {error}");
            log::info!("using default config");
            ServerConfig::default()
        }
    };

    SimpleLogger::new()
        .with_level(if config.system.verbose() {
            LevelFilter::Trace
        } else {
            LevelFilter::Warn
        })
        .init()
        .unwrap();

    log::info!("config loaded");

    match ClientListener::new(config) {
        Ok(client_listener) => client_listener,
        Err(error) => {
            log::error!("Failed to start client: {error}");
            process::exit(1);
        }
    }
    .run();
}
