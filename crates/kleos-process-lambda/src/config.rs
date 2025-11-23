use thiserror::Error;

/// Configuration for the process Lambda function.
///
/// All configuration is loaded at initialization time from environment variables.
/// Currently minimal, but structured to allow easy extension as processing
/// requirements grow (e.g., batch sizes, retry policies, output destinations).
#[derive(Debug, Clone)]
pub struct Config {}

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
    /// (None currently required - this is a placeholder for future config)
    ///
    /// # Errors
    /// Returns `ConfigError` if required configuration is missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        // No configuration required yet, but the pattern is established
        // Future config could include:
        // - Batch processing size
        // - Error retry policies
        // - Output destination (DynamoDB table, S3 bucket, etc.)
        // - Processing mode flags

        Ok(Self {})
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
