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

    fn whitespace_only_is_invalid() {
        let result = NonEmptyString::try_from("  ".to_string());
        assert!(result.is_err_and(|err| err == "String cannot be empty or whitespace"));
    }

    fn empty_is_invalid() {
        let result = NonEmptyString::try_from("".to_string());
        assert!(result.is_err_and(|err| err == "String cannot be empty or whitespace"));
    }
}
