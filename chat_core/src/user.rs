use std::{
    fmt::{self, Display},
    net::SocketAddr,
};

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::value::Array;

use crate::{guidelines::AgainstGuidelines, value::Value};

// To have the names loaded at runtime since (A) there is not need to edit them, and (B) there is not a good place to
// load them in.
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
    text_only: bool,
}

impl Default for UsernameGuidelines {
    fn default() -> Self {
        Self {
            max_length: 10,
            min_length: 3,
            whitespace: false,
            text_only: true,
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
    pub fn text_only(&self) -> bool {
        self.text_only
    }
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum UsernameError {
    #[error("tried to change username to non-text data")]
    TextOnly,
    #[error("username is too long")]
    TooLong,
    #[error("username is too short")]
    TooShort,
    #[error("username cannot contain whitespace")]
    Whitespace,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Username {
    name: Value,
}

impl Default for Username {
    fn default() -> Self {
        Self {
            name: Value::String(String::new()),
        }
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Username {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<Value>,
    {
        Self { name: name.into() }
    }
}

impl AgainstGuidelines<UsernameGuidelines> for Username {
    type Error = UsernameError;

    fn against_guidelines(self, guidelines: &UsernameGuidelines) -> Result<Self, Self::Error> {
        if let Value::String(name) = &self.name {
            if name.len() > guidelines.max_length() {
                return Err(UsernameError::TooLong);
            } else if name.len() < guidelines.min_length() {
                return Err(UsernameError::TooShort);
            } else if !guidelines.whitespace() && name.contains([' ', '\n', '\t', '\r']) {
                return Err(UsernameError::Whitespace);
            }
        } else {
            if guidelines.text_only {
                return Err(UsernameError::TextOnly);
            }
        }

        Ok(self)
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
    pub fn username(&self) -> &Username {
        &self.username
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
        self.username = username;
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
