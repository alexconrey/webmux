use crate::serial::SerialManager;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::{cors::CorsLayer, services::ServeDir};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod handlers;
pub use handlers::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::list_connections,
        handlers::get_connection_info,
        handlers::send_data,
        handlers::get_stats,
    ),
    components(
        schemas(
            handlers::ConnectionListItem,
            handlers::ConnectionInfo,
            handlers::SendDataRequest,
            handlers::DataFormat,
            crate::serial::ConnectionStats,
        )
    ),
    tags(
        (name = "connections", description = "Serial connection management endpoints"),
        (name = "data", description = "Data transmission endpoints"),
        (name = "statistics", description = "Connection statistics endpoints"),
    ),
    info(
        title = "WebMux API",
        version = "0.1.0",
        description = "A web-based serial port multiplexer API for managing multiple serial connections",
        contact(
            name = "WebMux",
            url = "https://github.com/yourusername/webmux"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    )
)]
pub struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub serial_manager: SerialManager,
}

pub fn create_router(serial_manager: SerialManager) -> Router {
    let state = AppState { serial_manager };

    Router::new()
        // Serve frontend at root
        .route("/", get(serve_index))
        // Health check
        .route("/health", get(health_check))
        // List all connections
        .route("/api/connections", get(list_connections))
        // Get connection info
        .route("/api/connections/:name", get(get_connection_info))
        // Send data to a connection
        .route("/api/connections/:name/send", post(send_data))
        // Get connection stats
        .route("/api/connections/:name/stats", get(get_stats))
        // WebSocket for streaming data
        .route("/api/connections/:name/ws", get(websocket_handler))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Serve static files
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => (StatusCode::OK, [("Content-Type", "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Frontend not found").into_response(),
    }
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError {
            error: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod handler_tests;
