use chat_core::{config::Config, message::MessageGuidelines, user::UsernameGuidelines};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct ServerConfig {
    pub net: NetConfig,
    pub system: SystemConfig,
    pub message_guidelines: MessageGuidelines,
    pub username_guidelines: UsernameGuidelines,
}

#[derive(Deserialize, Serialize)]
pub struct NetConfig {
    ip: String,
    read_port: u16,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1:1234".to_owned(),
            read_port: 4321,
        }
    }
}

impl NetConfig {
    pub fn ip(&self) -> &str {
        &self.ip
    }
    pub fn read_port(&self) -> u16 {
        self.read_port
    }
}

#[derive(Deserialize, Serialize)]
pub struct SystemConfig {
    threads: usize,
    key_start: usize,
    verbose: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            threads: 20,
            key_start: 0,
            verbose: true,
        }
    }
}

impl SystemConfig {
    pub fn threads(&self) -> usize {
        self.threads
    }
    pub fn key_start(&self) -> usize {
        self.key_start
    }
    pub fn verbose(&self) -> bool {
        self.verbose
    }
}

impl Config for ServerConfig {}
