use std::net::TcpStream;

use egui::CentralPanel;

use crate::{chat_gui::ChatGui, config::gui::ConfigGui};

pub struct App {
    // Later maybe add functionality for more chats
    chat_ui: ChatGui,
    config_ui: ConfigGui,
    client: TcpStream,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let client = TcpStream::connect("127.0.0.1:1234").unwrap();
        client.set_nonblocking(true).unwrap();

        Self {
            chat_ui: ChatGui::new(),
            config_ui: ConfigGui::new().unwrap(),
            client,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: Only request repaint when a new message arrives
        ctx.request_repaint();

        CentralPanel::default().show(ctx, |_ui| {
            self.config_ui.update(&ctx, &mut self.client).unwrap();

            self.chat_ui.update(&ctx, &mut self.client).unwrap();
        });
    }
}
