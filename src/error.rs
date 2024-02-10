use std::fmt::{self, Display};

use serde::{de, ser};

#[derive(Debug)]
pub struct ErrorWithOffset {
    offset: Option<usize>,
    kind: ErrorKind,
}

impl ErrorWithOffset {
    pub fn new(offset: usize, kind: ErrorKind) -> Self {
        ErrorWithOffset { offset: Some(offset), kind }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Message(String),
    MissingStringTerminator,
    StringLengthError(usize, usize),
    UnknownTypeId(u32),
    NumericOverflow,
    TypeMismatch,
    TrailingCharacters,
    LengthNotGiven,
    InvalidKeyType,
    TextEncodingError,
    InvalidHeader,
    UnsupportedValue,
}

impl ErrorKind {
    pub fn with<T>(self, offset: usize) -> Result<T, ErrorWithOffset> {
        Err(ErrorWithOffset { offset: Some(offset), kind: self })
    }
}

impl ser::Error for ErrorWithOffset {
    fn custom<T: Display>(msg: T) -> Self {
        ErrorWithOffset { kind: ErrorKind::Message(msg.to_string()), offset: None }
    }
}

impl de::Error for ErrorWithOffset {
    fn custom<T: Display>(msg: T) -> Self {
        ErrorWithOffset { kind: ErrorKind::Message(msg.to_string()), offset: None }
    }
}

impl ser::Error for ErrorKind {
    fn custom<T: Display>(msg: T) -> Self {
        ErrorKind::Message(msg.to_string())
    }
}

impl de::Error for ErrorKind {
    fn custom<T: Display>(msg: T) -> Self {
        ErrorKind::Message(msg.to_string())
    }
}

impl Display for ErrorWithOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "At offset {offset}: ")?;
        }
        write!(f, "{}", self.kind)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ErrorKind::Message(msg) => f.write_str(&msg),
            ErrorKind::StringLengthError(s_len, doc_len) => write!(f, "String length {s_len} too long, only {doc_len} bytes left in document"),
            ErrorKind::InvalidHeader => write!(f, "The file header is invalid"),
            ErrorKind::UnsupportedValue => write!(f, "Unsupported value in input"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for ErrorKind {}
impl std::error::Error for ErrorWithOffset {}