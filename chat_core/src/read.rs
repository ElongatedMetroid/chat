use std::net::TcpStream;

use crate::message::Message;

pub trait ChatReader {
    fn read_message(&mut self) -> Result<Message, Box<dyn std::error::Error>>;
}

impl ChatReader for TcpStream {
    fn read_message(&mut self) -> Result<Message, Box<dyn std::error::Error>> {
        let message: Message = bincode::deserialize_from(self)?;

        Ok(message)
    }
}
