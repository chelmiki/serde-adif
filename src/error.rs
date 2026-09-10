use serde::{de, ser};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    /// A tag was opened with "<" but no matching ">" was found. Carries a
    /// short snippet of what follows the "<", truncated since the rest of
    /// the input could be arbitrarily large.
    ExpectedClosingTag(String),
    /// A `<FIELD:LENGTH:TYPE>` tag is missing one of its colon-separated
    /// parts (e.g. no length section at all). Carries the raw tag content
    /// that failed to split correctly.
    MalformedTag(String),
    /// A tag's length section is present but isn't a valid unsigned
    /// integer. Carries the raw text that failed to parse.
    InvalidLength(String),
    Unsupported(&'static str),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Eof => f.write_str("unexpected end of input"),
            Error::ExpectedClosingTag(snippet) => {
                write!(f, "missing closing tag \">\" after \"{}\"", snippet)
            }
            Error::MalformedTag(tag) => {
                write!(f, "the tag \"{}\" is missing its key or value length", tag)
            }
            Error::InvalidLength(len) => write!(f, "\"{}\" is not a valid value length", len),
            Error::Unsupported(unsupported) => write!(f, "Unsupported type {}", unsupported),
        }
    }
}

impl std::error::Error for Error {}
