use std::{
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use chat_core::read_write_streams::ReadWriteStreams;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new().init().unwrap();

    let read = TcpStream::connect("127.0.0.1:1234").unwrap();
    log::info!("server should have now got our connection, and a new process was spawned in the thread pool that runs the `Client::make_connection()` function then if that is successful runs the `Client::run()` method on it");
    let mut write_addr = read.local_addr().unwrap();
    write_addr.set_port(4321);
    log::info!("the server should now be in the process of making a new client, it should be currently trying to connect to `peer_addr():(the port set in the config)`");
    let write = TcpListener::bind(write_addr).unwrap().accept().unwrap().0;
    log::info!("the server should now have created the client and is now in the `Client::run()` method.");

    let streams = ReadWriteStreams {
        read: Arc::new(Mutex::new(read)),
        write: Arc::new(Mutex::new(write)),
    };
    
    log::info!("on the stream there should be data waiting, the server should have broadcasted a welcome message");
    log::info!("the server is now waiting for a request in the `Request` type");
    log::info!("now this is where it gets interesting. lets send some data that is not of the `Request` type.");
}
