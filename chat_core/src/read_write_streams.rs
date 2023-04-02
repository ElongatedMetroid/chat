use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};

use crate::{read::ChatReader, write::ChatWriter};

/// Structure to hold two `Arc<Mutex<TcpStream>>`'s.
/// # Usage
/// This structure is so you can simultainiously read and write to something, for example if you wanted to read messages
/// and also send messages at the same time
#[derive(Debug, Clone)]
pub struct ReadWriteStreams {
    pub read: Arc<Mutex<TcpStream>>,
    pub write: Arc<Mutex<TcpStream>>,
}

impl ReadWriteStreams {
    pub fn peer_addrs(
        &mut self,
    ) -> (
        Result<std::net::SocketAddr, std::io::Error>,
        Result<std::net::SocketAddr, std::io::Error>,
    ) {
        (
            self.read.lock().unwrap().peer_addr(),
            self.write.lock().unwrap().peer_addr(),
        )
    }
}

impl ChatReader for ReadWriteStreams {
    fn read_data<T>(&mut self) -> Result<T, bincode::Error>
    where
        T: serde::Serialize + for<'a> serde::Deserialize<'a>,
    {
        let data = DefaultOptions::new()
            .with_limit(<Self as ChatReader>::byte_limit())
            .deserialize_from(&*self.read.lock().unwrap())?;

        Ok(data)
    }
}

impl ChatWriter for ReadWriteStreams {
    /// Write any data type that implements Serialize and Deserialize to Self
    fn write_data<T>(&mut self, data: &T) -> Result<(), bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        DefaultOptions::new()
            .with_limit(<Self as ChatWriter>::byte_limit())
            .serialize_into(&*self.write.lock().unwrap(), &data)?;

        Ok(())
    }
}
