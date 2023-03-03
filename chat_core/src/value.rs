use std::{fmt::{self, Display}, error::Error};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ValueError {
    CannotConvertValue(&'static str),
}

impl Error for ValueError {}

impl Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueError::CannotConvertValue(s) => write!(f, "cannot convert value to {s}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Image(Vec<u8>),
    File(Vec<u8>),
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

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(String::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl TryFrom<Value> for String {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s),
            Value::Integer(_) => Err(ValueError::CannotConvertValue("int")),
            Value::Float(_) => Err(ValueError::CannotConvertValue("float")),
            Value::Boolean(_) => Err(ValueError::CannotConvertValue("bool")),
            Value::Image(_) => Err(ValueError::CannotConvertValue("image")),
            Value::File(_) => Err(ValueError::CannotConvertValue("file")),
        }
    }
}