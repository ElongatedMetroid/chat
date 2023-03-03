use serde::{Deserialize, Serialize};

use crate::value::Value;

#[derive(Debug, Deserialize, Serialize)]
pub enum Request {
    /// Treat the payload as a message
    SendMessage(Value),
    /// Treat the payload as a new username
    ChangeUserName(Value),
    /// Give the client a List of users connected to server
    UserList,
}
