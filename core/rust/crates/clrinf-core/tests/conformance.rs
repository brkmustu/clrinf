use clrinf_core::{ErrorEnvelope, EventEnvelope, RequestContext};
use serde_json::Value;
use std::path::PathBuf;

fn fixture(name: &str) -> Value {
    let directory = std::env::var_os("CLRINF_CONFORMANCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../tests/conformance/fixtures")
        });
    let path = directory.join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "required fixture {}: {error}; set CLRINF_CONFORMANCE_DIR for standalone checkout",
            path.display()
        )
    });
    serde_json::from_slice(&bytes).expect("fixture must be JSON")
}

#[test]
fn canonical_contracts_roundtrip_without_losing_fields() {
    let valid = fixture("valid.json");
    let context: RequestContext = serde_json::from_value(valid["context"].clone()).unwrap();
    context.validate().unwrap();
    assert_eq!(serde_json::to_value(context).unwrap(), valid["context"]);
    let error: ErrorEnvelope = serde_json::from_value(valid["error"].clone()).unwrap();
    error.validate().unwrap();
    assert_eq!(serde_json::to_value(error).unwrap(), valid["error"]);
    let event: EventEnvelope = serde_json::from_value(valid["event"].clone()).unwrap();
    event.validate().unwrap();
    assert_eq!(serde_json::to_value(event).unwrap(), valid["event"]);
}

#[test]
fn canonical_invalid_contracts_are_rejected() {
    let invalid = fixture("invalid.json");
    for case in invalid["context"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<RequestContext>(case["value"].clone()).is_err(),
            "{}",
            case["name"]
        );
    }
    for case in invalid["error"].as_array().unwrap() {
        let result = serde_json::from_value::<ErrorEnvelope>(case["value"].clone())
            .map_err(|e| e.to_string())
            .and_then(|error| error.validate().map_err(|e| e.to_string()));
        assert!(result.is_err(), "{}", case["name"]);
    }
    for case in invalid["event"].as_array().unwrap() {
        let result = serde_json::from_value::<EventEnvelope>(case["value"].clone())
            .map_err(|e| e.to_string())
            .and_then(|event| event.validate().map_err(|e| e.to_string()));
        assert!(result.is_err(), "{}", case["name"]);
    }
}

#[test]
fn identifiers_reject_injection_and_child_preserves_tenant_and_workflow() {
    for value in [
        "",
        " ",
        "tenant\r\ninjected: yes",
        "tenant,other",
        "t/one",
        "tenant\u{1f4a5}",
    ] {
        assert!(RequestContext::new(value, "c", "cause").is_err());
    }
    assert!(RequestContext::new("t".repeat(129), "c", "cause").is_err());
    let root = RequestContext::root("tenant").unwrap();
    let child = root.child("parent-event").unwrap();
    assert_eq!(child.tenant_id, root.tenant_id);
    assert_eq!(child.correlation_id, root.correlation_id);
    assert_eq!(child.causation_id, "parent-event");
}

#[test]
fn envelope_flags_are_explicit_and_optional_nulls_are_rejected() {
    let context = RequestContext::root("tenant").unwrap();
    let mut event = EventEnvelope::new(
        &context,
        "module",
        "NotAnErrorCompensated",
        Default::default(),
    )
    .unwrap();
    assert_eq!(event.is_error, None);
    assert_eq!(event.is_compensation, None);
    event.is_error = Some(false);
    event.is_compensation = Some(true);
    let mut value = serde_json::to_value(&event).unwrap();
    assert_eq!(value["is_error"], false);
    assert_eq!(value["is_compensation"], true);
    value["is_error"] = Value::Null;
    assert!(serde_json::from_value::<EventEnvelope>(value).is_err());
    event.time = "not-a-timestamp".into();
    assert!(event.validate().is_err());
}
