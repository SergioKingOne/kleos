use aws_lambda_events::event::kinesis::KinesisEvent;
use kleos_ingest_lambda::UserAction;
use kleos_lib::{ProcessingResult, Processor, StreamConsumer};
use kleos_process_lambda::config::Config;
use kleos_process_lambda::consumer::KinesisBatchConsumer;
use kleos_process_lambda::processor::UserActionProcessor;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde_json::Value;

#[derive(Clone)]
struct ProcessHandler<C, P>
where
    C: StreamConsumer<UserAction> + Clone,
    P: Processor<UserAction> + Clone,
{
    consumer: C,
    processor: P,
}

impl<C, P> ProcessHandler<C, P>
where
    C: StreamConsumer<UserAction> + Clone,
    P: Processor<UserAction> + Clone,
{
    fn new(consumer: C, processor: P) -> Self {
        Self {
            consumer,
            processor,
        }
    }

    async fn handle(&self, event: LambdaEvent<KinesisEvent>) -> Result<Value, Error> {
        // Extract raw bytes from Kinesis records
        let records: Vec<&[u8]> = event
            .payload
            .records
            .iter()
            .map(|r| r.kinesis.data.as_ref())
            .collect();

        // Load batch into consumer (transitions Created -> Consumed)
        self.consumer
            .load(records)
            .await
            .map_err(|e| format!("Failed to load batch: {}", e))?;

        let mut processed_count = 0;
        let mut failed_count = 0;

        // Process each event via trait implementations
        while let Some(consumed_event) = self
            .consumer
            .consume()
            .await
            .map_err(|e| format!("Consume error: {}", e))?
        {
            match self.processor.process(&consumed_event).await {
                Ok(ProcessingResult::Success) => {
                    self.consumer
                        .ack(&consumed_event.id)
                        .await
                        .map_err(|e| format!("Ack error: {}", e))?;
                    processed_count += 1;
                }
                Ok(ProcessingResult::Failure(reason)) => {
                    eprintln!("Processing failed: {}", reason);
                    failed_count += 1;
                }
                Ok(ProcessingResult::Skipped(reason)) => {
                    println!("Skipped: {}", reason);
                    processed_count += 1;
                }
                Err(e) => {
                    eprintln!("Processing error: {}", e);
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
    let _config = Config::from_env().map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Wire adapters implementing kleos-lib traits
    let consumer = KinesisBatchConsumer::new();
    let processor = UserActionProcessor;

    // Create handler with trait implementations
    let handler = ProcessHandler::new(consumer, processor);

    let func = service_fn(move |event| {
        let h = handler.clone();
        async move { h.handle(event).await }
    });

    lambda_runtime::run(func).await?;
    Ok(())
}
