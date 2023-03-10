use std::{fmt, net::SocketAddr};

use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    username: String,
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
        &self.username
    }
    pub fn set_username<T>(&mut self, username: T)
    where
        T: Into<String>,
    {
        self.username = username.into()
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
    username: String,
    id: usize,
    addresses: Option<(SocketAddr, SocketAddr)>,
}

impl UserBuilder {
    pub fn username<T>(mut self, username: T) -> UserBuilder
    where
        T: Into<String>,
    {
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
