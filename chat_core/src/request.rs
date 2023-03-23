use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{message::MessageError, user::UsernameError, value::Value};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RequestError {
    #[error("could not get ip address's of streams")]
    Ip,
    /// Maybe make a wrapper around bincode::Error so the error
    /// type can be known.
    #[error("bad request")]
    Bad(String),
    #[error("bad username: {0}")]
    Username(UsernameError),
    #[error("bad message: {0}")]
    Message(MessageError),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Request {
    /// Treat the payload as a message
    SendMessage(Value),
    /// Treat the payload as a new username
    ChangeUserName(Value),
    /// Give the client a List of users connected to server
    UserList,
}
