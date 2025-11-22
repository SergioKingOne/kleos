use kleos_lib::events::{Event, Created, Payload};

#[test]
fn test_event_creation() {
    let payload = Payload::new("test_data".to_string());
    let event = Event::new(payload);

    assert_eq!(event.payload.data, "test_data");
    assert!(event.metadata.is_empty());
}

#[test]
fn test_event_metadata() {
    let payload = Payload::new("test_data".to_string());
    let event = Event::new(payload)
        .with_metadata("source".to_string(), "test".to_string())
        .with_metadata("priority".to_string(), "high".to_string());

    assert_eq!(event.metadata.get("source"), Some(&"test".to_string()));
    assert_eq!(event.metadata.get("priority"), Some(&"high".to_string()));
}

#[test]
fn test_event_transition() {
    let payload = Payload::new("test_data".to_string());
    let event: Event<String, Created> = Event::new(payload);
    let original_id = event.id;

    // Transition to Consumed (simulating the type change, although we need the target type to be available)
    // Since Consumed is public, we can use it.
    use kleos_lib::events::Consumed;
    
    let consumed_event: Event<String, Consumed> = event.transition();

    assert_eq!(consumed_event.id, original_id);
    assert_eq!(consumed_event.payload.data, "test_data");
}
