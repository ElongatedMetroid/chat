use std::{
    net::{TcpListener, TcpStream},
    process,
    sync::{Arc, Mutex},
};

use chat_core::read::ChatReader;
use rayon::ThreadPoolBuilder;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|e| {
        eprintln!("Failed to bind TcpListener: {e}");
        process::exit(1);
    });
    let pool = ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to create threadpool: {e}");
            process::exit(1);
        });
    let clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Get index of where our client will be before pushing it since indexing starts at 0
                let index = clients.lock().unwrap().len();
                clients.lock().unwrap().push(stream);

                pool.install(|| handle_client(index, clients.clone()));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
            }
        }
    }
}

fn handle_client(index: usize, clients: Arc<Mutex<Vec<TcpStream>>>) {
    loop { 
        // Read message
        match clients.lock().unwrap()[index].read_message() {
            Ok(message) => {
                println!("{:#?}", message);
            },
            Err(error) => {
                break;
            },
        }
        // Broadcast message
    }

    clients.lock().unwrap().remove(index);
}
