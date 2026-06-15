use std::sync::OnceLock;
use regex::Regex;

/// A domain type indicating a `String` that is not empty or blank.
///
/// # Examples
///
/// ```
/// # use common::domain::NonEmptyString;
/// let valid = NonEmptyString::try_from("  Hello Rust!  ".to_string());
/// assert!(valid.is_ok());
///
/// let invalid = NonEmptyString::try_from("   ".to_string());
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyString(String);

impl TryFrom<String> for NonEmptyString {
    type Error = &'static str;
    /// Parses a [`String`] into a possible [`NonEmptyString`].
    ///
    /// Strings that are empty or blank (only whitespace) are invalid and return [`Self::Error`].
    /// Otherwise, returns [`Ok`] containing the trimmed [`NonEmptyString`].
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err("String cannot be empty or whitespace")
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}
/// Allows `&NonEmptyString` to behave like a `&str`.
impl std::ops::Deref for NonEmptyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);


fn is_email_valid(email: &str) -> bool {
    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap()
    });
    regex.is_match(email)
}

impl TryFrom<String> for Email {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if is_email_valid(trimmed) {
            return Err("String is an invalid email");
        }
        Ok(Email(trimmed.to_string()))
    }
}

impl std::ops::Deref  for Email {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A domain type for a title that is non-empty and at most 120 characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(String);

impl TryFrom<String> for Title {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err("Title cannot be empty or whitespace")
        } else if trimmed.len() > 120 {
            Err("Title cannot exceed 120 characters")
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}

impl std::ops::Deref for Title {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A domain type for a description that can be empty or at most 500 characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(String);

impl TryFrom<String> for Description {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.len() > 500 {
            Err("Description cannot exceed 500 characters")
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}

impl std::ops::Deref for Description {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A domain type for a positive integer (greater than zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonNegInteger(u64);

impl TryFrom<u64> for NonNegInteger {
    type Error = &'static str;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            Err("Value must be greater than zero")
        } else {
            Ok(Self(value))
        }
    }
}

impl std::ops::Deref for NonNegInteger {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A domain type for a username between 2 and 50 characters, optionally starting with `@`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

impl TryFrom<String> for Username {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err("Username cannot be empty");
        }
        let name_part = trimmed.strip_prefix('@').unwrap_or(trimmed);
        if name_part.len() < 2 {
            Err("Username must be at least 2 characters")
        } else if name_part.len() > 50 {
            Err("Username cannot exceed 50 characters")
        } else {
            Ok(Self(name_part.to_string()))
        }
    }
}

impl std::ops::Deref for Username {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn parses_a_string_with_content() {
        let result = NonEmptyString::try_from("Something".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().deref(), "Something");
    }

    #[test]
    fn deref_is_implemented() {
        let result = NonEmptyString::try_from("Something".to_string());
        assert_eq!(result.unwrap().len(), 9);
    }

    #[test]
    fn parse_removes_whitespace() {
        let result = NonEmptyString::try_from("mobile input  ".to_string());
        let no_whitespace = NonEmptyString::try_from("mobile input".to_string());
        assert_eq!(result, no_whitespace);
    }

    #[test]
    fn whitespace_only_is_invalid() {
        let result = NonEmptyString::try_from("  ".to_string());
        assert!(result.is_err_and(|err| err == "String cannot be empty or whitespace"));
    }

    #[test]
    fn empty_is_invalid() {
        let result = NonEmptyString::try_from("".to_string());
        assert!(result.is_err_and(|err| err == "String cannot be empty or whitespace"));
    }

    #[test]
    fn title_accepts_valid_string() {
        let result = Title::try_from("Hello".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().deref(), "Hello");
    }

    #[test]
    fn title_rejects_empty() {
        let result = Title::try_from("".to_string());
        assert!(result.is_err_and(|err| err == "Title cannot be empty or whitespace"));
    }

    #[test]
    fn title_rejects_too_long() {
        let long = "a".repeat(121);
        let result = Title::try_from(long);
        assert!(result.is_err_and(|err| err == "Title cannot exceed 120 characters"));
    }

    #[test]
    fn title_trims_whitespace() {
        let result = Title::try_from("  spaced  ".to_string());
        assert_eq!(result.unwrap().deref(), "spaced");
    }

    #[test]
    fn description_accepts_empty() {
        let result = Description::try_from("".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn description_accepts_whitespace() {
        let result = Description::try_from("   ".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn description_rejects_too_long() {
        let long = "a".repeat(501);
        let result = Description::try_from(long);
        assert!(result.is_err_and(|err| err == "Description cannot exceed 500 characters"));
    }

    #[test]
    fn description_accepts_max_length() {
        let s = "a".repeat(500);
        let result = Description::try_from(s.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().deref(), s);
    }

    #[test]
    fn non_neg_integer_accepts_positive() {
        let result = NonNegInteger::try_from(1u64);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), 1);
    }

    #[test]
    fn non_neg_integer_rejects_zero() {
        let result = NonNegInteger::try_from(0u64);
        assert!(result.is_err_and(|err| err == "Value must be greater than zero"));
    }

    #[test]
    fn username_accepts_basic() {
        let result = Username::try_from("john".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().deref(), "john");
    }

    #[test]
    fn username_accepts_with_at() {
        let result = Username::try_from("@john".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().deref(), "john");
    }

    #[test]
    fn username_rejects_empty() {
        let result = Username::try_from("".to_string());
        assert!(result.is_err_and(|err| err == "Username cannot be empty"));
    }

    #[test]
    fn username_rejects_too_short() {
        let result = Username::try_from("a".to_string());
        assert!(result.is_err_and(|err| err == "Username must be at least 2 characters"));
    }

    #[test]
    fn username_rejects_too_long() {
        let long = "a".repeat(51);
        let result = Username::try_from(long);
        assert!(result.is_err_and(|err| err == "Username cannot exceed 50 characters"));
    }

    #[test]
    fn username_strips_at_for_length_check() {
        let result = Username::try_from("@a".to_string());
        assert!(result.is_err_and(|err| err == "Username must be at least 2 characters"));
    }
}
