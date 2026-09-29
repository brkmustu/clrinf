use clrinf_core::{
    persistence::{Claim, StoreError, WorkflowStore},
    EventEnvelope, RequestContext,
};
use serde_json::{Map, Value};

/// A transport-independent module. Callers supply a trusted clock in Unix seconds.
pub fn record_message(
    store: &dyn WorkflowStore,
    context: &RequestContext,
    key: &str,
    message: Map<String, Value>,
    now: u64,
) -> Result<Value, StoreError> {
    context.validate()?;
    // serde_json's default Map is key-sorted, so object key order does not change the fingerprint.
    let fingerprint = serde_json::to_string(&message)
        .map_err(|error| StoreError::Unavailable(error.to_string()))?;
    match store.claim(&context.tenant_id, key, &fingerprint, now, 30)? {
        Claim::Acquired(lease) => {
            let event = EventEnvelope::new(
                context,
                "clrinf/messages",
                "com.clrinf.MessageRecorded.v1",
                message,
            )?;
            let result = serde_json::json!({"event_id": event.id});
            store.commit(&lease, now, result.clone(), vec![event])?;
            Ok(result)
        }
        Claim::Replay(result) => Ok(result),
        Claim::InProgress => Err(StoreError::InProgress),
        Claim::Conflict => Err(StoreError::Conflict),
    }
}

pub mod http {
    use super::record_message;
    use axum::{
        extract::{Extension, Path, State},
        middleware,
        routing::{get, post},
        Json, Router,
    };
    use clrinf_adapters::{context_middleware, AppError, MemoryWorkflowStore};
    use clrinf_core::{persistence::StoreError, RequestContext};
    use serde_json::{Map, Value};
    use std::{
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub fn router(store: Arc<MemoryWorkflowStore>) -> Router {
        Router::new()
            .route("/messages/:key", post(record))
            .layer(middleware::from_fn(context_middleware))
            .route(
                "/health",
                get(|| async { Json(serde_json::json!({"status": "healthy"})) }),
            )
            .with_state(store)
    }

    async fn record(
        State(store): State<Arc<MemoryWorkflowStore>>,
        Extension(context): Extension<RequestContext>,
        Path(key): Path<String>,
        Json(message): Json<Map<String, Value>>,
    ) -> Result<Json<Value>, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| AppError::Internal(error.to_string()))?
            .as_secs();
        record_message(store.as_ref(), &context, &key, message, now)
            .map(Json)
            .map_err(|error| match error {
                StoreError::Invalid(error) => AppError::BadRequest(error.to_string()),
                StoreError::Conflict => AppError::Conflict("idempotency payload conflict".into()),
                StoreError::LeaseLost => {
                    AppError::Conflict("idempotency lease expired or reclaimed".into())
                }
                StoreError::InProgress => {
                    AppError::Unavailable("request already in progress".into())
                }
                StoreError::Unavailable(_) => {
                    AppError::Unavailable("persistence unavailable".into())
                }
                error => AppError::Internal(error.to_string()),
            })
    }
}
