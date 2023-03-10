use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use bincode::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};

use crate::{read::ChatReader, write::ChatWriter};

#[derive(Debug, Clone)]
pub enum ClientStreams {
    /// 0 = Read stream, 1 = Write Stream
    Client(Arc<Mutex<TcpStream>>, Arc<Mutex<TcpStream>>),
    /// 0 = Read stream, 1 = Write Stream
    Server(Arc<Mutex<TcpStream>>, Arc<Mutex<TcpStream>>),
}

impl ClientStreams {
    pub fn peer_addrs(
        &mut self,
    ) -> (
        Result<std::net::SocketAddr, std::io::Error>,
        Result<std::net::SocketAddr, std::io::Error>,
    ) {
        match self {
            ClientStreams::Client(read, write) => (
                read.lock().unwrap().peer_addr(),
                write.lock().unwrap().peer_addr(),
            ),
            ClientStreams::Server(read, write) => (
                read.lock().unwrap().peer_addr(),
                write.lock().unwrap().peer_addr(),
            ),
        }
    }
}

impl ChatReader for ClientStreams {
    fn read_data<T>(&mut self) -> Result<T, bincode::Error>
    where
        T: serde::Serialize + for<'a> serde::Deserialize<'a>,
    {
        let read = &*match self {
            ClientStreams::Client(read, _) => read,
            ClientStreams::Server(read, _) => read,
        }.lock().unwrap();

        let data = DefaultOptions::new()
            .with_limit(<Self as ChatReader>::byte_limit())
            .deserialize_from(read)?;

        Ok(data)
    }
}

impl ChatWriter for ClientStreams {
    /// Write any data type that implements Serialize and Deserialize to Self
    fn write_data<T>(&mut self, data: &T) -> Result<(), bincode::Error>
    where
        T: Serialize + for<'a> Deserialize<'a>,
    {
        let write = &*match self {
            ClientStreams::Client(_, write) => write,
            ClientStreams::Server(_, write) => write,
        }.lock().unwrap();

        DefaultOptions::new()
            .with_limit(<Self as ChatWriter>::byte_limit())
            .serialize_into(write, &data)?;

        Ok(())
    }
}
