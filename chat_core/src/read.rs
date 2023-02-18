use std::net::TcpStream;

use serde::{Deserialize, Serialize};

pub trait ChatReader {
    fn read_data<T>(&mut self) -> Result<T, bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>;
}

impl ChatReader for TcpStream {
    /// Read any data that implements Serialize and Deserialize from Self
    fn read_data<T>(&mut self) -> Result<T, bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let chat = bincode::deserialize_from(self)?;

        Ok(chat)
    }
}
