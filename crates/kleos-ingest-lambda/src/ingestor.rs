use async_trait::async_trait;
use kleos_lib::{Created, Event, IngestError, Ingestor, Payload};

use crate::UserAction;

/// Converts validated UserAction domain objects into Event<UserAction, Created>.
#[derive(Clone)]
pub struct UserActionIngestor;

#[async_trait]
impl Ingestor<UserAction> for UserActionIngestor {
    async fn ingest(&self, data: UserAction) -> Result<Event<UserAction, Created>, IngestError> {
        Ok(Event::new(Payload::new(data)))
    }
}
