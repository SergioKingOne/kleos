use aws_lambda_events::event::kinesis::KinesisEventRecord;
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use thiserror::Error;

// --- Modules ---

pub mod config;

// --- Errors ---

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Missing record metadata: {0}")]
    MissingMetadata(String),
}

// --- Types ---

/// Metadata extracted from a Kinesis record for observability and debugging.
#[derive(Debug, Clone)]
pub struct RecordMetadata {
    /// Kinesis sequence number for ordering and checkpointing
    pub sequence_number: String,
    /// Partition key used to distribute records across shards
    pub partition_key: String,
    /// Approximate timestamp when the record arrived in Kinesis (milliseconds since epoch)
    pub approximate_arrival_timestamp: Option<i64>,
}

/// A validated and parsed Kinesis record with its data and metadata.
///
/// This wrapper provides clean separation between the deserialized domain data
/// and the Kinesis-specific metadata, making handlers more testable and focused.
#[derive(Debug, Clone)]
pub struct ProcessableRecord<T> {
    /// The deserialized domain data from the Kinesis record
    pub data: T,
    /// Kinesis metadata for logging, monitoring, and debugging
    pub metadata: RecordMetadata,
}

impl<T: DeserializeOwned> TryFrom<KinesisEventRecord> for ProcessableRecord<T> {
    type Error = ProcessingError;

    fn try_from(record: KinesisEventRecord) -> Result<Self, Self::Error> {
        // Deserialize data (Base64Data automatically derefs to &[u8])
        let data = serde_json::from_slice(&record.kinesis.data)?;

        // Extract metadata with sensible defaults
        let metadata = RecordMetadata {
            sequence_number: record
                .kinesis
                .sequence_number
                .unwrap_or_else(|| "unknown".to_string()),
            partition_key: record
                .kinesis
                .partition_key
                .unwrap_or_else(|| "unknown".to_string()),
            approximate_arrival_timestamp: Some(record.kinesis.approximate_arrival_timestamp.0.timestamp_millis()),
        };

        Ok(ProcessableRecord { data, metadata })
    }
}
