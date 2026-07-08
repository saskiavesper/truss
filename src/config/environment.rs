use std::fmt;
use std::str::FromStr;

/// Represents the application environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Environment {
    #[serde(rename = "test")]
    Test,
    #[serde(rename = "dev")]
    Dev,
    #[serde(rename = "prod")]
    Prod,
    #[serde(rename = "local")]
    Local,
}

impl Environment {
    /// Returns true if the environment is production.
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Prod)
    }

    /// Returns true if the environment is development.
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Dev)
    }

    /// Returns true if the environment is test.
    pub fn is_test(&self) -> bool {
        matches!(self, Environment::Test)
    }

    /// Returns true if the environment is local.
    pub fn is_local(&self) -> bool {
        matches!(self, Environment::Local)
    }
}

impl FromStr for Environment {
    type Err = EnvironmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "test" => Ok(Environment::Test),
            "dev" => Ok(Environment::Dev),
            "prod" => Ok(Environment::Prod),
            "local" => Ok(Environment::Local),
            _ => Err(EnvironmentError::InvalidEnvironment(s.to_string())),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Test => write!(f, "test"),
            Environment::Dev => write!(f, "dev"),
            Environment::Prod => write!(f, "prod"),
            Environment::Local => write!(f, "local"),
        }
    }
}

/// Errors that can occur when parsing an environment.
#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    #[error("invalid environment: {0} (must be one of: test, dev, local, prod)")]
    InvalidEnvironment(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_environment() {
        assert_eq!("test".parse::<Environment>().unwrap(), Environment::Test);
        assert_eq!("dev".parse::<Environment>().unwrap(), Environment::Dev);
        assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Prod);
        assert_eq!("local".parse::<Environment>().unwrap(), Environment::Local);
    }

    #[test]
    fn test_parse_environment_case_insensitive() {
        assert_eq!("TEST".parse::<Environment>().unwrap(), Environment::Test);
        assert_eq!("Dev".parse::<Environment>().unwrap(), Environment::Dev);
        assert_eq!("PROD".parse::<Environment>().unwrap(), Environment::Prod);
    }

    #[test]
    fn test_parse_environment_invalid() {
        assert!("invalid".parse::<Environment>().is_err());
    }

    #[test]
    fn test_environment_checks() {
        assert!(Environment::Prod.is_production());
        assert!(Environment::Dev.is_development());
        assert!(Environment::Test.is_test());
        assert!(Environment::Local.is_local());

        assert!(!Environment::Dev.is_production());
        assert!(!Environment::Prod.is_development());
    }

    #[test]
    fn test_display() {
        assert_eq!(Environment::Test.to_string(), "test");
        assert_eq!(Environment::Dev.to_string(), "dev");
        assert_eq!(Environment::Prod.to_string(), "prod");
        assert_eq!(Environment::Local.to_string(), "local");
    }
}
