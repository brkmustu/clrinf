#[path = "../../core/rust/mod.rs"]
mod core;
#[path = "../../../../tools/clrinf-codegen/tests/golden/rust/mod.rs"]
mod golden;

#[test]
fn canonical_context_and_error_round_trip_with_original_wire_names() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("../../../conformance/fixtures/valid.json")).unwrap();
    let context: core::RequestContext = serde_json::from_value(fixture["context"].clone()).unwrap();
    assert_eq!(serde_json::to_value(context).unwrap(), fixture["context"]);
    let error: core::StandardErrorEnvelope = serde_json::from_value(fixture["error"].clone()).unwrap();
    assert_eq!(serde_json::to_value(error).unwrap(), fixture["error"]);
    let envelope: core::CloudEventEnvelope = serde_json::from_value(fixture["event"].clone()).unwrap();
    assert_eq!(serde_json::to_value(envelope).unwrap(), fixture["event"]);
}

#[test]
fn recursive_array_object_and_reserved_fields_keep_the_wire_shape() {
    let value = serde_json::json!({
        "displayName": "sample",
        "enabled": true,
        "nested": [[1, 2]],
        "payload": {"value": "nested"},
        "type": "first",
        "records": [{"any": 7}]
    });
    let record: golden::RecordV1 = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(record).unwrap(), value);
}
