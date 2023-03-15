use chat_core::{
    client_streams::ClientStreams,
    config::{Config, ConfigError},
    request::Request,
    value::Value,
    write::ChatWriter,
};
use egui::Window;
use std::{io, process};

use super::ClientConfig;

pub struct ConfigGui {
    config: Option<ClientConfig>,
    config_handled: bool,
    client_streams: ClientStreams,

    create_config_data: CreateConfigData,
}

pub struct CreateConfigData {
    random_username: bool,
    username: Option<String>,
    config: Option<ClientConfig>,
}

impl ConfigGui {
    /// Returns a new `ConfigGui`, this will open a config gui with the Config::load() function. If the config cannot be
    /// found an error is not returned since creating a config will be handled in the update_gui() method. However an
    /// error will be returned if something other than `io::ErrorKind::NotFound` is returned from Config::load().
    pub fn new(client_streams: ClientStreams) -> Result<Self, ConfigError> {
        let config = ConfigGui {
            config: match ClientConfig::load() {
                Ok(config) => Some(config),
                Err(error) => match error {
                    ConfigError::IoError(error) if error.kind() == io::ErrorKind::NotFound => None,
                    _ => return Err(error),
                },
            },
            config_handled: false,
            client_streams,

            create_config_data: CreateConfigData {
                random_username: false,
                username: Some(String::new()),
                config: None,
            },
        };

        Ok(config)
    }
    /// This function will handle checking if the config has already been checked, so no need to wrap it in a check
    pub fn update_gui(&mut self, ctx: &egui::Context) -> Result<(), bincode::Error> {
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
                // Config file does not exist, so we will create one
                None => {
                    Window::new("Config").show(ctx, |ui| {
                        ui.vertical(|ui| {
                            // If the random username checkbox is not checked, show a text box for a username
                            if !self.create_config_data.random_username {
                                ui.text_edit_singleline(
                                    self.create_config_data.username.as_mut().unwrap(),
                                );
                            }
                            ui.checkbox(
                                &mut self.create_config_data.random_username,
                                "Want a random username everytime?",
                            );
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("Do this Later").clicked() {
                                self.config_handled = true;
                            }

                            if ui.button("Done").clicked() {
                                self.create_config_data.config = Some(ClientConfig::default());
                                // Set the username, if the random_username button was checked set it to none, otherwise
                                // set it to the contents of the username text box.
                                self.create_config_data
                                    .config
                                    .as_mut()
                                    .unwrap()
                                    .username
                                    .set_name(if self.create_config_data.random_username {
                                        None
                                    } else {
                                        Some(self.create_config_data.username.take().unwrap())
                                    });

                                if let Err(error) =
                                    self.create_config_data.config.as_ref().unwrap().write()
                                {
                                    eprintln!("Could not write config!: {error}");
                                    process::exit(1);
                                } else if self.create_config_data.random_username {
                                    self.config_handled = true;
                                }
                            }
                        });
                    });

                    if let Some(config) = &self.create_config_data.config {
                        config
                            .username
                            .name
                            .as_ref()
                            // Convert Option<String> to Option<Request>
                            .map(|name| Request::ChangeUserName(Value::from(name.clone())))
                    } else {
                        None
                    }
                }
            };

            if let Some(request) = request {
                self.client_streams.write_data(&request)?;
                self.config_handled = true;
            }
        }

        Ok(())
    }
}
