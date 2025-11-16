//! API Gateway Lambda Producer
//!
//! Receives HTTP requests via API Gateway and pushes events to Kinesis Data Stream.
//!
//! Architecture:
//! - Initializes AWS SDK clients once (main) for reuse across invocations
//! - Validates input using type-driven domain types
//! - Pushes to Kinesis with proper error handling
//! - Returns appropriate HTTP responses

use aws_sdk_kinesis::{Client as KinesisClient, primitives::Blob};
use kleos_types::{
    ActionType, EventPayload, PartitionKey, RequestId, StreamEvent, String100, String1000,
    UserActionEvent,
};
use lambda_http::{Body, Error, Request, Response, run, service_fn};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Environment variable for Kinesis stream name
const STREAM_NAME_ENV: &str = "KINESIS_STREAM_NAME";

/// Request body from API Gateway
#[derive(Debug, Deserialize)]
struct ApiRequest {
    user_id: String,
    action: String,
    details: String,
}

/// Response body to API Gateway
#[derive(Debug, Serialize)]
struct ApiResponse {
    request_id: String,
    message: String,
}

/// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Main Lambda handler state
struct HandlerState {
    kinesis_client: KinesisClient,
    stream_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize structured logging (CloudWatch adds timestamps)
    tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .without_time()
        .init();

    info!("Initializing API Producer Lambda");

    // Load AWS SDK configuration once (reused across invocations)
    let config = aws_config::load_from_env().await;

    // Initialize Kinesis client once (100-150ms saved per invocation)
    let kinesis_client = KinesisClient::new(&config);

    // Get stream name from environment
    let stream_name = std::env::var(STREAM_NAME_ENV).unwrap_or_else(|_| {
        error!("Missing environment variable: {}", STREAM_NAME_ENV);
        "kleos-stream".to_string() // Default for development
    });

    info!(stream_name = %stream_name, "Kinesis stream configured");

    // Create handler state
    let state = HandlerState {
        kinesis_client,
        stream_name,
    };

    // Run the Lambda runtime
    run(service_fn(|event| handler(&state, event))).await
}

/// Handler for API Gateway requests
async fn handler(state: &HandlerState, event: Request) -> Result<Response<Body>, Error> {
    // Generate request ID for tracing
    let request_id = RequestId::generate();

    info!(
        request_id = %request_id,
        method = %event.method(),
        path = %event.uri().path(),
        "Received API request"
    );

    // Parse and validate request body
    let api_request = match parse_request(&event) {
        Ok(req) => req,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Invalid request");
            return Ok(error_response(400, format!("Invalid request: {}", e)));
        }
    };

    // Convert to domain types with validation
    let domain_event = match create_domain_event(request_id.clone(), api_request) {
        Ok(event) => event,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Validation failed");
            return Ok(error_response(400, format!("Validation failed: {}", e)));
        }
    };

    // Push to Kinesis
    match push_to_kinesis(state, &domain_event).await {
        Ok(_) => {
            info!(request_id = %request_id, "Successfully pushed to Kinesis");

            let response = ApiResponse {
                request_id: request_id.to_string(),
                message: "Event accepted".to_string(),
            };

            Ok(success_response(response))
        }
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "Failed to push to Kinesis"
            );
            Ok(error_response(500, "Failed to process request".to_string()))
        }
    }
}

/// Parse request body from API Gateway event
fn parse_request(event: &Request) -> Result<ApiRequest, anyhow::Error> {
    let body = event.body();

    match body {
        Body::Text(text) => {
            serde_json::from_str(text).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
        }
        Body::Binary(bytes) => {
            serde_json::from_slice(bytes).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
        }
        Body::Empty => Err(anyhow::anyhow!("Empty request body")),
    }
}

/// Convert API request to validated domain event
fn create_domain_event(
    request_id: RequestId,
    api_request: ApiRequest,
) -> Result<StreamEvent, anyhow::Error> {
    // Validate and create domain types (construction-time validation)
    let user_id = String100::create(api_request.user_id)
        .map_err(|e| anyhow::anyhow!("Invalid user_id: {}", e))?;

    let details = String1000::create(api_request.details)
        .map_err(|e| anyhow::anyhow!("Invalid details: {}", e))?;

    // Parse action type
    let action_type = match api_request.action.to_lowercase().as_str() {
        "create" => ActionType::Create,
        "update" => ActionType::Update,
        "delete" => ActionType::Delete,
        "view" => ActionType::View,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid action type: {}",
                api_request.action
            ));
        }
    };

    // Create event payload
    let payload = EventPayload::UserAction(UserActionEvent {
        user_id,
        action_type,
        details,
    });

    // Create stream event
    Ok(StreamEvent::new(request_id, payload))
}

/// Push event to Kinesis Data Stream
async fn push_to_kinesis(state: &HandlerState, event: &StreamEvent) -> Result<(), anyhow::Error> {
    // Serialize event to JSON
    let data = serde_json::to_string(event)?;

    // Convert to Blob
    let blob = Blob::new(data.as_bytes());

    // Use request_id as partition key for even distribution
    let partition_key = PartitionKey::create(event.request_id.to_string())
        .ok_or_else(|| anyhow::anyhow!("Invalid partition key"))?;

    // Put record to Kinesis
    state
        .kinesis_client
        .put_record()
        .stream_name(&state.stream_name)
        .data(blob)
        .partition_key(partition_key.as_str())
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Kinesis put_record failed: {}", e))?;

    Ok(())
}

/// Create successful HTTP response
fn success_response(body: ApiResponse) -> Response<Body> {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::Text(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

/// Create error HTTP response
fn error_response(status: u16, message: String) -> Response<Body> {
    let error = ErrorResponse { error: message };

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::Text(serde_json::to_string(&error).unwrap()))
        .unwrap()
}
