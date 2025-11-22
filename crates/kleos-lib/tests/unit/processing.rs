use kleos_lib::processing::{ProcessingError, ProcessingResult};

#[test]
fn test_processing_error_display() {
    let error = ProcessingError::ExecutionError("something went wrong".to_string());
    assert_eq!(error.to_string(), "Processing failed: something went wrong");

    let transient = ProcessingError::TransientError("timeout".to_string());
    assert_eq!(transient.to_string(), "Transient error, retriable: timeout");
}

#[test]
fn test_processing_result_serialization() {
    let success = ProcessingResult::Success;
    let serialized = serde_json::to_string(&success).unwrap();
    assert_eq!(serialized, "\"Success\"");

    let failure = ProcessingResult::Failure("bad data".to_string());
    let serialized = serde_json::to_string(&failure).unwrap();
    assert_eq!(serialized, "{\"Failure\":\"bad data\"}");

    let skipped = ProcessingResult::Skipped("duplicate".to_string());
    let serialized = serde_json::to_string(&skipped).unwrap();
    assert_eq!(serialized, "{\"Skipped\":\"duplicate\"}");
}
