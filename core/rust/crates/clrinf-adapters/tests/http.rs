use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    middleware,
    routing::{get, post},
    Json, Router,
};
use clrinf_adapters::{context_middleware, propagate_context, AppError};
use clrinf_core::{ErrorEnvelope, RequestContext};
use serde_json::{json, Value};
use tower::ServiceExt;

fn router() -> Router {
    Router::new()
        .route(
            "/error",
            get(|| async { Err::<(), _>(AppError::Forbidden("Not allowed".into())) }),
        )
        .route(
            "/json",
            post(|Json(value): Json<Value>| async { Json(value) }),
        )
        .layer(middleware::from_fn(context_middleware))
}

#[tokio::test]
async fn contextual_error_is_exact_contract_and_headers_propagate() {
    let mut request = Request::builder()
        .uri("/error")
        .body(Body::empty())
        .unwrap();
    propagate_context(
        request.headers_mut(),
        &RequestContext::new("tenant", "workflow", "cause").unwrap(),
    )
    .unwrap();
    let response = router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.headers()["x-tenant-id"], "tenant");
    assert_eq!(response.headers()["x-correlation-id"], "workflow");
    assert_eq!(response.headers()["x-causation-id"], "cause");
    let value: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(
        value,
        json!({"error_code":"FORBIDDEN","message":"Not allowed","correlation_id":"workflow","tenant_id":"tenant","retryable":false})
    );
}

#[tokio::test]
async fn malformed_json_uses_same_error_contract() {
    let request = Request::builder()
        .method("POST")
        .uri("/json")
        .header("x-tenant-id", "tenant")
        .header("content-type", "application/json")
        .body(Body::from("{"))
        .unwrap();
    let response = router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let value: ErrorEnvelope =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(value.tenant_id, "tenant");
    assert!(!value.correlation_id.is_empty());
    assert!(!value.retryable);
}

#[tokio::test]
async fn absent_malformed_or_duplicate_tenants_are_rejected_without_default() {
    for tenants in [
        vec![],
        vec![""],
        vec!["tenant,other"],
        vec!["tenant", "other"],
    ] {
        let mut builder = Request::builder().uri("/error");
        for tenant in tenants {
            builder = builder.header("x-tenant-id", tenant);
        }
        let response = router()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(value["error_code"], "INVALID_CONTEXT");
        assert_eq!(value["tenant_id"], "");
    }
}
