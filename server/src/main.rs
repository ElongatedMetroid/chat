use std::{
    io,
    net::{TcpListener, TcpStream},
    process,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::Duration,
};

use chat_core::{message::Message, read::ChatReader, request::Request, user::User, value::Value};
use lazy_static::lazy_static;
use rayon::ThreadPoolBuilder;
use server::broadcast::{BroadcastMessage, Broadcaster};

lazy_static! {
    static ref SERVER_USER: User = User::builder().id(0).username("SERVER").build();
}

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

    let message_broadcaster = Broadcaster::default().run();

    let tx = Arc::new(Mutex::new(message_broadcaster));

    let mut key = 0;
    for stream in listener.incoming() {
        key += 1;
        match stream {
            Ok(stream) => {
                let client = Arc::new(Mutex::new(stream));

                {
                    let client = Arc::clone(&client);
                    tx.lock()
                        .unwrap()
                        .send(BroadcastMessage::NewClient(client, key))
                        .unwrap();
                }

                pool.spawn({
                    let tx = Arc::clone(&tx);

                    move || handle_client(key, client, tx)
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
    client: Arc<Mutex<TcpStream>>,
    message_broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>,
) {
    /// This is strictly used only to make to sure that the
    /// client the corresponds to the key is removed from the
    /// HashMap on exiting this function. This is important
    /// because not removing a client will mean wasted memory
    /// and broadcasting messages would fail everytime.
    /// https://rust-unofficial.github.io/patterns/idioms/dtor-finally.html
    struct Exit {
        message_broadcaster: Arc<Mutex<Sender<BroadcastMessage>>>,
        key: usize,
    }
    impl Drop for Exit {
        fn drop(&mut self) {
            self.message_broadcaster
                .lock()
                .unwrap()
                .send(BroadcastMessage::RemoveClient(self.key))
                .unwrap();
            println!("Exiting handle_client()");
        }
    }

    let _exit = {
        let message_broadcaster = Arc::clone(&message_broadcaster);

        Exit {
            message_broadcaster,
            key,
        }
    };

    // Create a user
    let mut user = {
        // Get peer_address
        let peer_address = client.lock().unwrap().peer_addr();
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

    // Set client to non-blocking, later maybe do this with async
    let result = client.lock().unwrap().set_nonblocking(true);
    if let Err(error) = result {
        eprintln!("Failed to set non_blocking: {error}");
        return;
    }

    println!("New client connected: {user:?}");

    message_broadcaster
        .lock()
        .unwrap()
        .send(BroadcastMessage::ChatMessage(
            Message::builder()
                .from(SERVER_USER.clone())
                .payload(Value::String(format!("{user} has joined")))
                .build(),
        ))
        .unwrap();

    loop {
        thread::sleep(Duration::from_millis(100));
        // Read request
        let request = client.lock().unwrap().read_data::<Request>();

        let request = match request {
            Ok(request) => request,
            Err(error) => match *error {
                // No data to be read,
                bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    continue
                }
                _ => {
                    eprintln!("Bad request from client {:?}: {error}", user.addr());
                    return;
                }
            },
        };

        match request {
            Request::SendMessage(message) => {
                let message = Message::builder()
                    .from(user.hide_addr())
                    .payload(message)
                    .build();

                // Broadcast message to other clients
                message_broadcaster
                    .lock()
                    .unwrap()
                    .send(BroadcastMessage::ChatMessage(message))
                    .unwrap();
            }
            Request::ChangeUserName(username) => {
                message_broadcaster
                    .lock()
                    .unwrap()
                    .send(BroadcastMessage::ChatMessage(
                        Message::builder()
                            .from(SERVER_USER.clone())
                            .payload(Value::String(format!(
                                "Requesting change username. {user} -> {username}"
                            )))
                            .build(),
                    ))
                    .unwrap();

                user.set_username::<String>(match username.try_into() {
                    Ok(username) => username,
                    Err(error) => {
                        eprintln!("Cannot change username to non-string data: {error}");
                        return;
                    }
                });
            }
            Request::UserList => todo!(),
        }
    }
}
