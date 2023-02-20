use std::{net::TcpStream, collections::HashMap};

use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};

use crate::message::Message;

pub trait ChatWriter {
    fn write_data<T>(&mut self, data: &T) -> Result<(), bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>;
    fn byte_limit(&self) -> u64 {
        4000
    }
}

impl ChatWriter for TcpStream {
    /// Write any data type that implements Serialize and Deserialize to Self
    fn write_data<T>(&mut self, data: &T) -> Result<(), bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        DefaultOptions::new()
            .with_limit(self.byte_limit())
            .serialize_into(self, &data)?;

        Ok(())
    }
}

pub trait ChatBroadcaster {
    fn broadcast(&mut self, message: &Message) -> Result<(), bincode::Error>;
}

impl<T> ChatBroadcaster for HashMap<T, TcpStream> {
    fn broadcast(&mut self, message: &Message) -> Result<(), bincode::Error> {
        for client in self.values_mut() {
            client.write_data(message)?;
        }

        Ok(())
    }
}

impl ChatBroadcaster for Vec<TcpStream> {
    /// Broadcast a Message to all TcpStreams in the Vec
    fn broadcast(&mut self, message: &Message) -> Result<(), bincode::Error> {
        for client in self {
            client.write_data(message)?;
        }

        Ok(())
    }
}
