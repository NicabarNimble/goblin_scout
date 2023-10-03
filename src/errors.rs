// errors.rs
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CustomError {
    IOError(io::Error),
    // ... Add other error types as needed.
}

impl From<io::Error> for CustomError {
    fn from(error: io::Error) -> Self {
        CustomError::IOError(error)
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::IOError(e) => write!(f, "IOError: {}", e),
        }
    }
}

// Implement `Display` and other traits as needed for better error handling.
