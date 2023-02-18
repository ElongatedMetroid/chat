use std::net::TcpStream;

use crate::message::Message;

pub trait ChatWriter {
    fn write_message(&mut self, message: &Message) -> Result<(), bincode::Error>;
}

impl ChatWriter for TcpStream {
    fn write_message(&mut self, message: &Message) -> Result<(), bincode::Error> {
        bincode::serialize_into(self, &message)?;

        Ok(())
    }
}

pub trait ChatBroadcaster {
    fn broadcast(&mut self, message: &Message) -> Result<(), bincode::Error>;
}

impl ChatBroadcaster for Vec<TcpStream> {
    fn broadcast(&mut self, message: &Message) -> Result<(), bincode::Error> {
        for client in self {
            client.write_message(&message)?;
        }

        Ok(())
    }
}
