use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_sdk_kinesis::Client as KinesisClient;
use kleos_ingest_lambda::UserAction;
use kleos_ingest_lambda::config::Config;
use kleos_ingest_lambda::ingestor::UserActionIngestor;
use kleos_ingest_lambda::publisher::KinesisPublisher;
use kleos_lib::{Ingestor, StreamPublisher};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use std::convert::TryFrom;

#[derive(Clone)]
struct IngestHandler<I, P>
where
    I: Ingestor<UserAction> + Clone,
    P: StreamPublisher<UserAction> + Clone,
{
    ingestor: I,
    publisher: P,
}

impl<I, P> IngestHandler<I, P>
where
    I: Ingestor<UserAction> + Clone,
    P: StreamPublisher<UserAction> + Clone,
{
    fn new(ingestor: I, publisher: P) -> Self {
        Self {
            ingestor,
            publisher,
        }
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

        // 2. Ingest: raw data -> Event<T, Created> (via Ingestor trait)
        let event = match self.ingestor.ingest(user_action).await {
            Ok(e) => e,
            Err(e) => {
                return Ok(ApiGatewayProxyResponse {
                    status_code: 500,
                    body: Some(format!("Ingestion error: {}", e).into()),
                    ..Default::default()
                });
            }
        };

        // 3. Publish: Event<T, Created> -> Stream (via StreamPublisher trait)
        match self.publisher.publish(&event).await {
            Ok(_) => Ok(ApiGatewayProxyResponse {
                status_code: 200,
                body: Some("Event ingested successfully".to_string().into()),
                ..Default::default()
            }),
            Err(e) => Ok(ApiGatewayProxyResponse {
                status_code: 500,
                body: Some(format!("Failed to publish event: {}", e).into()),
                ..Default::default()
            }),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load configuration at initialization - fail fast if config is invalid
    let config = Config::from_env().map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Initialize AWS SDK clients
    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let kinesis_client = KinesisClient::new(&aws_config);

    // Wire adapters implementing kleos-lib traits
    let ingestor = UserActionIngestor;
    let publisher = KinesisPublisher::new(kinesis_client, config.stream_name);

    // Create handler with trait implementations
    let handler = IngestHandler::new(ingestor, publisher);

    let func = service_fn(move |event| {
        let h = handler.clone();
        async move { h.handle(event).await }
    });

    lambda_runtime::run(func).await?;
    Ok(())
}
