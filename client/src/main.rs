use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use client::app::App;
use eframe::{run_native, NativeOptions};

fn main() {
    let native_options = NativeOptions::default();
    let messages = Arc::new(Mutex::new(Vec::new()));
    let client = Arc::new(Mutex::new(TcpStream::connect("127.0.0.1:1234").unwrap()));

    run_native(
        "chat",
        native_options,
        Box::new(|cc| Box::new(App::new(client, messages, cc))),
    )
    .unwrap();
}
