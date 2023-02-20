use std::{
    io,
    net::{TcpListener, TcpStream},
    process,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration, collections::HashMap,
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
    let clients = Arc::new(Mutex::new(HashMap::new()));

    let (tx, rx) = mpsc::channel();

    // Broadcasting thread
    thread::spawn({
        let clients = Arc::clone(&clients);

        move || {
            broadcast_messages(rx, clients);
        }
    });

    let tx = Arc::new(Mutex::new(tx));

    let mut key = 0;
    for stream in listener.incoming() {
        key += 1;
        match stream {
            Ok(stream) => {
                clients.lock().unwrap().insert(key, stream);

                pool.spawn({
                    let clients = Arc::clone(&clients);
                    let tx = Arc::clone(&tx);

                    move || handle_client(key, clients, tx)
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {e}");
            }
        }
    }
}

fn handle_client(
    key: usize,
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
    tx: Arc<Mutex<mpsc::Sender<Message>>>,
) {
    let peer_address = match clients.lock().unwrap().get(&key).unwrap().peer_addr() {
        Ok(peer_address) => peer_address,
        Err(error) => {
            eprintln!("Failed to get peer_address: {error}");
            clients.lock().unwrap().remove(&key);
            return;
        }
    };
    if let Err(error) = clients.lock().unwrap().get(&key).unwrap().set_nonblocking(true) {
        eprintln!("Failed to set {peer_address} to non_blocking: {error}");
        clients.lock().unwrap().remove(&key);
        return;
    }

    println!("New client connected: {}, {}", peer_address, key);

    loop {
        thread::sleep(Duration::from_millis(250));
        // Read message
        let message = clients.lock().unwrap().get_mut(&key).unwrap().read_data();

        let message = match message {
            Ok(message) => message,
            Err(error) => match *error {
                bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    continue
                }
                _ => {
                    eprintln!("Dropped/Lost connection to client {peer_address}: {error}");
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

    clients.lock().unwrap().remove(&key);
}

fn broadcast_messages(rx: Receiver<Message>, clients: Arc<Mutex<HashMap<usize, TcpStream>>>) {
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
