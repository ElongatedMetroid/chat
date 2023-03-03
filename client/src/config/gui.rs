use chat_core::{
    request::{Request, Requesting},
    value::Value,
    write::ChatWriter,
};
use egui::Window;
use std::{io, net::TcpStream};

use super::{Config, ConfigError};

pub struct ConfigGui {
    config: Option<Config>,
    config_handled: bool,
}

impl ConfigGui {
    pub fn new() -> Result<Self, ConfigError> {
        let config = ConfigGui {
            config: match Config::load() {
                Ok(config) => Some(config),
                Err(error) => match error {
                    ConfigError::IoError(error) if error.kind() == io::ErrorKind::NotFound => None,
                    _ => return Err(error),
                },
            },
            config_handled: false,
        };

        Ok(config)
    }
    /// This function will handle checking if the config has already been checked, so no need to wrap it in a check
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        client: &mut TcpStream,
    ) -> Result<(), bincode::Error> {
        // if the config has not been handled
        if !self.config_handled {
            let mut request_builder = None;

            match &self.config {
                // Config file exists, but we still need to send username data (or set a username)
                Some(config) => {
                    // Username is set, so we will add it as a payload (if it is not set the builder will still be none)
                    if let Some(name) = &config.username.name {
                        request_builder = Some(
                            Request::builder()
                                .requesting(Requesting::ChangeUserName)
                                .payload(Value::String(name.clone())),
                        );
                    }
                }
                // Config file does not exist, so we will create one
                None => loop {
                    Window::new("Configuration").show(&ctx, |ui| {});
                },
            }

            if let Some(request_builder) = request_builder {
                let request = request_builder.build();

                println!("{request:?}");

                client.write_data(&request)?;
            }

            self.config_handled = true;
        }

        Ok(())
    }
}
