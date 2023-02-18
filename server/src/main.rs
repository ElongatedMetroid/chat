use std::{
    io,
    net::{TcpListener, TcpStream},
    process,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use chat_core::{message::Message, read::ChatReader, write::ChatBroadcaster};
use rayon::ThreadPoolBuilder;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap_or_else(|e| {
        eprintln!("Failed to bind TcpListener: {e}");
        process::exit(1);
    });
    let pool = ThreadPoolBuilder::new()
        .num_threads(20)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to create threadpool: {e}");
            process::exit(1);
        });
    let clients = Arc::new(Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel();

    // Broadcasting thread
    thread::spawn({
        let clients = Arc::clone(&clients);

        move || {
            broadcast_messages(rx, clients);
        }
    });

    let tx = Arc::new(Mutex::new(tx));
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Get index of where our client will be before pushing it since indexing starts at 0
                let index = clients.lock().unwrap().len();
                clients.lock().unwrap().push(stream);

                pool.spawn({
                    let clients = Arc::clone(&clients);
                    let tx = Arc::clone(&tx);

                    move || handle_client(index, clients, tx)
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
            }
        }
    }
}

fn handle_client(
    index: usize,
    clients: Arc<Mutex<Vec<TcpStream>>>,
    tx: Arc<Mutex<mpsc::Sender<Message>>>,
) {
    clients.lock().unwrap()[index]
        .set_nonblocking(true)
        .unwrap();
    let peer_address = clients.lock().unwrap()[index].peer_addr().unwrap();
    println!("New client connected: {}", peer_address);

    loop {
        thread::sleep(Duration::from_millis(500));
        // Read message
        let message = clients.lock().unwrap()[index].read_message();
        let message = match message {
            Ok(message) => message,
            Err(error) => match *error {
                bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    continue
                }
                _ => {
                    eprintln!("Lost connection to client {peer_address}: {error}");
                    break;
                }
            },
        };

        // Broadcast message to other clients
        let result = tx.lock().unwrap().send(message);

        if let Err(error) = result {
            eprintln!("Client {peer_address} failed to broadcast message: {error}");
            break;
        }
    }

    clients.lock().unwrap().remove(index);
}

fn broadcast_messages(rx: Receiver<Message>, clients: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        let message = match rx.recv() {
            Ok(message) => message,
            Err(error) => {
                eprintln!("Recieving message failed: {error}");
                continue;
            }
        };

        let result = clients.lock().unwrap().broadcast(&message);

        if let Err(error) = result {
            eprintln!("Failed to broadcast message: {error}");
            continue;
        };
    }
}
