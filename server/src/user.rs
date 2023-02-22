use std::net::SocketAddr;

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

#[derive(Debug)]
pub struct User {
    username: String,
    id: usize,
    address: SocketAddr,
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
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn id(&self) -> usize {
        self.id
    }
    pub fn addr(&self) -> &SocketAddr {
        &self.address
    }
}

#[derive(Default)]
pub struct UserBuilder {
    username: String,
    id: usize,
    address: Option<SocketAddr>,
}

impl UserBuilder {
    pub fn username(mut self, username: String) -> UserBuilder {
        self.username = username;
        self
    }
    pub fn id(mut self, id: usize) -> UserBuilder {
        self.id = id;
        self
    }
    pub fn address(mut self, address: SocketAddr) -> UserBuilder {
        self.address = Some(address);
        self
    }
    pub fn build(self) -> User {
        User {
            username: self.username,
            id: self.id,
            address: self.address.unwrap(),
        }
    }
}
