use kleos_lib::ingestion::IngestError;

#[test]
fn test_ingest_error_display() {
    let parse_err = IngestError::ParseError("invalid json".to_string());
    assert_eq!(parse_err.to_string(), "Failed to parse payload: invalid json");

    let sys_err = IngestError::SystemError("disk full".to_string());
    assert_eq!(sys_err.to_string(), "Ingestion failed: disk full");
}
