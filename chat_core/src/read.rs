use std::net::TcpStream;

use bincode::{Options, DefaultOptions};
use serde::{Deserialize, Serialize};

pub trait ChatReader {
    fn read_data<T>(&mut self) -> Result<T, bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>;
    fn byte_limit(&self) -> u64 {
        4000
    }
}

impl ChatReader for TcpStream {
    /// Read any data that implements Serialize and Deserialize from Self
    fn read_data<T>(&mut self) -> Result<T, bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        Ok(DefaultOptions::new()
            .with_limit(self.byte_limit())
            .deserialize_from(self)?)
    }
}
