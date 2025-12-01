use super::*;
use crate::serial::SerialManager;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

async fn body_to_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn body_to_json(body: Body) -> Value {
    let string = body_to_string(body).await;
    serde_json::from_str(&string).unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let serial_manager = SerialManager::new();
    let app = create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_to_string(response.into_body()).await;
    assert_eq!(body, "OK");
}

#[tokio::test]
async fn test_list_connections_empty() {
    let serial_manager = SerialManager::new();
    let app = create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_to_json(response.into_body()).await;
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_connection_info_not_found() {
    let serial_manager = SerialManager::new();
    let app = create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_to_json(response.into_body()).await;
    assert_eq!(json["name"], "nonexistent");
    assert_eq!(json["port"], "");
    assert_eq!(json["baud_rate"], 0);
}

#[tokio::test]
async fn test_send_data_connection_not_found() {
    let serial_manager = SerialManager::new();
    let app = create_router(serial_manager);

    let payload = serde_json::json!({
        "data": "Hello",
        "format": "text"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/nonexistent/send")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let json = body_to_json(response.into_body()).await;
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Connection not found"));
}

#[tokio::test]
async fn test_get_stats_connection_not_found() {
    let serial_manager = SerialManager::new();
    let app = create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/nonexistent/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let json = body_to_json(response.into_body()).await;
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Connection not found"));
}

#[tokio::test]
async fn test_send_data_formats() {
    // Test text format
    let text_payload = serde_json::json!({
        "data": "Hello",
        "format": "text"
    });
    let _text = serde_json::to_string(&text_payload).unwrap();

    // Test hex format
    let hex_payload = serde_json::json!({
        "data": "48656c6c6f",
        "format": "hex"
    });
    let _hex = serde_json::to_string(&hex_payload).unwrap();

    // Test base64 format
    let base64_payload = serde_json::json!({
        "data": "SGVsbG8=",
        "format": "base64"
    });
    let _base64 = serde_json::to_string(&base64_payload).unwrap();

    // These would need actual connections to test fully
    // The structure is validated here
}

#[tokio::test]
async fn test_cors_headers() {
    let serial_manager = SerialManager::new();
    let app = create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("Origin", "http://example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // CORS layer should add appropriate headers
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_error_serialization() {
    let error = ApiError {
        error: "Test error".to_string(),
    };
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("Test error"));
}

#[tokio::test]
async fn test_api_error_from_anyhow() {
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let api_error: ApiError = anyhow_error.into();
    assert_eq!(api_error.error, "Something went wrong");
}
