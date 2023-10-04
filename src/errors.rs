// errors.rs
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum CustomError {
    IOError(io::Error),
    StrError(String), // New variant for handling &str errors
}

impl From<io::Error> for CustomError {
    fn from(error: io::Error) -> Self {
        CustomError::IOError(error)
    }
}

impl From<&str> for CustomError {
    fn from(error: &str) -> Self {
        CustomError::StrError(error.to_string())
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::IOError(e) => write!(f, "IOError: {}", e),
            CustomError::StrError(s) => write!(f, "StrError: {}", s),
        }
    }
}

// You can continue implementing other traits and methods for better error handling as required.
