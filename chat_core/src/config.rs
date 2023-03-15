use std::{
    fmt,
    fs::{self, File},
    io::Write,
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ConfigError {
    SerError(toml::ser::Error),
    DeError(toml::de::Error),
    IoError(std::io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::DeError(error) => write!(f, "{error}"),
            ConfigError::IoError(error) => write!(f, "{error}"),
            ConfigError::SerError(error) => write!(f, "{error}"),
        }
    }
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

pub trait Config
where
    Self: Sized + for<'a> Deserialize<'a> + Serialize,
{
    fn load() -> Result<Self, ConfigError> {
        Ok(toml::from_str(&fs::read_to_string("Config.toml")?)?)
    }
    fn write(&self) -> Result<(), ConfigError> {
        let s = toml::to_string(&self)?;

        let mut f = File::create("Config.toml")?;
        write!(f, "{s}")?;

        Ok(())
    }
}
