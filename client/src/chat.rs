use chat_core::{
    read_write_streams::ReadWriteStreams, message::Message, read::ChatReader, request::Request,
    response::Response, value::Value, write::ChatWriter,
};
use egui::{Key, Modifiers, ScrollArea, TextEdit, Window};
use std::{
    process,
    sync::{Arc, Mutex},
    thread,
};

pub struct Chat {
    client_streams: ReadWriteStreams,
    message_text: String,
    messages: Arc<Mutex<Vec<Message>>>,
}

impl Chat {
    /// Create a new ChatGui, and start message checking thread
    pub fn new(client_streams: ReadWriteStreams) -> Self {
        let chat_gui = Self {
            client_streams,
            message_text: String::new(),
            messages: Arc::new(Mutex::new(Vec::new())),
        };

        chat_gui.start();

        chat_gui
    }
    /// Start a new thread that will check for new messages, and start a thread that will check for responses
    fn start(&self) {
        thread::spawn({
            let mut client_streams = self.client_streams.clone();
            let messages = self.messages.clone();
            move || loop {
                match client_streams.read_data::<Response>() {
                    Ok(response) => match response {
                        Ok(message) => messages.lock().unwrap().push(message),
                        Err(error) => {
                            eprintln!("Server returned error: {error}");
                            process::exit(1);
                        }
                    },
                    Err(error) => {
                        eprintln!("Error reading new message: {error}");
                        process::exit(1);
                    }
                }
            }
        });
    }
    /// Update gui
    pub fn update_gui(&mut self, ctx: &egui::Context) -> Result<(), bincode::Error> {
        Window::new("Chat").show(ctx, |ui| {
            // Messages scroll area
            ScrollArea::vertical()
                .id_source("messages")
                .auto_shrink([false, false])
                .max_height(ui.available_height() / 1.5)
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    for message in &*self.messages.lock().unwrap() {
                        ui.label(format!("{message}"));
                    }
                });

            ui.separator();

            if self.message_text.bytes().len() as u64 > <ReadWriteStreams as ChatWriter>::byte_limit()
            {
                ui.label("Message Too Long!");
                ui.separator();
            }

            // Send message text scroll area
            let response = ScrollArea::vertical()
                .max_height(5.0)
                .show(ui, |ui| {
                    ui.add(TextEdit::multiline(&mut self.message_text).desired_rows(2))
                })
                .inner;

            if ui.input(|i| i.modifiers.matches(Modifiers::SHIFT) && i.key_pressed(Key::Enter)) {
                self.message_text.push('\n');
            }

            if ui.input(|i| {
                i.key_pressed(Key::Enter)
                    && response.has_focus()
                    && !i.modifiers.matches(Modifiers::SHIFT)
            }) {
                self.client_streams
                    .write_data(&Request::SendMessage(Value::from(
                        self.message_text.trim_end(),
                    )))
                    .unwrap();

                self.message_text.clear();
            }
        });

        Ok(())
    }
}
