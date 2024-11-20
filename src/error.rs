use std::io::{Error as IoError, ErrorKind};

pub type Error = IoError;

pub fn other(msg: &str) -> Error {
    IoError::new(ErrorKind::Other, msg)
}

pub fn invalid_input(msg: &str) -> Error {
    IoError::new(ErrorKind::InvalidInput, msg)
}

pub fn invalid_data(msg: &str) -> Error {
    IoError::new(ErrorKind::InvalidInput, msg)
}

pub fn unsupported(msg: &str) -> Error {
    IoError::new(ErrorKind::Unsupported, msg)
}

pub fn parse_error() -> Error {
    invalid_data("parse error")
}

pub fn missing_key() -> Error {
    invalid_data("missing key")
}

pub fn decode_error() -> Error {
    invalid_data("decode error")
}

pub fn encode_error() -> Error {
    invalid_data("encode error")
}

pub fn verification_failed() -> Error {
    invalid_data("verification failed")
}

pub fn message_malformed() -> Error {
    invalid_data("message malformed")
}
