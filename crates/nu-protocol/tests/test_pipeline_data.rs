use nu_protocol::{IntoPipelineData, Span, Value};

#[test]
fn test_convert_pipeline_data_to_value() {
    // Setup PipelineData
    let value_val = 10;
    let value = Value::int(value_val, Span::new(1, 3));
    let pipeline_data = value.into_pipeline_data();

    // Test that conversion into Value is correct
    let new_span = Span::new(5, 6);
    let converted_value = pipeline_data.into_value(new_span);

    assert_eq!(converted_value, Ok(Value::int(value_val, new_span)));
}

#[test]
fn associated_constructors_preserve_values_streams_and_metadata() {
    use nu_protocol::{ByteStream, ListStream, PipelineData, PipelineMetadata, Signals};
    const EMPTY: PipelineData = PipelineData::empty();
    assert!(matches!(EMPTY, PipelineData::Empty));
    let span = Span::test_data();
    let metadata = PipelineMetadata::default().with_content_type(Some("text/plain".into()));
    let data = PipelineData::value(Value::test_int(42), metadata.clone());
    assert_eq!(data.metadata(), Some(metadata.clone()));
    assert_eq!(data.into_value(span).unwrap(), Value::int(42, span));
    let stream = ListStream::new(vec![Value::test_int(1)].into_iter(), span, Signals::empty());
    let optional_metadata = Some(metadata.clone());
    let data = PipelineData::list_stream(stream, optional_metadata);
    assert_eq!(data.metadata(), Some(metadata.clone()));
    assert_eq!(data.into_iter().count(), 1);
    let stream = ByteStream::read_string("hello".into(), span, Signals::empty());
    let data = PipelineData::byte_stream(stream, metadata.clone());
    assert_eq!(data.metadata(), Some(metadata));
    assert_eq!(data.into_value(span).unwrap(), Value::string("hello", span));
    assert!(
        PipelineData::value(Value::nothing(span), None)
            .metadata()
            .is_none()
    );
}
