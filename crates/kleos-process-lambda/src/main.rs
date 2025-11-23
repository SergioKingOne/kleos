use aws_lambda_events::event::kinesis::KinesisEvent;
use kleos_ingest_lambda::UserAction;
use kleos_lib::Event;
use kleos_process_lambda::ProcessableRecord;
use kleos_process_lambda::config::Config;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde_json::Value;
use std::convert::TryFrom;

struct ProcessableAction(UserAction);

impl ProcessableAction {
    pub fn execute(&self) {
        match &self.0 {
            UserAction::PageView { user_id, url } => {
                println!("User {} viewed page {}", user_id, url);
            }
            UserAction::Click {
                user_id,
                element_id,
                url,
            } => {
                println!("User {} clicked {} on {}", user_id, element_id, url);
            }
            UserAction::Purchase {
                user_id,
                product_id,
                amount,
            } => {
                println!("User {} purchased {} for {}", user_id, product_id, amount);
            }
        }
    }
}

#[derive(Clone)]
struct ProcessHandler {
    _config: Config,
}

impl ProcessHandler {
    fn new(config: Config) -> Self {
        Self { _config: config }
    }
}

impl ProcessHandler {
    async fn handle(&self, event: LambdaEvent<KinesisEvent>) -> Result<Value, Error> {
        let mut processed_count = 0;
        let mut failed_count = 0;

        for record in event.payload.records {
            // Extract record with metadata using TryFrom
            match ProcessableRecord::<Event<UserAction>>::try_from(record) {
                Ok(processable) => {
                    // Execute the action
                    let action = ProcessableAction(processable.data.payload.data);
                    action.execute();

                    // Log with metadata for observability
                    println!(
                        "Processed record {} from partition {}",
                        processable.metadata.sequence_number, processable.metadata.partition_key
                    );

                    processed_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to process record: {}", e);
                    failed_count += 1;
                }
            }
        }

        Ok(serde_json::json!({
            "processed": processed_count,
            "failed": failed_count
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load configuration at initialization - fail fast if config is invalid
    let config = Config::from_env().map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Create handler with config
    let handler = ProcessHandler::new(config);

    let func = service_fn(move |event| {
        let h = handler.clone();
        async move { h.handle(event).await }
    });

    lambda_runtime::run(func).await?;
    Ok(())
}
