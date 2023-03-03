use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{user::User, value::Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    from: User,
    // to: Vec<User>,
    payload: Value,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.from, self.payload)
    }
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }
}

#[derive(Default)]
pub struct MessageBuilder {
    from: Option<User>,
    // to: Option<Vec<User>>,
    payload: Option<Value>,
}

impl MessageBuilder {
    pub fn from(mut self, from: User) -> Self {
        self.from = Some(from);
        self
    }
    // pub fn to(mut self, to: Vec<User>) -> Self {
    //     self.to = Some(to);
    //     self
    // }
    pub fn payload(mut self, payload: Value) -> Self {
        self.payload = Some(payload);
        self
    }
    /// Will panic if you did not set all values
    pub fn build(self) -> Message {
        Message {
            from: self.from.unwrap(),
            /* to: self.to.unwrap(), */ payload: self.payload.unwrap(),
        }
    }
}
