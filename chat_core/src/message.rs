use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Image(Vec<u8>),
    File(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub username: String,
    pub payload: Value,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.username, self.payload)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(str) => write!(f, "{str}"),
            Value::Integer(int) => write!(f, "{int}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Boolean(bool) => write!(f, "{bool}"),
            Value::Image(_) => todo!(),
            Value::File(_) => todo!(),
        }
    }
}