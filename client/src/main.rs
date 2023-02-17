use std::net::TcpStream;

use chat_core::{write::ChatWriter, message::{Message, Value}};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:1234").unwrap();

    stream.write_message(Message {
        payload: Value::Integer(7) 
    }).unwrap();
}
