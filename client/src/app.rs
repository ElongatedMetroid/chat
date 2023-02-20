use std::{
    io,
    net::TcpStream,
    process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chat_core::{
    message::{Message, Value},
    read::ChatReader,
    write::ChatWriter,
};
use egui::{CentralPanel, Key, Modifiers, ScrollArea, TextEdit, Window, Response};

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
                check_new_messages(client, messages).unwrap_or_else(|error| {
                    eprintln!("Error checking for new messages: {error}");
                    process::exit(1);
                });
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
        // TODO: Only request repaint when a new message arrives
        ctx.request_repaint();
        
        CentralPanel::default().show(ctx, |_ui| {
            Window::new("chat1").show(ctx, |ui| {
                // Messages scroll area
                let resopnse = ScrollArea::vertical()
                    .id_source("messages")
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() / 1.5)
                    .max_width(f32::INFINITY)
                    .show(ui, |ui| {
                        let messages = self.messages.lock().unwrap();
                        for message in messages.iter() {
                            ui.label(format!("{message}"));
                        }
                    });

                // /resopnse.

                ui.separator();

                // Send message text scroll area
                let response = ScrollArea::vertical()
                    .max_height(5.0)
                    .show(ui, |ui| {
                        ui.add(TextEdit::multiline(&mut self.message_text).desired_rows(2))
                    })
                    .inner;

                if ui.input(|i| i.modifiers.matches(Modifiers::SHIFT) && i.key_pressed(Key::Enter))
                {
                    self.message_text.push('\n');
                }

                if ui.input(|i| {
                    i.key_pressed(Key::Enter)
                        && response.has_focus()
                        && !i.modifiers.matches(Modifiers::SHIFT)
                }) {
                    let message = Message {
                        username: String::from("Joe Smoe"),
                        payload: Value::String(self.message_text.clone()),
                    };

                    println!("{message:?}");

                    self.client
                        .lock()
                        .unwrap()
                        .write_data(&message)
                        .unwrap();
                    self.message_text = String::new();
                }
            });
        });
    }
}

fn check_new_messages(client: Arc<Mutex<TcpStream>>, messages: Arc<Mutex<Vec<Message>>>) -> Result<(), bincode::Error> {
    loop {
        thread::sleep(Duration::from_millis(250));
        let message = client.lock().unwrap().read_data::<Message>();
        messages.lock().unwrap().push(match message {
            Ok(message) => message,
            Err(error) => match *error {
                bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                _ => return Err(error),
            },
        })
    }
}
