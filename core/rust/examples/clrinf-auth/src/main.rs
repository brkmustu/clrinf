use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use clrinf_adapters::{context_middleware, AppError};
use clrinf_auth::{
    AllowRule, AuthPolicy, Claims, ConfiguredCredentials, CredentialAdapter, DeclarativePolicy,
    JwtConfig, JwtService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

// ── App state ────────────────────────────────────────────────────────────────

struct AuthState {
    credentials: ConfiguredCredentials,
    jwt: JwtService,
    policy: DeclarativePolicy,
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct LoginRequest {
    id: String,
    credential: String,
    tenant_id: String,
    #[serde(default)]
    is_service_account: bool,
}

#[derive(Deserialize)]
struct VerifyRequest {
    token: String,
    tenant_id: String,
}

#[derive(Deserialize)]
struct AuthorizeRequest {
    token: String,
    tenant_id: String,
    action: String,
    resource: String,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
    expires_in: u32,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn app_error_response(e: AppError) -> (StatusCode, Json<Value>) {
    // AppError carries its message in each variant; extract it directly.
    let (status, code, message) = match e {
        AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", m),
        AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "FORBIDDEN", m),
        AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", m),
        AppError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m),
        AppError::Conflict(m) => (StatusCode::CONFLICT, "CONFLICT", m),
        AppError::Unavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE", m),
        AppError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Internal server error".into(),
        ),
    };
    (
        status,
        Json(json!({"error_code": code, "message": message, "retryable": false})),
    )
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "clrinf-auth",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

async fn jwks_endpoint(Extension(state): Extension<Arc<AuthState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.jwt.jwks().clone()))
}

async fn login(
    Extension(state): Extension<Arc<AuthState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let principal = state
        .credentials
        .authenticate(
            &req.id,
            &req.credential,
            &req.tenant_id,
            req.is_service_account,
        )
        .map_err(app_error_response)?;
    let token = state
        .jwt
        .issue(&principal, &req.tenant_id)
        .map_err(app_error_response)?;
    let ttl = state.jwt.ttl(req.is_service_account);
    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            token,
            expires_in: ttl,
        }),
    ))
}

async fn verify_token(
    Extension(state): Extension<Arc<AuthState>>,
    Json(req): Json<VerifyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let claims: Claims = state
        .jwt
        .verify(&req.token, &req.tenant_id)
        .map_err(app_error_response)?;
    Ok((StatusCode::OK, Json(json!(claims))))
}

async fn authorize(
    Extension(state): Extension<Arc<AuthState>>,
    Json(req): Json<AuthorizeRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let claims = state
        .jwt
        .verify(&req.token, &req.tenant_id)
        .map_err(app_error_response)?;
    if state.policy.is_allowed(&claims, &req.action, &req.resource) {
        Ok((StatusCode::OK, Json(json!({"allowed": true}))))
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(
                json!({"error_code": "FORBIDDEN", "message": "Action not allowed", "retryable": false}),
            ),
        ))
    }
}

// ── Bootstrap ─────────────────────────────────────────────────────────────────

fn load_state() -> anyhow::Result<AuthState> {
    // Credentials: loaded from CLRINF_PRINCIPALS_JSON env (JSON array of PrincipalConfig).
    let principals_json = std::env::var("CLRINF_PRINCIPALS_JSON").unwrap_or_else(|_| "[]".into());
    let principal_configs: Vec<clrinf_auth::PrincipalConfig> =
        serde_json::from_str(&principals_json)?;
    let credentials = ConfiguredCredentials::from_config(principal_configs, false)?;

    // JWT: RSA PEM content from env variables (not file paths).
    let private_pem = std::env::var("CLRINF_JWT_PRIVATE_PEM")
        .map_err(|_| anyhow::anyhow!("CLRINF_JWT_PRIVATE_PEM is required"))?;
    let public_pem = std::env::var("CLRINF_JWT_PUBLIC_PEM")
        .map_err(|_| anyhow::anyhow!("CLRINF_JWT_PUBLIC_PEM is required"))?;
    let jwt_config = JwtConfig {
        issuer: std::env::var("CLRINF_JWT_ISSUER").unwrap_or_else(|_| "clrinf/auth".into()),
        audience: std::env::var("CLRINF_JWT_AUDIENCE").unwrap_or_else(|_| "clrinf".into()),
        kid: std::env::var("CLRINF_JWT_KID").unwrap_or_else(|_| "default".into()),
        user_ttl_seconds: std::env::var("CLRINF_JWT_USER_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600),
        service_ttl_seconds: std::env::var("CLRINF_JWT_SERVICE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(86400),
    };
    let jwt = JwtService::new(jwt_config, private_pem.as_bytes(), public_pem.as_bytes())?;

    // Policy: loaded from CLRINF_POLICY_JSON env (JSON array of AllowRule).
    let policy_json = std::env::var("CLRINF_POLICY_JSON").unwrap_or_else(|_| "[]".into());
    let rules: Vec<AllowRule> = serde_json::from_str(&policy_json)?;
    let allow_wildcards = std::env::var("CLRINF_POLICY_WILDCARDS")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);
    let policy = DeclarativePolicy::new(rules, allow_wildcards)?;

    Ok(AuthState {
        credentials,
        jwt,
        policy,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,clrinf_auth=debug".into()),
        )
        .init();

    // clrinf port scheme: 51705 is the default for the auth IAM service.
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "51705".to_string())
        .parse()
        .unwrap_or(51705);

    let state = Arc::new(load_state()?);

    info!("Starting clrinf-auth IAM service on port {port}");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/.well-known/jwks.json", get(jwks_endpoint))
        .route("/auth/login", post(login))
        .route("/auth/verify", post(verify_token))
        .route("/auth/authorize", post(authorize))
        .layer(Extension(state))
        .layer(axum_middleware::from_fn(context_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Auth service listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
