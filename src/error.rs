use std::error::Error;
use std::fmt::{self, Display};
use std::io;

/// Library-wide error type capturing domain-neutral and underlying I/O failures.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    /// Configuration or environment issue that prevents command execution.
    ConfigError(String),
    /// Raised when a requested resource cannot be located.
    NotFound(String),
    /// Clipboard interaction failure surfaced to the user.
    ClipboardError(String),
    /// Invalid key provided for touch command
    InvalidKey(String),
    /// Path traversal attempt detected (security violation)
    PathTraversal(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "{}", err),
            AppError::ConfigError(message) => write!(f, "{message}"),
            AppError::NotFound(message) => write!(f, "{message}"),
            AppError::ClipboardError(message) => write!(f, "{message}"),
            AppError::InvalidKey(key) => write!(f, "Invalid key: {key}"),
            AppError::PathTraversal(message) => write!(f, "{message}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::ConfigError(_)
            | AppError::NotFound(_)
            | AppError::ClipboardError(_)
            | AppError::InvalidKey(_)
            | AppError::PathTraversal(_) => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value)
    }
}

impl AppError {
    pub(crate) fn config_error<S: Into<String>>(message: S) -> Self {
        AppError::ConfigError(message.into())
    }

    /// Provide an `io::ErrorKind`-like view for callers expecting legacy behavior.
    pub fn kind(&self) -> io::ErrorKind {
        match self {
            AppError::Io(err) => err.kind(),
            AppError::ConfigError(_) => io::ErrorKind::InvalidInput,
            AppError::NotFound(_) => io::ErrorKind::NotFound,
            AppError::ClipboardError(_) => io::ErrorKind::Other,
            AppError::InvalidKey(_) => io::ErrorKind::InvalidInput,
            AppError::PathTraversal(_) => io::ErrorKind::InvalidInput,
        }
    }

    pub(crate) fn not_found<S: Into<String>>(message: S) -> Self {
        AppError::NotFound(message.into())
    }

    pub(crate) fn clipboard_error<S: Into<String>>(message: S) -> Self {
        AppError::ClipboardError(message.into())
    }

    pub fn invalid_key<S: Into<String>>(message: S) -> Self {
        AppError::InvalidKey(message.into())
    }

    pub fn path_traversal<S: Into<String>>(message: S) -> Self {
        AppError::PathTraversal(message.into())
    }
}
