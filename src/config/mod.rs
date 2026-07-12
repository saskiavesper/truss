mod environment;

pub use environment::{Environment, EnvironmentError};

use config::{Config as ConfigRs, ConfigError, Environment as ConfigEnvironment};

/// Holds all application configuration.
///
/// Environment variables use underscore as separator:
/// - TRUSS_ENVIRONMENT
/// - TRUSS_DATABASE_HOST
/// - TRUSS_DATABASE_PORT
/// - TRUSS_NATS_PORT
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// The current application environment.
    pub environment: Environment,

    /// Database configuration.
    pub database: DatabaseConfig,

    /// NATS configuration.
    pub nats: NatsConfig,
}

/// Database configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
}

/// NATS configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NatsConfig {
    pub port: u16,
    pub mgmt_port: u16,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// Configuration is loaded using config-rs which reads from:
    /// 1. Default values
    /// 2. Environment variables (with prefix "TRUSS" and "_" separator)
    ///
    /// Example env vars:
    /// - TRUSS_ENVIRONMENT=dev
    /// - TRUSS_DATABASE_HOST=localhost
    /// - TRUSS_NATS_PORT=4222
    pub fn load() -> Result<Self, ConfigError> {
        let defaults = serde_json::json!({
            "environment": "dev",
            "database": {
                "host": "localhost",
                "port": 5432,
                "user": "postgres",
                "password": "postgres",
                "database": "truss_dev",
                "pool_size": 10
            },
            "nats": {
                "port": 4222,
                "mgmt_port": 8222
            }
        });

        let config = ConfigRs::builder()
            // Add default values
            .add_source(ConfigRs::try_from(defaults.as_object().unwrap()).unwrap())
            // Add environment variables with TRUSS prefix
            // e.g., TRUSS_ENVIRONMENT or TRUSS_DATABASE_HOST
            .add_source(ConfigEnvironment::with_prefix("TRUSS").separator("_"))
            .build()?;

        // Deserialize into our Config struct
        // We need special handling for the environment field since it's an enum
        let environment_str: String = config.get_string("environment")?;
        let environment: Environment = environment_str.parse().map_err(|e: EnvironmentError| {
            ConfigError::Message(e.to_string())
        })?;

        Ok(Config {
            environment,
            database: DatabaseConfig {
                host: config.get_string("database.host")?,
                port: config.get::<u16>("database.port")?,
                user: config.get_string("database.user")?,
                password: config.get_string("database.password")?,
                database: config.get_string("database.database")?,
                pool_size: config.get::<u32>("database.pool_size")?,
            },
            nats: NatsConfig {
                port: config.get::<u16>("nats.port")?,
                mgmt_port: config.get::<u16>("nats.mgmt_port")?,
            },
        })
    }
}

/// Loads configuration and panics on error (useful for main).
pub fn must_load() -> Config {
    Config::load().expect("failed to load configuration")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Serialize tests that modify env vars
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_load_default() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // Clear any existing env vars
        unsafe {
            env::remove_var("TRUSS_ENVIRONMENT");
            env::remove_var("TRUSS_DATABASE_HOST");
        }
        let config = Config::load().unwrap();
        assert_eq!(config.environment, Environment::Dev);
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
    }

    #[test]
    fn test_load_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        // Clear first to avoid interference
        unsafe {
            env::remove_var("TRUSS_ENVIRONMENT");
            env::remove_var("TRUSS_DATABASE_HOST");
        }
        unsafe {
            env::set_var("TRUSS_ENVIRONMENT", "prod");
            env::set_var("TRUSS_DATABASE_HOST", "db.example.com");
        }
        let config = Config::load().unwrap();
        assert_eq!(config.environment, Environment::Prod);
        assert_eq!(config.database.host, "db.example.com");
        unsafe {
            env::remove_var("TRUSS_ENVIRONMENT");
            env::remove_var("TRUSS_DATABASE_HOST");
        }
    }

    #[test]
    fn test_load_invalid_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var("TRUSS_ENVIRONMENT", "invalid");
        }
        let result = Config::load();
        assert!(result.is_err());
        unsafe {
            env::remove_var("TRUSS_ENVIRONMENT");
        }
    }
}
