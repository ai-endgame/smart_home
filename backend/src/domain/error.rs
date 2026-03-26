use std::fmt;

/// Structured domain error type replacing untyped `String` errors.
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    NotFound(String),
    AlreadyExists(String),
    InvalidOperation(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "{}", msg),
            DomainError::AlreadyExists(msg) => write!(f, "{}", msg),
            DomainError::InvalidOperation(msg) => write!(f, "{}", msg),
        }
    }
}
