#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

/// Represents a validation error for a specific field.
#[derive(Debug, Clone)]
pub struct FieldError {
    pub message: String,
    pub translation: String,
}

/// Holds all validation errors organized by field name.
#[derive(Debug, Clone, Default)]
pub struct Error {
    field_errors: HashMap<String, Vec<FieldError>>,
}

impl Error {
    /// Creates a new Error with an empty field errors map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a validation error for the specified field.
    pub fn add(
        &mut self,
        field: impl Into<String>,
        message: impl Into<String>,
        translation: impl Into<String>,
    ) {
        self.field_errors
            .entry(field.into())
            .or_default()
            .push(FieldError {
                message: message.into(),
                translation: translation.into(),
            });
    }

    /// Returns true if there are any validation errors.
    pub fn has_errors(&self) -> bool {
        !self.field_errors.is_empty()
    }

    /// Returns all validation errors for the specified field.
    pub fn errors_for(&self, field: &str) -> Option<&[FieldError]> {
        self.field_errors.get(field).map(|v| v.as_slice())
    }

    /// Returns the number of fields with errors.
    pub fn field_count(&self) -> usize {
        self.field_errors.len()
    }

    /// Returns the total number of errors across all fields.
    pub fn error_count(&self) -> usize {
        self.field_errors.values().map(|v| v.len()).sum()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.has_errors() {
            return Ok(());
        }

        let messages: Vec<String> = self
            .field_errors
            .values()
            .flat_map(|errors| errors.iter().map(|e| e.message.clone()))
            .collect();

        write!(f, "validation failed: {}", messages.join("; "))
    }
}

impl std::error::Error for Error {}

/// A trait for types that can validate themselves.
pub trait Validate {
    fn validate(&self) -> Error;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_error() {
        let error = Error::new();
        assert!(!error.has_errors());
        assert_eq!(error.field_count(), 0);
        assert_eq!(error.error_count(), 0);
    }

    #[test]
    fn test_add_error() {
        let mut error = Error::new();
        error.add("email", "email is required", "Email is required");

        assert!(error.has_errors());
        assert_eq!(error.field_count(), 1);
        assert_eq!(error.error_count(), 1);
    }

    #[test]
    fn test_add_multiple_errors_same_field() {
        let mut error = Error::new();
        error.add("email", "email is required", "Email is required");
        error.add("email", "email is invalid", "Email is invalid");

        assert_eq!(error.field_count(), 1);
        assert_eq!(error.error_count(), 2);
    }

    #[test]
    fn test_add_multiple_errors_different_fields() {
        let mut error = Error::new();
        error.add("email", "email is required", "Email is required");
        error.add("name", "name is required", "Name is required");

        assert_eq!(error.field_count(), 2);
        assert_eq!(error.error_count(), 2);
    }

    #[test]
    fn test_errors_for() {
        let mut error = Error::new();
        error.add("email", "email is required", "Email is required");

        let email_errors = error.errors_for("email");
        assert!(email_errors.is_some());
        assert_eq!(email_errors.unwrap().len(), 1);

        let name_errors = error.errors_for("name");
        assert!(name_errors.is_none());
    }

    #[test]
    fn test_display() {
        let mut error = Error::new();
        error.add("email", "email is required", "Email is required");

        let display = format!("{}", error);
        assert!(display.contains("validation failed"));
        assert!(display.contains("email is required"));
    }

    #[test]
    fn test_display_no_errors() {
        let error = Error::new();
        let display = format!("{}", error);
        assert!(display.is_empty());
    }
}
