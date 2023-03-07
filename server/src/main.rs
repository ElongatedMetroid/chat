use std::{
    net::TcpListener,
    process,
    sync::{Arc, Mutex},
};

use rayon::ThreadPoolBuilder;
use server::{
    broadcast::{BroadcastMessage, Broadcaster},
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

    let message_broadcaster = Broadcaster::default().run();

    let tx = Arc::new(Mutex::new(message_broadcaster));

    let mut key = 0;
    for stream in listener.incoming() {
        key += 1;
        match stream {
            Ok(stream) => {
                let client = Client::new(key, Arc::new(Mutex::new(stream)));

                {
                    let client = client.clone();
                    tx.lock()
                        .unwrap()
                        .send(BroadcastMessage::AddClient(client, key))
                        .unwrap();
                }

                pool.spawn({
                    let tx = Arc::clone(&tx);

                    move || client.run(tx)
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
            }
        }
    }
}
