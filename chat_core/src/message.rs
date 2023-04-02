use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{guidelines::AgainstGuidelines, user::User, value::Value};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum MessageError {
    #[error("message is empty")]
    Empty,
    #[error("message is just whitespace")]
    JustWhitespace,
    #[error("message has leading whitespace")]
    LeadingWhitespace,
    #[error("message had trailing whitespace")]
    TrailingWhitespace,
    #[error("messages can only be text")]
    TextOnly,
}

#[derive(Serialize, Deserialize)]
pub struct MessageGuidelines {
    message_size: usize,
    just_whitespace: bool,
    trailing_whitespace: bool,
    leading_whitespace: bool,
    empty: bool,
    text_only: bool,
}

impl Default for MessageGuidelines {
    fn default() -> Self {
        Self {
            message_size: 4000,
            just_whitespace: false,
            trailing_whitespace: false,
            leading_whitespace: false,
            empty: false,
            text_only: true,
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
    pub fn text_only(&self) -> bool {
        self.text_only
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
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }
}

impl AgainstGuidelines<MessageGuidelines> for Message {
    type Error = MessageError;

    fn against_guidelines(self, guidelines: &MessageGuidelines) -> Result<Self, Self::Error> {
        if let Value::String(text_message) = &self.payload {
            // A message cannot be empty but the message is empty
            if !guidelines.empty() && text_message.is_empty() {
                log::debug!("message cannot be empty but is empty");
                return Err(MessageError::Empty);
            }
            // A message cannot be just whitespace
            else if !guidelines.just_whitespace()
                && text_message.chars().all(|c| c.is_whitespace())
            {
                log::debug!("message cannot be just whitespace but is just whitespace");
                return Err(MessageError::JustWhitespace);
            }
            // A message cannot have trailing whitespace but the last character is whitespace
            else if !guidelines.trailing_whitespace()
                && text_message.chars().rev().next().unwrap().is_whitespace()
            {
                log::debug!("message cannot have trailing whitespace but has trailing whitespace");
                return Err(MessageError::TrailingWhitespace);
            }
            // A message cannot of leading whitespace but the first character is leading whitespace
            else if !guidelines.leading_whitespace()
                && text_message.chars().next().unwrap().is_whitespace()
            {
                log::debug!("message cannot have leading whitespace but has leading whitespace");
                return Err(MessageError::LeadingWhitespace);
            }
        } else {
            if guidelines.text_only() {
                return Err(MessageError::TextOnly);
            }
        }

        Ok(self)
    }
}

#[derive(Default)]
pub struct MessageBuilder {
    from: Option<User>,
    // to: Option<Vec<User>>,
    payload: Option<Value>,
}

impl MessageBuilder {
    pub fn from_who(mut self, from: User) -> Self {
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
