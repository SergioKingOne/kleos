use kleos_lib::stream::StreamError;

#[test]
fn test_stream_error_display() {
    let publish_err = StreamError::PublishError("kafka down".to_string());
    assert_eq!(publish_err.to_string(), "Failed to publish to stream: kafka down");

    let consume_err = StreamError::ConsumeError("empty".to_string());
    assert_eq!(consume_err.to_string(), "Failed to consume from stream: empty");

    let conn_err = StreamError::ConnectionError("timeout".to_string());
    assert_eq!(conn_err.to_string(), "Connection error: timeout");
}
