//! Kinesis Stream Consumer Lambda
//!
//! Processes events from Kinesis Data Stream with:
//! - Batch processing for efficiency
//! - Partial batch response for error handling
//! - Type-safe event deserialization
//! - Structured logging
//!
//! Architecture:
//! - Receives batches from Kinesis (auto-polling by Lambda runtime)
//! - Deserializes to strongly-typed domain events
//! - Processes each event based on type (exhaustive pattern matching)
//! - Returns partial failures for retry

use aws_lambda_events::event::kinesis::KinesisEvent;
use kleos_types::{EventPayload, StreamEvent};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

/// Partial batch item failure for Kinesis error handling
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchItemFailure {
    item_identifier: String,
}

/// Response for partial batch failures
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamsEventResponse {
    batch_item_failures: Vec<BatchItemFailure>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize structured logging (CloudWatch adds timestamps)
    tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .without_time()
        .init();

    info!("Initializing Stream Consumer Lambda");

    // Run the Lambda runtime
    lambda_runtime::run(service_fn(handler)).await
}

/// Handler for Kinesis stream events
async fn handler(event: LambdaEvent<KinesisEvent>) -> Result<StreamsEventResponse, Error> {
    let (kinesis_event, _context) = event.into_parts();

    let record_count = kinesis_event.records.len();
    info!(record_count = record_count, "Processing Kinesis batch");

    let mut failed_records = Vec::new();

    // Process each record in the batch
    for record in kinesis_event.records {
        let sequence_number = record
            .kinesis
            .sequence_number
            .unwrap_or_else(|| "unknown".to_string());

        let partition_key = record.kinesis.partition_key.as_deref().unwrap_or("unknown");

        info!(
            sequence_number = %sequence_number,
            partition_key = %partition_key,
            "Processing record"
        );

        // Process the record
        match process_record(&record.kinesis.data).await {
            Ok(_) => {
                info!(sequence_number = %sequence_number, "Record processed successfully");
            }
            Err(e) => {
                error!(
                    sequence_number = %sequence_number,
                    error = %e,
                    "Failed to process record"
                );

                // Add to failed records for retry
                failed_records.push(BatchItemFailure {
                    item_identifier: sequence_number,
                });
            }
        }
    }

    let success_count = record_count - failed_records.len();
    let failure_count = failed_records.len();

    info!(
        success = success_count,
        failures = failure_count,
        "Batch processing complete"
    );

    // Return partial batch response
    Ok(StreamsEventResponse {
        batch_item_failures: failed_records,
    })
}

/// Process a single Kinesis record
async fn process_record(data: &[u8]) -> Result<(), anyhow::Error> {
    // Deserialize to domain event
    let stream_event: StreamEvent = serde_json::from_slice(data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize event: {}", e))?;

    info!(
        request_id = %stream_event.request_id,
        timestamp = %stream_event.timestamp,
        "Deserialized stream event"
    );

    // Process based on event type (exhaustive pattern matching)
    match &stream_event.payload {
        EventPayload::UserAction(user_action) => {
            process_user_action(&stream_event.request_id.to_string(), user_action).await?;
        }
        EventPayload::SystemEvent(system_event) => {
            process_system_event(&stream_event.request_id.to_string(), system_event).await?;
        }
        EventPayload::DataProcessed(data_processed) => {
            process_data_processed(&stream_event.request_id.to_string(), data_processed).await?;
        }
    }

    Ok(())
}

/// Process user action event
async fn process_user_action(
    request_id: &str,
    event: &kleos_types::UserActionEvent,
) -> Result<(), anyhow::Error> {
    info!(
        request_id = request_id,
        user_id = %event.user_id,
        action = ?event.action_type,
        "Processing user action"
    );

    // Skeleton processing logic based on action type
    match event.action_type {
        kleos_types::ActionType::Create => {
            info!(request_id = request_id, "Handling CREATE action");
            // TODO: Implement create logic
            // e.g., save to database, trigger downstream events, etc.
        }
        kleos_types::ActionType::Update => {
            info!(request_id = request_id, "Handling UPDATE action");
            // TODO: Implement update logic
        }
        kleos_types::ActionType::Delete => {
            warn!(request_id = request_id, "Handling DELETE action");
            // TODO: Implement delete logic
        }
        kleos_types::ActionType::View => {
            info!(request_id = request_id, "Handling VIEW action");
            // TODO: Implement view tracking logic
        }
    }

    Ok(())
}

/// Process system event
async fn process_system_event(
    request_id: &str,
    event: &kleos_types::SystemEventData,
) -> Result<(), anyhow::Error> {
    info!(
        request_id = request_id,
        source = %event.source,
        severity = ?event.severity,
        message = %event.message,
        "Processing system event"
    );

    // Skeleton processing logic based on severity
    match event.severity {
        kleos_types::Severity::Critical | kleos_types::Severity::Error => {
            error!(
                request_id = request_id,
                severity = ?event.severity,
                "High severity system event detected"
            );
            // TODO: Send alerts, create incidents, etc.
        }
        kleos_types::Severity::Warning => {
            warn!(request_id = request_id, "Warning system event");
            // TODO: Log for investigation
        }
        kleos_types::Severity::Info => {
            info!(request_id = request_id, "Informational system event");
            // TODO: Standard logging/metrics
        }
    }

    Ok(())
}

/// Process data processed event
async fn process_data_processed(
    request_id: &str,
    event: &kleos_types::DataProcessedEvent,
) -> Result<(), anyhow::Error> {
    info!(
        request_id = request_id,
        original_request = %event.original_request_id,
        "Processing data processed event"
    );

    // Handle based on processing result (illegal states impossible)
    match &event.result {
        kleos_types::ProcessingResult::Success {
            records_processed,
            duration_ms,
        } => {
            info!(
                request_id = request_id,
                records = records_processed,
                duration_ms = duration_ms,
                "Processing completed successfully"
            );
            // TODO: Update metrics, mark job complete, etc.
        }
        kleos_types::ProcessingResult::Failed {
            error_message,
            retry_count,
        } => {
            error!(
                request_id = request_id,
                error = %error_message,
                retry_count = retry_count,
                "Processing failed"
            );
            // TODO: Handle failure, possibly retry or alert
        }
    }

    Ok(())
}
