//! GPU-independent server browser. Opening this library makes no network calls.
pub mod address;
pub mod directory;
pub mod model;
pub mod persistence;
pub mod query;
pub mod service;

use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
    Duplicate,
    NewerSchema(u32),
    RateLimited(u64),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Invalid(e) => write!(f, "{e}"),
            Self::Duplicate => f.write_str("duplicate server address"),
            Self::NewerSchema(v) => write!(f, "unsupported server browser schema {v}"),
            Self::RateLimited(seconds) => write!(f, "directory retry after {seconds} seconds"),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_secs())
}
