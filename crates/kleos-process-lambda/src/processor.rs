use async_trait::async_trait;
use kleos_ingest_lambda::UserAction;
use kleos_lib::{Consumed, Event, ProcessingError, ProcessingResult, Processor};

/// Processes Event<UserAction, Consumed> with business logic.
#[derive(Clone)]
pub struct UserActionProcessor;

#[async_trait]
impl Processor<UserAction> for UserActionProcessor {
    async fn process(
        &self,
        event: &Event<UserAction, Consumed>,
    ) -> Result<ProcessingResult, ProcessingError> {
        match &event.payload.data {
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
        Ok(ProcessingResult::Success)
    }
}
