use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Blob(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub payload: Value,
}
