use std::{
    collections::HashMap,
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

use chat_core::{
    read::ChatReader,
    request::{Request, Requesting},
    value::Value,
    write::ChatBroadcaster,
};
use rayon::ThreadPoolBuilder;
use server::user::User;

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
    tx: Arc<Mutex<mpsc::Sender<Value>>>,
) {
    let user = {
        let peer_address = match clients.lock().unwrap().get(&key).unwrap().peer_addr() {
            Ok(peer_address) => peer_address,
            Err(error) => {
                eprintln!("Failed to get peer_address: {error}");
                clients.lock().unwrap().remove(&key);
                return;
            }
        };

        User::builder()
            .username(format!("anonymous-{}", User::random_name()))
            .id(key)
            .address(peer_address)
            .build()
    };

    if let Err(error) = clients
        .lock()
        .unwrap()
        .get(&key)
        .unwrap()
        .set_nonblocking(true)
    {
        eprintln!("Failed to set non_blocking: {error}");
        clients.lock().unwrap().remove(&key);
        return;
    }

    println!("New client connected: {user:?}");

    loop {
        thread::sleep(Duration::from_millis(250));
        // Read message
        let request = clients
            .lock()
            .unwrap()
            .get_mut(&key)
            .unwrap()
            .read_data::<Request>();

        let mut request = match request {
            Ok(request) => request,
            Err(error) => match *error {
                bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    continue
                }
                _ => {
                    eprintln!("Dropped/Lost connection to client {}: {error}", user.addr());
                    break;
                }
            },
        };

        match request.requesting() {
            Requesting::SendMessage => {
                // Broadcast message to other clients
                let result = tx.lock().unwrap().send(match request.payload() {
                    Some(message) => message,
                    None => {
                        eprintln!(
                            "Client {} tried to send a message with a payload of `Option::None`",
                            user.addr()
                        );
                        break;
                    }
                });

                if let Err(error) = result {
                    eprintln!(
                        "Client {} failed to broadcast message: {error}",
                        user.addr()
                    );
                    break;
                }
            }
            Requesting::ChangeUserName => todo!(),
            Requesting::UserList => todo!(),
        }
    }

    clients.lock().unwrap().remove(&key);
}

fn broadcast_messages(rx: Receiver<Value>, clients: Arc<Mutex<HashMap<usize, TcpStream>>>) {
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
