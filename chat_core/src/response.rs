use crate::{message::Message, request::RequestError};

pub type Response = Result<Message, RequestError>;
