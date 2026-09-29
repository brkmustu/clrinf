use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use clrinf_adapters::{propagate_context, MemoryWorkflowStore};
use clrinf_core::{persistence::OutboxStore, RequestContext};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn http_and_in_process_share_contract_and_idempotency() {
    let store = Arc::new(MemoryWorkflowStore::default());
    let context = RequestContext::new("tenant", "workflow", "cause").unwrap();
    let message = json!({"text": "hello"}).as_object().unwrap().clone();
    let first =
        clrinf_generic::record_message(store.as_ref(), &context, "key", message, 0).unwrap();
    let mut request = Request::builder()
        .method("POST")
        .uri("/messages/key")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"text":"hello"}"#))
        .unwrap();
    propagate_context(request.headers_mut(), &context).unwrap();
    let response = clrinf_generic::http::router(store.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let replay: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(first, replay);
    let events = store.pending("tenant", 100).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].context().unwrap(), context);
    assert_eq!(events[0].data["text"], "hello");

    let mut request = Request::builder()
        .method("POST")
        .uri("/messages/key")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"text":"changed"}"#))
        .unwrap();
    propagate_context(request.headers_mut(), &context).unwrap();
    let response = clrinf_generic::http::router(store.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), 409);
    assert_eq!(store.pending("tenant", 100).unwrap().len(), 1);
}
