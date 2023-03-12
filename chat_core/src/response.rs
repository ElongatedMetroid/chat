use serde::{Deserialize, Serialize};

use crate::message::Message;

#[derive(Serialize, Deserialize)]
pub enum Response {
    Message(Message),
    Error(String),
}
