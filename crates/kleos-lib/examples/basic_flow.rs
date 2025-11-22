use async_trait::async_trait;
use kleos_lib::{
    events::{Event, Created, Consumed, EventId, Payload},
    ingestion::{Ingestor, IngestError},
    stream::{StreamPublisher, StreamConsumer, StreamError},
    processing::{Processor, ProcessingError, ProcessingResult},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// --- Domain Data ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

// --- Implementations ---

// 1. Ingestor
struct UserIngestor;

#[async_trait]
impl Ingestor<User> for UserIngestor {
    async fn ingest(&self, data: User) -> Result<Event<User, Created>, IngestError> {
        println!("[Ingestor] Received user: {}", data.name);
        let event = Event::new(Payload::new(data));
        Ok(event)
    }
}

// 2. Stream (In-Memory)
struct InMemoryStream {
    sender: tokio::sync::mpsc::Sender<Event<User, Created>>,
    receiver: Arc<Mutex<tokio::sync::mpsc::Receiver<Event<User, Created>>>>,
}

impl InMemoryStream {
    fn new() -> (Self, Self) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stream = Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        };
        // Return two instances: one for publishing (uses sender), one for consuming (uses receiver)
        // Since we just clone the whole struct, both have both, but that's fine for a mock.
        (stream.clone(), stream)
    }
}

impl Clone for InMemoryStream {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

#[async_trait]
impl StreamPublisher<User> for InMemoryStream {
    async fn publish(&self, event: &Event<User, Created>) -> Result<(), StreamError> {
        println!("[Stream] Publishing event: {}", event.id);
        self.sender.send(event.clone()).await
            .map_err(|e| StreamError::PublishError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl StreamConsumer<User> for InMemoryStream {
    async fn consume(&self) -> Result<Option<Event<User, Consumed>>, StreamError> {
        let mut rx = self.receiver.lock().await;
        match rx.recv().await {
            Some(event) => {
                println!("[Stream] Consumed event: {}", event.id);
                // Transition state from Created to Consumed
                Ok(Some(event.transition()))
            }
            None => Ok(None),
        }
    }

    async fn ack(&self, event_id: &EventId) -> Result<(), StreamError> {
        println!("[Stream] Acknowledged event: {}", event_id);
        Ok(())
    }
}

// 3. Processor
struct UserProcessor;

#[async_trait]
impl Processor<User> for UserProcessor {
    async fn process(&self, event: &Event<User, Consumed>) -> Result<ProcessingResult, ProcessingError> {
        let user = &event.payload.data;
        println!("[Processor] Processing user: {} ({})", user.name, user.email);
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        println!("[Processor] User saved to DB (simulated)");
        Ok(ProcessingResult::Success)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Starting Kleos Basic Flow Example ---");

    // Setup
    let ingestor = UserIngestor;
    let (publisher, consumer) = InMemoryStream::new();
    let processor = UserProcessor;

    // Simulate Data
    let users = vec![
        User { id: "1".into(), name: "Alice".into(), email: "alice@example.com".into() },
        User { id: "2".into(), name: "Bob".into(), email: "bob@example.com".into() },
    ];

    // Run Flow
    for user in users {
        println!("\nProcessing flow for: {}", user.name);
        
        // 1. Ingest
        let event = ingestor.ingest(user).await?;
        
        // 2. Publish
        publisher.publish(&event).await?;

        // 3. Consume
        if let Some(consumed_event) = consumer.consume().await? {
            // 4. Process
            let result = processor.process(&consumed_event).await?;
            println!("[Main] Process result: {:?}", result);

            // 5. Ack
            consumer.ack(&consumed_event.id).await?;
        }
    }

    println!("\n--- Example Finished ---");
    Ok(())
}
