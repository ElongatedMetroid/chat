use std::net::TcpStream;

use crate::message::Message;

pub trait ChatWriter {
    fn write_message(&mut self, message: Message) -> Result<(), Box<dyn std::error::Error>>;
}

impl ChatWriter for TcpStream {
    fn write_message(&mut self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        bincode::serialize_into(self, &message)?;

        Ok(())
    }
}
