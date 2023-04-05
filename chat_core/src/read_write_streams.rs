use std::{
    net::TcpStream,
    sync::{Arc, Mutex}, io,
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

pub trait ConnectPeerStream {
    fn connect_peer_stream(self, port: u16) -> Result<ReadWriteStreams, io::Error>;
}

impl ConnectPeerStream for TcpStream {
    /// Create a `ReadWriteStream` from a given `TcpStream`. The given stream will be used as the writing port, using that streams address
    /// (with the port changed to `port`) this method attempts to connect to a `TcpStream` for reading.
    /// # Errors
    /// This method will return an error under two circumstances, first if it fails to get the address of `self`, and two if it fails to 
    /// connect to the new address created.
    fn connect_peer_stream(self, port: u16) -> Result<ReadWriteStreams, io::Error> {
        log::info!("creating a new stream");
        // Get the address of the stream
        let mut addr = self.peer_addr()?;
        // Set the port to the reading port (the client should have opened this port, and is waiting for a connection on it)
        addr.set_port(port);
        log::debug!("attempting to connect to {addr}");
        // Try to connect to the address we set as the read address
        let read_stream = TcpStream::connect(addr)?;
        // Create a new ReadWriteStreams enum with the Server variant
        Ok(ReadWriteStreams {
            read: Arc::new(Mutex::new(read_stream)),
            write: Arc::new(Mutex::new(self)),
        })
    }
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
