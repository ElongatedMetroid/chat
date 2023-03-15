use chat_core::config::Config;
use serde::{Deserialize, Serialize};

pub mod gui;

#[derive(Default, Deserialize, Serialize)]
pub struct ClientConfig {
    pub username: Username,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Username {
    name: Option<String>,
}

impl Username {
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
}

impl Config for ClientConfig {}
