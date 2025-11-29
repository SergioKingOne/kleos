use async_trait::async_trait;
use kleos_ingest_lambda::UserAction;
use kleos_lib::{Consumed, Created, Event, EventId, StreamConsumer, StreamError};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Consumes Kinesis batch events as Event<UserAction, Consumed>.
#[derive(Clone)]
pub struct KinesisBatchConsumer {
    pending: Arc<Mutex<VecDeque<Event<UserAction, Consumed>>>>,
}

impl KinesisBatchConsumer {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Default for KinesisBatchConsumer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamConsumer<UserAction> for KinesisBatchConsumer {
    async fn load(&self, records: Vec<&[u8]>) -> Result<(), StreamError> {
        let mut pending = self.pending.lock().await;
        for data in records {
            let event: Event<UserAction, Created> = serde_json::from_slice(data)
                .map_err(|e| StreamError::ConsumeError(e.to_string()))?;

            let consumed: Event<UserAction, Consumed> = event.transition();
            pending.push_back(consumed);
        }
        Ok(())
    }

    async fn consume(&self) -> Result<Option<Event<UserAction, Consumed>>, StreamError> {
        Ok(self.pending.lock().await.pop_front())
    }

    async fn ack(&self, _event_id: &EventId) -> Result<(), StreamError> {
        Ok(())
    }
}
