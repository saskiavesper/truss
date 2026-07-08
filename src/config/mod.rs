mod environment;

pub use environment::{Environment, EnvironmentError};

use std::env;
use std::fs;
use std::path::Path;

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to parse environment: {0}")]
    Environment(#[from] EnvironmentError),
}

/// Holds all application configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// The current application environment.
    pub environment: Environment,
}

impl Config {
    /// Returns a Config with sensible default values.
    pub fn default_config() -> Self {
        Config {
            environment: Environment::Dev,
        }
    }
}

/// Loads configuration from environment variables and optional config file.
///
/// Configuration is loaded in the following order (later values override earlier):
/// 1. Default values
/// 2. Config file (if exists)
/// 3. Environment variables (TRUSS_ENVIRONMENT)
pub fn load() -> Result<Config, ConfigError> {
    let mut config = Config::default_config();

    // Try to load from config file
    let config_paths = ["truss.toml", "configs/default.toml", "configs/config.toml"];
    for path in &config_paths {
        if Path::new(path).exists() {
            let contents = fs::read_to_string(path)?;
            let file_config: Config = toml::from_str(&contents)?;
            config.environment = file_config.environment;
            break;
        }
    }

    // Override with environment variable if set
    if let Ok(env_str) = env::var("TRUSS_ENVIRONMENT") {
        config.environment = env_str.parse()?;
    }

    Ok(config)
}

/// Loads configuration and panics on error (useful for main).
pub fn must_load() -> Config {
    load().expect("failed to load configuration")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        assert_eq!(config.environment, Environment::Dev);
    }

    #[test]
    fn test_load_default() {
        // Clear any existing env var
        unsafe { env::remove_var("TRUSS_ENVIRONMENT") };
        let config = load().unwrap();
        assert_eq!(config.environment, Environment::Dev);
    }

    #[test]
    fn test_load_from_env() {
        unsafe { env::set_var("TRUSS_ENVIRONMENT", "prod") };
        let config = load().unwrap();
        assert_eq!(config.environment, Environment::Prod);
        unsafe { env::remove_var("TRUSS_ENVIRONMENT") };
    }

    #[test]
    fn test_load_invalid_env() {
        unsafe { env::set_var("TRUSS_ENVIRONMENT", "invalid") };
        let result = load();
        assert!(result.is_err());
        unsafe { env::remove_var("TRUSS_ENVIRONMENT") };
    }
}
