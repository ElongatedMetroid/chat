use std::{io, net::TcpStream};

use chat_core::{
    message::Message,
    read::ChatReader,
    request::{Request, Requesting},
    value::Value,
    write::ChatWriter,
};
use egui::{Key, Modifiers, ScrollArea, TextEdit, Window};

pub struct ChatGui {
    message_text: String,
    messages: Vec<Message>,
}

impl ChatGui {
    pub fn new() -> Self {
        Self {
            message_text: String::new(),
            messages: Vec::new(),
        }
    }
    /// This method is really messy, later organize this and make it return a result since it unwraps the value returned
    /// from write_data()
    pub fn update(
        &mut self,
        ctx: &egui::Context,
        client: &mut TcpStream,
    ) -> Result<(), bincode::Error> {
        // Check if there is a new message
        let message: Option<Message> = match client.read_data::<Message>() {
            Ok(message) => Some(message),
            Err(error) => match *error {
                bincode::ErrorKind::Io(error) if error.kind() == io::ErrorKind::WouldBlock => None,
                _ => return Err(error),
            },
        };
        // If there was a new message push it to our messages
        if let Some(message) = message {
            self.messages.push(message);
        }

        Window::new("chat1").show(ctx, |ui| {
            // Messages scroll area
            ScrollArea::vertical()
                .id_source("messages")
                .auto_shrink([false, false])
                .max_height(ui.available_height() / 1.5)
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    for message in &self.messages {
                        ui.label(format!("{message}"));
                    }
                });

            ui.separator();

            if self.message_text.bytes().len() as u64 > ChatWriter::byte_limit(client) {
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
                let request = Request::builder()
                    .requesting(Requesting::SendMessage)
                    .payload(Value::String(self.message_text.trim_end().to_owned()))
                    .build();

                client.write_data(&request).unwrap();

                self.message_text.clear();
            }
        });

        Ok(())
    }
}
