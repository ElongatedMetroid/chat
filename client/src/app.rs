use std::{process, io};

use egui::CentralPanel;

use crate::chat::Chat;

pub struct App {
    chat: Chat,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
    ) -> Self {
        Self {
            chat: Chat::new("127.0.0.1:1234").unwrap()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: Only request repaint when a new message arrives
        ctx.request_repaint();

        CentralPanel::default().show(ctx, |_ui| {
            if let Err(error) = self.chat.check_new_messages() {
                match *error {
                    bincode::ErrorKind::Io(e) if e.kind() == io::ErrorKind::WouldBlock => (),
                    _ => {
                        eprintln!("Error checking for new messages: {error}");
                        process::exit(1);
                    }
                }
            }
            self.chat.update(&ctx);
            // if !self.config_handled {
            //     match config {

            //     }
            //     match self.config.username() {
            //         // Username is set
            //         Some(username) => {

            //         },
            //         // Username is not set, and is not set to be random everytime
            //         None if !self.config.username().random_username() => {

            //         }
            //         // Username is set to be random everytime
            //         None if self.config.username().random_username() => {

            //         }
            //     }
            // } else {
                
            // }
        });
    }
}
