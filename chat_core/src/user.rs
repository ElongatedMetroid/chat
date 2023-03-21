use std::{fmt::{self, Display}, net::SocketAddr};

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::value::Array;

lazy_static! {
    static ref NAMES: Vec<toml::Value> = {
        #[derive(Deserialize, Serialize)]
        struct NamesToml {
            names: Array,
        }
        let names_str = include_str!("../names.toml");
        toml::from_str::<NamesToml>(names_str).unwrap().names
    };
}

#[derive(Serialize, Deserialize)]
pub struct UsernameGuidelines {
    max_length: usize,
    min_length: usize,
    whitespace: bool,
}

impl Default for UsernameGuidelines {
    fn default() -> Self {
        Self {
            max_length: 10,
            min_length: 3,
            whitespace: false,
        }
    }
}

impl UsernameGuidelines {
    pub fn max_length(&self) -> usize {
        self.max_length
    }
    pub fn min_length(&self) -> usize {
        self.min_length
    }
    pub fn whitespace(&self) -> bool {
        self.whitespace
    }
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum UsernameError {
    #[error("tried to change username to non-string data")]
    NonStringData,
    #[error("username is too long")]
    TooLong,
    #[error("username is too short")]
    TooShort,
    #[error("username cannot contain whitespace")]
    Whitespace,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Username {
    name: String,
}

impl Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Username {
    pub fn new<T>(guidelines: Option<&UsernameGuidelines>, name: T) -> Result<Self, UsernameError> 
    where T: TryInto<String>
    {
        let name: String = match name.try_into() {
            Ok(name) => name,
            Err(_) => return Err(UsernameError::NonStringData),
        };

        if let Some(guidelines) = guidelines {
            if name.len() > guidelines.max_length() {
                return Err(UsernameError::TooLong)
            } else if name.len() < guidelines.min_length() {
                return Err(UsernameError::TooShort)
            } else if !guidelines.whitespace() && name.contains([' ', '\n', '\t', '\r']) {
                return Err(UsernameError::Whitespace)
            }
        }
        
        Ok(Self {
            name
        })
    }
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    username: Username,
    id: usize,
    addresses: Option<(SocketAddr, SocketAddr)>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.username(), self.id())
    }
}

impl User {
    pub fn builder() -> UserBuilder {
        UserBuilder::default()
    }
    /// Picks a random name from a file
    pub fn random_name() -> &'static str {
        NAMES
            .choose(&mut rand::thread_rng())
            .unwrap()
            .as_str()
            .unwrap()
    }
    /// Creates a CLONE of Self with the address field set to None
    pub fn hide_addr(&self) -> User {
        let mut new = self.clone();
        new.addresses = None;
        new
    }
    pub fn username(&self) -> &str {
        self.username.as_str()
    }
    pub fn set_username(&mut self, username: Username) {
        self.username = username
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn addrs(&self) -> &Option<(SocketAddr, SocketAddr)> {
        &self.addresses
    }
}

#[derive(Default)]
pub struct UserBuilder {
    username: Username,
    id: usize,
    addresses: Option<(SocketAddr, SocketAddr)>,
}

impl UserBuilder {
    pub fn username(mut self, username: Username) -> UserBuilder {
        self.username = username.into();
        self
    }
    pub fn id(mut self, id: usize) -> UserBuilder {
        self.id = id;
        self
    }
    pub fn addresses(mut self, addresses: Option<(SocketAddr, SocketAddr)>) -> UserBuilder {
        self.addresses = addresses;
        self
    }
    pub fn build(self) -> User {
        User {
            username: self.username,
            id: self.id,
            addresses: self.addresses,
        }
    }
}
