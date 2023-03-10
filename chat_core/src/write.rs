use std::net::TcpStream;

use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};

pub trait ChatWriter {
    fn write_data<T>(&mut self, data: &T) -> Result<(), bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>;
    fn byte_limit() -> u64 {
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
            .with_limit(Self::byte_limit())
            .serialize_into(self, &data)?;

        Ok(())
    }
}
