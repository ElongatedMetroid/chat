use chat_core::{request::Request, value::Value, write::ChatWriter};
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
            let request = match &self.config {
                // Config file exists, but we still need to send username data (or set a username)
                Some(config) => {
                    config
                        .username
                        .name
                        .as_ref()
                        // Convert Option<String> to Option<Request>
                        .map(|name| Request::ChangeUserName(Value::from(name.clone())))
                }
                // TODO: Config file does not exist, so we will create one
                None => None,
            };

            if let Some(request) = request {
                client.write_data(&request)?;
            }

            self.config_handled = true;
        }

        Ok(())
    }
}
