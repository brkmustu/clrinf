use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use clrinf_core::{ErrorEnvelope, RequestContext, ValidationError};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Unavailable(String),
    Internal(String),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::Unauthorized(message) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message.clone())
            }
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, "FORBIDDEN", message.clone()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", message.clone()),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "NOT_FOUND", message.clone()),
            Self::Conflict(message) => (StatusCode::CONFLICT, "CONFLICT", message.clone()),
            Self::Unavailable(message) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "UNAVAILABLE",
                message.clone(),
            ),
            Self::Internal(message) => {
                tracing::error!(error = %message, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".into(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        // Outside middleware there is no trusted request identity.
        let mut response = error_response(status, code, message, None);
        response.extensions_mut().insert(self);
        response
    }
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: String,
    context: Option<&RequestContext>,
) -> Response {
    let error = ErrorEnvelope {
        error_code: code.into(),
        message,
        correlation_id: context
            .map(|c| c.correlation_id.clone())
            .unwrap_or_default(),
        tenant_id: context.map(|c| c.tenant_id.clone()).unwrap_or_default(),
        retryable: status == StatusCode::SERVICE_UNAVAILABLE
            || status == StatusCode::TOO_MANY_REQUESTS,
        details: None,
    };
    (status, Json(error)).into_response()
}

fn header(headers: &HeaderMap, name: &'static str) -> Result<Option<String>, ValidationError> {
    let mut values = headers.get_all(name).iter();
    let value = values.next();
    if values.next().is_some() {
        return Err(ValidationError(format!("duplicate {name}")));
    }
    value
        .map(|v| {
            v.to_str()
                .map(str::to_owned)
                .map_err(|_| ValidationError(format!("invalid {name}")))
        })
        .transpose()
}

pub fn extract_context(headers: &HeaderMap) -> Result<RequestContext, ValidationError> {
    RequestContext::new(
        header(headers, "x-tenant-id")?
            .ok_or_else(|| ValidationError("x-tenant-id is required".into()))?,
        header(headers, "x-correlation-id")?.unwrap_or_else(|| Uuid::new_v4().to_string()),
        header(headers, "x-causation-id")?.unwrap_or_else(|| Uuid::new_v4().to_string()),
    )
}

pub fn propagate_context(
    headers: &mut HeaderMap,
    context: &RequestContext,
) -> Result<(), ValidationError> {
    context.validate()?;
    for (name, value) in [
        ("x-tenant-id", &context.tenant_id),
        ("x-correlation-id", &context.correlation_id),
        ("x-causation-id", &context.causation_id),
    ] {
        headers.insert(
            name,
            HeaderValue::from_str(value).map_err(|_| ValidationError(format!("invalid {name}")))?,
        );
    }
    Ok(())
}

pub async fn context_middleware(mut req: Request, next: Next) -> Response {
    let context = match extract_context(req.headers()) {
        Ok(context) => context,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "INVALID_CONTEXT",
                error.to_string(),
                None,
            )
        }
    };
    req.extensions_mut().insert(context.clone());
    let mut response = next.run(req).await;
    if let Some(error) = response.extensions_mut().remove::<AppError>() {
        let (status, code, message) = error.parts();
        response = error_response(status, code, message, Some(&context));
    } else if response.status().is_client_error() || response.status().is_server_error() {
        let status = response.status();
        // Extractor, routing and other framework rejections must use the wire contract too.
        response = error_response(
            status,
            status
                .canonical_reason()
                .unwrap_or("HTTP_ERROR")
                .to_ascii_uppercase()
                .replace(' ', "_")
                .as_str(),
            status.canonical_reason().unwrap_or("HTTP error").into(),
            Some(&context),
        );
    }
    if let Err(error) = propagate_context(response.headers_mut(), &context) {
        return AppError::Internal(error.to_string()).into_response();
    }
    response
}
