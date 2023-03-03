use std::{
    fs::{self, File},
    io::Write,
};

use serde::{Deserialize, Serialize};

pub mod gui;

#[derive(Deserialize, Serialize)]
pub struct Config {
    username: Username,
}

#[derive(Deserialize, Serialize)]
pub struct Username {
    name: Option<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    SerError(toml::ser::Error),
    DeError(toml::de::Error),
    IoError(std::io::Error),
}

impl From<toml::ser::Error> for ConfigError {
    fn from(value: toml::ser::Error) -> Self {
        ConfigError::SerError(value)
    }
}
impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::DeError(value)
    }
}
impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::IoError(value)
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(toml::from_str(&fs::read_to_string("Config.toml")?)?)
    }
    pub fn write(config: &Config) -> Result<(), ConfigError> {
        let s = toml::to_string(&config)?;

        let mut f = File::create("Config.toml")?;
        write!(f, "{s}")?;

        Ok(())
    }
}
