use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{user::User, value::Value};

#[derive(Serialize, Deserialize)]
pub struct MessageGuidelines {
    message_size: usize,
    just_whitespace: bool,
    trailing_whitespace: bool,
    leading_whitespace: bool,
    empty: bool,
}

impl Default for MessageGuidelines {
    fn default() -> Self {
        Self {
            message_size: 4000,
            just_whitespace: false,
            trailing_whitespace: false,
            leading_whitespace: false,
            empty: false,
        }
    }
}

impl MessageGuidelines {
    pub fn message_size(&self) -> usize {
        self.message_size
    }
    pub fn just_whitespace(&self) -> bool {
        self.just_whitespace
    }
    pub fn trailing_whitespace(&self) -> bool {
        self.trailing_whitespace
    }
    pub fn leading_whitespace(&self) -> bool {
        self.leading_whitespace
    }
    pub fn empty(&self) -> bool {
        self.empty
    }
}

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
    pub fn builder<'a>() -> MessageBuilder<'a> {
        MessageBuilder::default()
    }
}

#[derive(Default)]
pub struct MessageBuilder<'a> {
    guidelines: Option<&'a MessageGuidelines>,
    from: Option<User>,
    // to: Option<Vec<User>>,
    payload: Option<Value>,
}

impl<'a> MessageBuilder<'a> {
    pub fn with_guidelines(mut self, guidelines: &'a MessageGuidelines) -> Self {
        self.guidelines = Some(guidelines);
        self
    }
    pub fn from_who(mut self, from: User) -> Self {
        self.from = Some(from);
        self
    }
    // pub fn to(mut self, to: Vec<User>) -> Self {
    //     self.to = Some(to);
    //     self
    // }
    pub fn payload(mut self, payload: Value) -> Self {
        if let Some(guidelines) = self.guidelines {
            // Guidelines that only apply to a string message
            if let Value::String(text_message) = &payload {
                // A message cannot be empty but the message is empty
                if !guidelines.empty() && text_message.is_empty() {
                    log::debug!("message cannot be empty but is empty");
                    // Return err
                } 
                // A message cannot be just whitespace
                if !guidelines.just_whitespace() && text_message.chars().all(|c| c.is_whitespace()) {
                    log::debug!("message cannot be just whitespace but is just whitespace");

                }
                // A message cannot have trailing whitespace but the last character is whitespace
                if !guidelines.trailing_whitespace() && text_message.chars().rev().next().unwrap().is_whitespace() {
                    log::debug!("message cannot have trailing whitespace but has trailing whitespace");

                }
                // A message cannot of leading whitespace but the first character is leading whitespace
                if !guidelines.leading_whitespace() && text_message.chars().next().unwrap().is_whitespace() {
                    log::debug!("message cannot have leading whitespace but has leading whitespace");

                }
            }
        }

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
