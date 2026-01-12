mod db;
mod handler;
mod resources;
mod tools;

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use handler::PostgresHandler;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

struct AppState {
    handler: Arc<PostgresHandler>,
}

async fn handle_jsonrpc(
    State(state): State<Arc<AppState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    if request.jsonrpc != "2.0" {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: "Invalid JSON-RPC version".to_string(),
                    data: None,
                }),
            }),
        )
            .into_response();
    }

    match state.handler.handle_request(&request.method, request.params).await {
        Ok(result) => (
            StatusCode::OK,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Error handling request {}: {}", request.method, e);
            (
                StatusCode::OK,
                Json(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    }),
                }),
            )
                .into_response()
        }
    }
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL environment variable not set")?;

    let allow_write_ops = env::var("DANGEROUSLY_ALLOW_WRITE_OPS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let host = env::var("MCP_SERVER_HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("MCP_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    if allow_write_ops {
        info!("⚠️  Write operations are ENABLED");
    } else {
        info!("✓ Read-only mode (write operations disabled)");
    }

    let handler = Arc::new(PostgresHandler::new(database_url, allow_write_ops));

    let state = Arc::new(AppState { handler });

    let app = Router::new()
        .route("/", post(handle_jsonrpc))
        .route("/health", axum::routing::get(health_check))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("Starting PostgreSQL MCP Server on http://{}", addr);
    info!("Protocol: JSON-RPC 2.0 over HTTP");
    info!("Health check: http://{}/health", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}
