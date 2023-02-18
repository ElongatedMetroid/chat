use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread, io, time::Duration,
};

use chat_core::{
    message::{Message, Value},
    read::ChatReader,
    write::ChatWriter,
};
use egui::{CentralPanel, Key, ScrollArea, TextEdit, Window};

pub struct App {
    client: Arc<Mutex<TcpStream>>,
    messages: Arc<Mutex<Vec<Message>>>,
    message_text: String,
}

impl App {
    pub fn new(
        client: Arc<Mutex<TcpStream>>,
        messages: Arc<Mutex<Vec<Message>>>,
        _cc: &eframe::CreationContext<'_>,
    ) -> Self {
        client.lock().unwrap().set_nonblocking(true).unwrap();

        thread::spawn({
            let client = Arc::clone(&client);
            let messages = Arc::clone(&messages);
            move || {
                loop {
                    thread::sleep(Duration::from_millis(500));
                    let message = client.lock().unwrap().read_message();
                    let message = match message {
                        Ok(message) => message,
                        Err(error) => {
                            match *error {
                                bincode::ErrorKind::Io(error) => {
                                    if error.kind() == io::ErrorKind::WouldBlock {
                                        continue
                                    }
                                    eprintln!("Io Error: {error}");
                                    break;
                                },
                                _ => {
                                    eprintln!("Lost connection to server: {error}");
                                    break;
                                },
                            }
                        },
                    };

                    messages.lock().unwrap().push(message);
                }
            }
        });

        Self {
            client,
            messages,
            message_text: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            Window::new("chat1").show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_source("messages")
                    .max_height(ui.available_height() / 1.5)
                    .max_width(f32::INFINITY)
                    .show(ui, |ui| {
                        let messages = self.messages.lock().unwrap();
                        for message in messages.iter() {
                            ui.label(format!("{message}"));
                        }
                    });

                ui.separator();

                let response = ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        ui.add(TextEdit::multiline(&mut self.message_text).desired_rows(4))
                    })
                    .inner;

                if ui.ctx().input(|i| i.key_pressed(Key::Enter)) && response.has_focus() {
                    self.client
                        .lock()
                        .unwrap()
                        .write_message(&Message {
                            username: String::from("Joe Smoe"),
                            payload: Value::String(self.message_text.clone()),
                        })
                        .unwrap();
                    self.message_text = String::new();
                }
            });
        });
    }
}