use serde::{Deserialize, Serialize};

use crate::value::Value;

#[derive(Deserialize, Serialize)]
pub struct Request {
    requesting: Requesting,
    payload: Option<Value>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum Requesting {
    /// Treat the payload as a message
    SendMessage,
    /// Treat the payload as a new username
    ChangeUserName,
    /// Give the client a List of users connected to server, payload is None
    UserList,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::default()
    }
    pub fn requesting(&self) -> Requesting {
        self.requesting
    }
    pub fn payload(&mut self) -> Option<Value> {
        self.payload.take()
    }
}

#[derive(Default)]
pub struct RequestBuilder {
    requesting: Option<Requesting>,
    payload: Option<Value>,
}

impl RequestBuilder {
    pub fn requesting(mut self, requesting: Requesting) -> Self {
        self.requesting = Some(requesting);
        self
    }
    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = Some(payload);
        self
    }
    /// # Panics
    /// If you did not set a value for `requesting` this method will panic
    pub fn build(self) -> Request {
        Request {
            requesting: self.requesting.unwrap(),
            payload: self.payload,
        }
    }
}
