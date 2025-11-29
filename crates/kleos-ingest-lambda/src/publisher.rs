use async_trait::async_trait;
use aws_sdk_kinesis::Client as KinesisClient;
use aws_sdk_kinesis::primitives::Blob;
use kleos_lib::{Created, Event, StreamError, StreamPublisher};

use crate::UserAction;

/// Publishes Event<UserAction, Created> to Kinesis.
#[derive(Clone)]
pub struct KinesisPublisher {
    client: KinesisClient,
    stream_name: String,
}

impl KinesisPublisher {
    pub fn new(client: KinesisClient, stream_name: String) -> Self {
        Self {
            client,
            stream_name,
        }
    }
}

#[async_trait]
impl StreamPublisher<UserAction> for KinesisPublisher {
    async fn publish(&self, event: &Event<UserAction, Created>) -> Result<(), StreamError> {
        let data =
            serde_json::to_vec(event).map_err(|e| StreamError::PublishError(e.to_string()))?;

        self.client
            .put_record()
            .stream_name(&self.stream_name)
            .data(Blob::new(data))
            .partition_key(event.id.to_string())
            .send()
            .await
            .map_err(|e| StreamError::PublishError(e.to_string()))?;

        Ok(())
    }
}
