mod modeller;
mod motor;
mod servis;

use axum::{
    extract::Extension,
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clrinf_adapters::context_middleware;
use serde_json::json;
use servis::{hesapla_kampanya, listele_kampanyalar, KampanyaDeposu};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "clrinf-campaign-engine",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Structured JSON logging
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,clrinf_kampanya=debug".into()),
        )
        .init();

    info!("Starting clrinf-kampanya Constraint Solver Engine (Port 8082)...");

    let kampanya_deposu = Arc::new(KampanyaDeposu::new());

    let app = Router::new()
        .route("/kampanya/hesapla", post(hesapla_kampanya))
        .route("/kampanya/listele", get(listele_kampanyalar))
        .layer(Extension(kampanya_deposu))
        .layer(axum_middleware::from_fn(context_middleware))
        .route("/health", get(health_check))
        .route("/saglik", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .unwrap_or(8082);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Campaign Engine listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
