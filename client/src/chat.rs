use std::{net::{TcpStream, ToSocketAddrs}, io};

use chat_core::{message::Message, read::ChatReader, value::Value, request::{Requesting, Request}, write::ChatWriter};
use egui::{Window, ScrollArea, Modifiers, Key, TextEdit};

pub struct Chat {
    message_text: String,
    messages: Vec<Message>,
    client: TcpStream,
}

impl Chat {
    pub fn new<A>(addr: A) -> Result<Self, io::Error> 
    where A: ToSocketAddrs
    {
        let chat = Chat { message_text: String::new(), messages: Vec::new(), client: TcpStream::connect(addr)? };
        chat.client.set_nonblocking(true)?;
        Ok(chat)
    }
    /// This method is really messy, later organize this and make it return a result since it unwraps the value returned 
    /// from write_data()
    pub fn update(&mut self, ctx: &egui::Context) {
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

            if self.message_text.bytes().len() as u64 > ChatWriter::byte_limit(&self.client) {
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

            if ui.input(|i| i.modifiers.matches(Modifiers::SHIFT) && i.key_pressed(Key::Enter))
            {
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

                self.client.write_data(&request).unwrap();

                self.message_text.clear();
            }
        });
    }
    pub fn check_new_messages(&mut self) -> Result<(), bincode::Error> {
        self.messages.push(self.client.read_data::<Message>()?);
        Ok(())
    }
}

