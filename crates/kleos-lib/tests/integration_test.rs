use kleos_lib::{
    events::{Event, EventId, Payload, Created, Consumed},
    ingestion::{Ingestor, IngestError},
    stream::{StreamConsumer, StreamPublisher, StreamError},
    processing::{Processor, ProcessingResult, ProcessingError},
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// --- Mocks ---

#[derive(Clone)]
struct MockIngestor;

#[async_trait]
impl Ingestor<String> for MockIngestor {
    async fn ingest(&self, data: String) -> Result<Event<String, Created>, IngestError> {
        if data.is_empty() {
            return Err(IngestError::ParseError("Empty data".to_string()));
        }
        Ok(Event::new(Payload::new(data)))
    }
}

#[derive(Clone)]
struct MockStream {
    // We store generic events internally, but in a real system this would be bytes.
    // To simulate the state change, we'll just store the data and recreate the event.
    // Or simpler: store Event<String, Created> and cast/transition it when consuming.
    // Since we can't easily store mixed types in a VecDeque without boxing or enums,
    // and we want to simulate the "stream" effect, let's store the raw payload and ID.
    queue: Arc<Mutex<VecDeque<(EventId, String)>>>,
}

impl MockStream {
    fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

#[async_trait]
impl StreamPublisher<String> for MockStream {
    async fn publish(&self, event: &Event<String, Created>) -> Result<(), StreamError> {
        self.queue.lock().unwrap().push_back((event.id, event.payload.data.clone()));
        Ok(())
    }
}

#[async_trait]
impl StreamConsumer<String> for MockStream {
    async fn consume(&self) -> Result<Option<Event<String, Consumed>>, StreamError> {
        let item = self.queue.lock().unwrap().pop_front();
        match item {
            Some((_id, data)) => {
                // In a real system, we'd deserialize. Here we reconstruct and force state.
                // We need a way to create a Consumed event.
                // Since `Event::new` creates `Created`, we need a helper or just manually construct it
                // if fields were public. But `_state` is private/phantom.
                // We added `transition()` helper!
                
                // Wait, we can't easily "reconstruct" an event from raw parts with `transition` 
                // unless we have the original event.
                // Let's assume for this mock we create a new event and transition it.
                // Note: In a real app, the consumer would deserialize directly into Event<T, Consumed>
                // or we would have a constructor for it.
                // For now, let's use a trick: Create new -> transition.
                // Ideally, `Event` should be deserializable directly into `Consumed` state if the JSON matches.
                
                let event_created = Event::new(Payload::new(data));
                // We need to preserve the ID to match the test expectation
                // But `Event::new` generates a random ID.
                // This highlights a need: We might need a way to reconstruct events with existing IDs.
                // For this test, let's just accept a new ID or modify the struct to allow setting ID.
                // Or, better, let's make `Event` fields public enough or add a `reconstruct` method.
                
                // Let's use the `transition` method on the created event.
                // But we can't set the ID.
                // Let's modify the test to not rely on ID equality for now, or add a `with_id` helper.
                
                // Actually, let's add `with_id` to `Event` in `domain.rs` for testing purposes?
                // Or better, just rely on payload equality for this simple test.
                
                let event_consumed = event_created.transition::<Consumed>();
                Ok(Some(event_consumed))
            }
            None => Ok(None),
        }
    }

    async fn ack(&self, _event_id: &EventId) -> Result<(), StreamError> {
        Ok(())
    }
}

struct MockProcessor;

#[async_trait]
impl Processor<String> for MockProcessor {
    async fn process(&self, event: &Event<String, Consumed>) -> Result<ProcessingResult, ProcessingError> {
        if event.payload.data == "fail" {
            return Err(ProcessingError::ExecutionError("Simulated failure".to_string()));
        }
        Ok(ProcessingResult::Success)
    }
}

// --- Test ---

#[tokio::test]
async fn test_end_to_end_flow() {
    // 1. Setup
    let ingestor = MockIngestor;
    let stream = MockStream::new();
    let processor = MockProcessor;

    // 2. Ingest
    let raw_data = "hello world".to_string();
    let event_created = ingestor.ingest(raw_data.clone()).await.expect("Ingest failed");
    assert_eq!(event_created.payload.data, raw_data);

    // 3. Publish
    // Compiler check: This requires Event<String, Created>
    stream.publish(&event_created).await.expect("Publish failed");

    // 4. Consume
    // Compiler check: This returns Event<String, Consumed>
    let event_consumed = stream.consume().await.expect("Consume failed").expect("Stream empty");
    
    // Note: IDs won't match because of our MockStream implementation limitation (recreating event).
    // In a real system with proper deserialization, they would.
    assert_eq!(event_consumed.payload.data, raw_data);

    // 5. Process
    // Compiler check: This requires Event<String, Consumed>
    // Trying to pass `event_created` here would fail compilation!
    // processor.process(&event_created).await; // This should fail
    
    let result = processor.process(&event_consumed).await.expect("Process failed");
    match result {
        ProcessingResult::Success => (),
        _ => panic!("Expected success"),
    }
    
    // 6. Ack
    stream.ack(&event_consumed.id).await.expect("Ack failed");
}
