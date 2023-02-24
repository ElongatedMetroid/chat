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
    message::Message,
    read::ChatReader,
    request::{Request, Requesting},
    user::User,
    value::Value,
};
use rayon::ThreadPoolBuilder;
use server::broadcast::ChatBroadcaster;

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

    // This is used to send messages to a different thread that will handle broadcasting the message
    let (message_broadcaster, message_reciever) = mpsc::channel();

    // Broadcasting thread
    thread::spawn({
        let clients = Arc::clone(&clients);

        move || {
            broadcast_messages(message_reciever, clients);
        }
    });

    let tx = Arc::new(Mutex::new(message_broadcaster));

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
    message_broadcaster: Arc<Mutex<mpsc::Sender<Message>>>,
) {
    /// This is strictly used only to make to sure that the
    /// client the corresponds to the key is removed from the
    /// HashMap on exiting this function. This is important
    /// because not removing a client will mean wasted memory
    /// and broadcasting messages would fail everytime.
    /// https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
    struct Exit {
        clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
        key: usize,
    }
    impl Drop for Exit {
        fn drop(&mut self) {
            self.clients.lock().unwrap().remove(&self.key);
            println!("Exiting handle_client()");
        }
    }

    let _exit = {
        let clients = Arc::clone(&clients);

        Exit { clients, key }
    };

    // Create a user
    let user = {
        // Get peer_address
        let peer_address = clients.lock().unwrap().get(&key).unwrap().peer_addr();
        let peer_address = match peer_address {
            Ok(peer_address) => peer_address,
            Err(error) => {
                eprintln!("Failed to get peer_address: {error}");
                return;
            }
        };

        // Build a user with a random name
        User::builder()
            .username(format!("anonymous-{}", User::random_name()))
            .id(key)
            .address(Some(peer_address))
            .build()
    };

    // Set client to non-blocking
    let result = clients
        .lock()
        .unwrap()
        .get(&key)
        .unwrap()
        .set_nonblocking(true);
    if let Err(error) = result {
        eprintln!("Failed to set non_blocking: {error}");
        return;
    }

    println!("New client connected: {user:?}");

    let join_message = Message::builder()
        .from(
            User::builder()
                .username(String::from("SERVER"))
                .id(0)
                .build(),
        )
        .payload(Value::String(format!("{user} has joined")))
        .build();

    let result = message_broadcaster.lock().unwrap().send(join_message);

    if let Err(error) = result {
        eprintln!("Failed to broadcast join message: {error}");
        return;
    }

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
                    eprintln!(
                        "Dropped/Lost connection to client {:?}: {error}",
                        user.addr()
                    );
                    return;
                }
            },
        };

        match request.requesting() {
            Requesting::SendMessage => {
                let message = match request.payload() {
                    Some(payload) => Message::builder()
                        .from(user.hide_addr())
                        .payload(payload)
                        .build(),
                    None => {
                        eprintln!(
                            "Client {:?} tried to send a message with a payload of `Option::None`",
                            user.addr()
                        );
                        return;
                    }
                };

                // Broadcast message to other clients
                let result = message_broadcaster.lock().unwrap().send(message);

                if let Err(error) = result {
                    eprintln!(
                        "Failed to broadcast message from {:?}: {error}",
                        user.addr()
                    );
                    return;
                }
            }
            Requesting::ChangeUserName => todo!(),
            Requesting::UserList => todo!(),
        }
    }
}

// create broadcast module with broadcast struct and stuff
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
