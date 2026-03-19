use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::AppConfig;
use crate::exporter;
use crate::model::{
    ConfigResponse, ExportRequest, HealthResponse, MonitorRequest, WebhookSignatureResponse,
    WebhookVerifyResponse,
};
use crate::monitor::MonitorService;
use crate::{analytics, webhook};

#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<AppConfig>,
    pub monitor_service: MonitorService,
}

pub async fn run(state: ApiState, host: String, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/config", get(config))
        .route("/api/v1/monitors", post(create_monitor))
        .route("/api/v1/events", post(fetch_events))
        .route("/api/v1/analyze", post(analyze_events))
        .route("/api/v1/export", post(export_events))
        .route("/api/v1/webhook/sign", post(sign_webhook))
        .route("/api/v1/webhook/verify", post(verify_webhook))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!(address = %addr, "xenon http server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<ApiState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        profile: state.config.profile.clone(),
        x_api_configured: state.monitor_service.is_configured(),
    })
}

async fn config(State(state): State<ApiState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        profile: state.config.profile.clone(),
        x_api_base_url: state.config.x_api_base_url.clone(),
        x_api_configured: state.monitor_service.is_configured(),
        webhook_secret_configured: state.config.webhook_secret.is_some(),
        request_timeout_seconds: state.config.request_timeout_seconds,
    })
}

async fn create_monitor(
    State(state): State<ApiState>,
    Json(payload): Json<MonitorRequest>,
) -> impl IntoResponse {
    match state.monitor_service.generate_batch(&payload).await {
        Ok(batch) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "accepted": true,
                "profile": state.config.profile,
                "preview": batch.into_iter().take(3).collect::<Vec<_>>()
            })),
        )
            .into_response(),
        Err(error) => service_error(error),
    }
}

async fn fetch_events(
    State(state): State<ApiState>,
    Json(payload): Json<MonitorRequest>,
) -> impl IntoResponse {
    match state.monitor_service.generate_batch(&payload).await {
        Ok(events) => (
            StatusCode::OK,
            Json(serde_json::json!({ "events": events })),
        )
            .into_response(),
        Err(error) => service_error(error),
    }
}

async fn analyze_events(
    State(state): State<ApiState>,
    Json(payload): Json<MonitorRequest>,
) -> impl IntoResponse {
    match state.monitor_service.generate_batch(&payload).await {
        Ok(events) => (
            StatusCode::OK,
            Json(serde_json::json!(analytics::summarize(&events))),
        )
            .into_response(),
        Err(error) => service_error(error),
    }
}

async fn export_events(
    State(state): State<ApiState>,
    Json(payload): Json<ExportRequest>,
) -> impl IntoResponse {
    let request = MonitorRequest {
        handle: payload.handle,
        kinds: payload.kinds,
        limit: payload.limit,
    };

    match state.monitor_service.generate_batch(&request).await {
        Ok(events) => match exporter::render(&events, payload.format) {
            Ok(export) => (StatusCode::OK, Json(serde_json::json!(export))).into_response(),
            Err(error) => service_error(error),
        },
        Err(error) => service_error(error),
    }
}

async fn sign_webhook(
    State(state): State<ApiState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let secret = match payload
        .get("secret")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| state.config.webhook_secret.clone())
    {
        Some(secret) => secret,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "missing secret" })),
            )
                .into_response()
        }
    };

    let body = payload
        .get("payload")
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    match webhook::sign_payload(&secret, body) {
        Ok(signature) => (
            StatusCode::OK,
            Json(serde_json::json!(WebhookSignatureResponse {
                algorithm: "hmac-sha256",
                signature,
            })),
        )
            .into_response(),
        Err(error) => service_error(error),
    }
}

async fn verify_webhook(
    State(state): State<ApiState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let secret = match payload
        .get("secret")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| state.config.webhook_secret.clone())
    {
        Some(secret) => secret,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "missing secret" })),
            )
                .into_response()
        }
    };

    let body = payload
        .get("payload")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let signature = payload
        .get("signature")
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    match webhook::verify_payload(&secret, body, signature) {
        Ok(valid) => (
            StatusCode::OK,
            Json(serde_json::json!(WebhookVerifyResponse { valid })),
        )
            .into_response(),
        Err(error) => service_error(error),
    }
}

fn service_error(error: anyhow::Error) -> axum::response::Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({
            "error": error.to_string()
        })),
    )
        .into_response()
}
