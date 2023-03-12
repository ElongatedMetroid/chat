use std::{
    net::{TcpListener, TcpStream},
    process,
    sync::{Arc, Mutex},
};

use chat_core::client_streams::ClientStreams;
use rayon::ThreadPoolBuilder;
use server::{broadcast::Broadcaster, client::Client};

fn main() {
    // TODO move most of the code in main to a seperate client listener module
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|e| {
        eprintln!("Failed to bind TcpListener: {e}");
        process::exit(1);
    });
    // TODO send a response to the client if the room is full.
    let pool = ThreadPoolBuilder::new()
        .num_threads(20)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to create threadpool: {e}");
            process::exit(1);
        });

    // Start the message broadcaster on a seperate thread, the sender needs to be inside an Arc<Mutex<T>> since it will be shared between
    // all client threads.
    let message_broadcaster = Arc::new(Mutex::new(Broadcaster::default().run()));

    let mut key = 0;
    for stream in listener.incoming() {
        key += 1;
        match stream {
            Ok(write_stream) => {
                // Get the address of the stream that connected
                let mut read_port = match write_stream.peer_addr() {
                    Ok(port) => port,
                    Err(error) => {
                        eprintln!("Failed to get peer_addr!: {error}");
                        return;
                    }
                };
                // Set the port to the reading port (the client has opened this port, and is waiting for a connection on it)
                read_port.set_port(4321);
                let read_stream = match TcpStream::connect(read_port) {
                    Ok(stream) => stream,
                    Err(error) => {
                        eprintln!("Error connecting to server read stream (client write): {error}");
                        return;
                    }
                };
                // Create a new ClientStreams enum with the Server variant
                let client_streams = ClientStreams::Server(
                    Arc::new(Mutex::new(read_stream)),
                    Arc::new(Mutex::new(write_stream)),
                );

                eprintln!("Estabilished Connection: {client_streams:#?}");

                let mut client = Client::new(key, client_streams);

                pool.spawn({
                    let message_broadcaster = Arc::clone(&message_broadcaster);
                    move || client.run(message_broadcaster)
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
            }
        }
    }
}
