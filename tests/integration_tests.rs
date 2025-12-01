use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use webmux::config::*;
use webmux::serial::SerialManager;
use webmux::web;
use tower::ServiceExt;

async fn body_to_json(body: Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    let string = String::from_utf8(bytes.to_vec()).unwrap();
    serde_json::from_str(&string).unwrap()
}

#[tokio::test]
async fn test_server_health_endpoint() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

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
}

#[tokio::test]
async fn test_list_connections_workflow() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    // List connections (should be empty initially)
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
async fn test_connection_info_workflow() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    // Check for non-existent connection
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/test_device")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_to_json(response.into_body()).await;
    assert_eq!(json["name"], "test_device");
    assert_eq!(json["port"], "");
    assert_eq!(json["baud_rate"], 0);
}

#[tokio::test]
async fn test_send_data_text_format() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    let payload = serde_json::json!({
        "data": "Hello, Device!",
        "format": "text"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/test_device/send")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail because connection doesn't exist
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let json = body_to_json(response.into_body()).await;
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Connection not found"));
}

#[tokio::test]
async fn test_send_data_hex_format() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    let payload = serde_json::json!({
        "data": "48656c6c6f",
        "format": "hex"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/test_device/send")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_send_data_base64_format() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    let payload = serde_json::json!({
        "data": "SGVsbG8=",
        "format": "base64"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/test_device/send")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_get_stats_nonexistent_connection() {
    let serial_manager = SerialManager::new();
    let app = web::create_router(serial_manager);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/connections/test_device/stats")
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

#[test]
fn test_config_example_yaml_structure() {
    // This tests that our example config matches expected structure
    let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080

serial_connections:
  - name: "iot_sensor_01"
    port: "/dev/ttyUSB0"
    baud_rate: 115200
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: true
    logging:
      enabled: false
      path: "./logs/iot_sensor_01.log"
    description: "IoT temperature sensor"

  - name: "embedded_mcu"
    port: "/dev/ttyACM0"
    baud_rate: 9600
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: true
    logging:
      enabled: false
      path: "./logs/embedded_mcu.log"
    description: "Arduino-based control system"

  - name: "industrial_plc"
    port: "/dev/ttyS0"
    baud_rate: 19200
    data_bits: 8
    stop_bits: 2
    parity: "even"
    flow_control: "hardware"
    enabled: true
    logging:
      enabled: true
      path: "./logs/industrial_plc.log"
    description: "Industrial PLC controller"
"#;

    let config: Config = serde_yaml::from_str(yaml).unwrap();

    // Validate server config
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);

    // Validate we have 3 connections matching config.example.yaml
    assert_eq!(config.serial_connections.len(), 3);

    // Validate IoT sensor
    let iot = &config.serial_connections[0];
    assert_eq!(iot.name, "iot_sensor_01");
    assert_eq!(iot.port, "/dev/ttyUSB0");
    assert_eq!(iot.baud_rate, 115200);
    assert_eq!(iot.data_bits, DataBits::Eight);
    assert_eq!(iot.stop_bits, StopBits::One);
    assert_eq!(iot.parity, Parity::None);
    assert_eq!(iot.flow_control, FlowControl::None);
    assert!(iot.enabled);
    assert!(!iot.logging.enabled);

    // Validate embedded MCU
    let mcu = &config.serial_connections[1];
    assert_eq!(mcu.name, "embedded_mcu");
    assert_eq!(mcu.port, "/dev/ttyACM0");
    assert_eq!(mcu.baud_rate, 9600);
    assert!(mcu.enabled);

    // Validate industrial PLC
    let plc = &config.serial_connections[2];
    assert_eq!(plc.name, "industrial_plc");
    assert_eq!(plc.port, "/dev/ttyS0");
    assert_eq!(plc.baud_rate, 19200);
    assert_eq!(plc.stop_bits, StopBits::Two);
    assert_eq!(plc.parity, Parity::Even);
    assert_eq!(plc.flow_control, FlowControl::Hardware);
    assert!(plc.logging.enabled);

    // Validate the config
    config.validate().unwrap();
}

#[test]
fn test_serial_manager_creation() {
    let manager = SerialManager::new();
    // Manager should be created successfully
    drop(manager);
}

#[tokio::test]
async fn test_serial_manager_list_empty() {
    let manager = SerialManager::new();
    let connections = manager.list_connections().await;
    assert!(connections.is_empty());
}

#[tokio::test]
async fn test_serial_manager_get_nonexistent() {
    let manager = SerialManager::new();
    let connection = manager.get_connection("nonexistent").await;
    assert!(connection.is_none());
}

#[test]
fn test_data_format_conversions() {
    // Test text to bytes
    let text = "Hello";
    let bytes = text.as_bytes();
    assert_eq!(bytes, b"Hello");

    // Test hex to bytes
    let hex_decoded = hex::decode("48656c6c6f").unwrap();
    assert_eq!(hex_decoded, b"Hello");

    // Test base64 to bytes
    use base64::{engine::general_purpose, Engine as _};
    let base64_decoded = general_purpose::STANDARD.decode("SGVsbG8=").unwrap();
    assert_eq!(base64_decoded, b"Hello");
}

#[test]
fn test_all_serial_port_configurations() {
    // Test that all config options from example are parseable
    let configs = vec![
        (DataBits::Five, 5),
        (DataBits::Six, 6),
        (DataBits::Seven, 7),
        (DataBits::Eight, 8),
    ];

    for (data_bits, _expected) in configs {
        let _sp_data_bits: serialport::DataBits = data_bits.into();
    }

    let stop_bits_configs = vec![(StopBits::One, 1), (StopBits::Two, 2)];

    for (stop_bits, _expected) in stop_bits_configs {
        let _sp_stop_bits: serialport::StopBits = stop_bits.into();
    }

    let parity_configs = vec![Parity::None, Parity::Odd, Parity::Even];

    for parity in parity_configs {
        let _sp_parity: serialport::Parity = parity.into();
    }

    let flow_control_configs = vec![
        FlowControl::None,
        FlowControl::Software,
        FlowControl::Hardware,
    ];

    for flow_control in flow_control_configs {
        let _sp_flow: serialport::FlowControl = flow_control.into();
    }
}
