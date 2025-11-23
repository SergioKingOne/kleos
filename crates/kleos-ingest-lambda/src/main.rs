use aws_sdk_kinesis::Client as KinesisClient;
use aws_sdk_kinesis::primitives::Blob;
use kleos_ingest_lambda::UserAction;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct ApiGatewayResponse {
    status_code: u16,
    body: String,
}

#[derive(Clone)]
struct IngestHandler {
    client: KinesisClient,
    stream_name: String,
}

impl IngestHandler {
    fn new(client: KinesisClient) -> Self {
        let stream_name =
            std::env::var("KINESIS_STREAM_NAME").unwrap_or_else(|_| "kleos-stream".to_string());
        Self {
            client,
            stream_name,
        }
    }

    async fn handle(&self, event: LambdaEvent<Value>) -> Result<Value, Error> {
        // 1. Extract Body
        let body = if let Some(body) = event.payload.get("body") {
            body.as_str().unwrap_or("{}")
        } else {
            &event.payload.to_string()
        };

        // 2. Parse & Validate (Type Driven)
        let user_action = match UserAction::try_from_json(body) {
            Ok(action) => action,
            Err(e) => {
                return Ok(serde_json::to_value(ApiGatewayResponse {
                    status_code: 400,
                    body: format!("Invalid request: {}", e),
                })?);
            }
        };

        // 3. Wrap in System Event
        let event = user_action.into_event();

        // 4. Publish
        let data = match serde_json::to_vec(&event) {
            Ok(d) => d,
            Err(e) => {
                return Ok(serde_json::to_value(ApiGatewayResponse {
                    status_code: 500,
                    body: format!("Serialization error: {}", e),
                })?);
            }
        };

        match self
            .client
            .put_record()
            .stream_name(&self.stream_name)
            .data(Blob::new(data))
            .partition_key(event.id.to_string())
            .send()
            .await
        {
            Ok(_) => Ok(serde_json::to_value(ApiGatewayResponse {
                status_code: 200,
                body: "Event ingested successfully".to_string(),
            })?),
            Err(e) => Ok(serde_json::to_value(ApiGatewayResponse {
                status_code: 500,
                body: format!("Failed to ingest event: {}", e),
            })?),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let kinesis_client = KinesisClient::new(&config);

    let handler = IngestHandler::new(kinesis_client);

    let func = service_fn(move |event| {
        let h = handler.clone();
        async move { h.handle(event).await }
    });

    lambda_runtime::run(func).await?;
    Ok(())
}
