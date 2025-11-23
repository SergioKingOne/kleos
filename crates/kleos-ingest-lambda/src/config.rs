use std::env;
use thiserror::Error;

/// Configuration for the ingest Lambda function.
///
/// All configuration is loaded at initialization time from environment variables.
/// This ensures fail-fast behavior - if config is invalid, the Lambda won't start.
#[derive(Debug, Clone)]
pub struct Config {
    /// Name of the Kinesis stream to publish events to
    pub stream_name: String,
}

/// Errors that can occur during configuration loading
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid configuration value for {key}: {reason}")]
    InvalidValue { key: String, reason: String },
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    /// - `KINESIS_STREAM_NAME`: Name of the Kinesis stream (default: "kleos-stream")
    ///
    /// # Errors
    /// Returns `ConfigError` if required configuration is missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        let stream_name =
            env::var("KINESIS_STREAM_NAME").unwrap_or_else(|_| "kleos-stream".to_string());

        // Basic validation - stream name should not be empty
        if stream_name.is_empty() {
            return Err(ConfigError::InvalidValue {
                key: "KINESIS_STREAM_NAME".to_string(),
                reason: "stream name cannot be empty".to_string(),
            });
        }

        Ok(Self { stream_name })
    }
}
