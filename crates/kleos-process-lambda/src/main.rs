use aws_lambda_events::event::kinesis::KinesisEvent;
use kleos_ingest_lambda::UserAction;
use kleos_lib::Event;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde_json::Value;

// Wrapper type to allow local impl
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
struct ProcessHandler;

impl ProcessHandler {
    async fn handle(&self, event: LambdaEvent<KinesisEvent>) -> Result<Value, Error> {
        let mut processed_count = 0;
        for record in event.payload.records {
            if let Ok(event) = serde_json::from_slice::<Event<UserAction>>(&record.kinesis.data) {
                // Wrap and execute
                let action = ProcessableAction(event.payload.data);
                action.execute();
                processed_count += 1;
            }
        }
        Ok(serde_json::json!({ "message": format!("Processed {} records", processed_count) }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = ProcessHandler;

    let func = service_fn(move |event| {
        let h = handler.clone();
        async move { h.handle(event).await }
    });

    lambda_runtime::run(func).await?;
    Ok(())
}
