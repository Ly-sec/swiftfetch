//! Centralized error handling for swiftfetch

use std::io;
use std::fmt;

/// Custom error type for swiftfetch operations
#[derive(Debug)]
pub enum SwiftfetchError {
    /// I/O errors (file reading, command execution)
    Io(io::Error),
    /// Parsing errors (invalid data format)
    #[allow(dead_code)]
    Parse(String),
    /// Configuration errors
    #[allow(dead_code)]
    Config(String),
    /// System detection errors
    Detection(String),
}

impl fmt::Display for SwiftfetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwiftfetchError::Io(err) => write!(f, "I/O error: {}", err),
            SwiftfetchError::Parse(msg) => write!(f, "Parse error: {}", msg),
            SwiftfetchError::Config(msg) => write!(f, "Config error: {}", msg),
            SwiftfetchError::Detection(msg) => write!(f, "Detection error: {}", msg),
        }
    }
}

impl std::error::Error for SwiftfetchError {}

impl From<io::Error> for SwiftfetchError {
    fn from(error: io::Error) -> Self {
        SwiftfetchError::Io(error)
    }
}

/// Type alias for Results in swiftfetch
pub type Result<T> = std::result::Result<T, SwiftfetchError>;
