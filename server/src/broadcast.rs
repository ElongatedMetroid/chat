use std::{collections::HashMap, net::TcpStream};

use chat_core::{message::Message, write::ChatWriter};

pub trait ChatBroadcaster {
    fn broadcast(&mut self, message: &Message) -> Result<(), bincode::Error>;
}

impl<T> ChatBroadcaster for HashMap<T, TcpStream> {
    /// Broadcast a
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
