use std::{
    net::TcpListener,
    process,
    sync::{Arc, Mutex},
};

use rayon::ThreadPoolBuilder;
use server::{
    broadcast::Broadcaster,
    client::Client,
};

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
            Ok(stream) => {
                let client = Client::new(key, Arc::new(Mutex::new(stream)));

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
