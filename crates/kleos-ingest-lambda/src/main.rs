use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_sdk_kinesis::Client as KinesisClient;
use aws_sdk_kinesis::primitives::Blob;
use kleos_ingest_lambda::config::Config;
use kleos_ingest_lambda::UserAction;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use std::convert::TryFrom;

#[derive(Clone)]
struct IngestHandler {
    config: Config,
    client: KinesisClient,
}

impl IngestHandler {
    fn new(config: Config, client: KinesisClient) -> Self {
        Self { config, client }
    }

    async fn handle(
        &self,
        event: LambdaEvent<ApiGatewayProxyRequest>,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // 1. Extract & Validate (Type Driven with TryFrom)
        let user_action = match UserAction::try_from(event.payload) {
            Ok(action) => action,
            Err(e) => {
                return Ok(ApiGatewayProxyResponse {
                    status_code: 400,
                    body: Some(format!("Invalid request: {}", e).into()),
                    ..Default::default()
                });
            }
        };

        // 2. Wrap in System Event
        let event = user_action.into_event();

        // 3. Publish to Kinesis
        let data = match serde_json::to_vec(&event) {
            Ok(d) => d,
            Err(e) => {
                return Ok(ApiGatewayProxyResponse {
                    status_code: 500,
                    body: Some(format!("Serialization error: {}", e).into()),
                    ..Default::default()
                });
            }
        };

        match self
            .client
            .put_record()
            .stream_name(&self.config.stream_name)
            .data(Blob::new(data))
            .partition_key(event.id.to_string())
            .send()
            .await
        {
            Ok(_) => Ok(ApiGatewayProxyResponse {
                status_code: 200,
                body: Some("Event ingested successfully".to_string().into()),
                ..Default::default()
            }),
            Err(e) => Ok(ApiGatewayProxyResponse {
                status_code: 500,
                body: Some(format!("Failed to ingest event: {}", e).into()),
                ..Default::default()
            }),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load configuration at initialization - fail fast if config is invalid
    let config = Config::from_env()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Initialize AWS SDK clients
    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let kinesis_client = KinesisClient::new(&aws_config);

    // Create handler with config and clients
    let handler = IngestHandler::new(config, kinesis_client);

    let func = service_fn(move |event| {
        let h = handler.clone();
        async move { h.handle(event).await }
    });

    lambda_runtime::run(func).await?;
    Ok(())
}
