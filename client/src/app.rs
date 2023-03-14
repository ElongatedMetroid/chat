use std::{
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use chat_core::client_streams::ClientStreams;
use egui::CentralPanel;

use crate::{chat::Chat, config::gui::ConfigGui};

pub struct App {
    // Later maybe add functionality for more chats
    chat: Chat,
    config: ConfigGui,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let read = TcpStream::connect("127.0.0.1:1234").unwrap();

        let mut write_port = read.local_addr().unwrap();
        write_port.set_port(4321);

        let write = TcpListener::bind(write_port).unwrap().accept().unwrap().0;

        let client_streams =
            ClientStreams::Client(Arc::new(Mutex::new(read)), Arc::new(Mutex::new(write)));

        eprintln!("Estabilished Connection: {client_streams:#?}");

        Self {
            chat: Chat::new(client_streams.clone()),
            config: ConfigGui::new(client_streams).unwrap(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: Only request repaint when a new message arrives
        ctx.request_repaint();

        CentralPanel::default().show(ctx, |_ui| {
            self.config.update_gui(&ctx).unwrap();

            self.chat.update_gui(&ctx).unwrap();
        });
    }
}
