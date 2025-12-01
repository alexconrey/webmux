use super::handlers::*;

#[test]
fn test_data_format_default() {
    let json = r#"{"data": "test"}"#;
    let request: SendDataRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(request.format, DataFormat::Text));
}

#[test]
fn test_data_format_text() {
    let json = r#"{"data": "test", "format": "text"}"#;
    let request: SendDataRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(request.format, DataFormat::Text));
    assert_eq!(request.data, "test");
}

#[test]
fn test_data_format_hex() {
    let json = r#"{"data": "48656c6c6f", "format": "hex"}"#;
    let request: SendDataRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(request.format, DataFormat::Hex));
    assert_eq!(request.data, "48656c6c6f");
}

#[test]
fn test_data_format_base64() {
    let json = r#"{"data": "SGVsbG8=", "format": "base64"}"#;
    let request: SendDataRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(request.format, DataFormat::Base64));
    assert_eq!(request.data, "SGVsbG8=");
}

#[test]
fn test_connection_list_item_serialization() {
    let item = ConnectionListItem {
        name: "test".to_string(),
    };
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("test"));
}

#[test]
fn test_connection_info_serialization() {
    let info = ConnectionInfo {
        name: "test".to_string(),
        port: "/dev/ttyUSB0".to_string(),
        baud_rate: 115200,
        data_bits: "8".to_string(),
        stop_bits: "1".to_string(),
        parity: "None".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("ttyUSB0"));
    assert!(json.contains("115200"));
}

#[test]
fn test_hex_decode_with_spaces() {
    // Simulating what happens in the handler
    let hex_string = "48 65 6c 6c 6f";
    let cleaned = hex_string.replace(" ", "");
    let decoded = hex::decode(cleaned).unwrap();
    assert_eq!(decoded, b"Hello");
}

#[test]
fn test_hex_decode_without_spaces() {
    let hex_string = "48656c6c6f";
    let decoded = hex::decode(hex_string).unwrap();
    assert_eq!(decoded, b"Hello");
}

#[test]
fn test_base64_decode() {
    use base64::{engine::general_purpose, Engine as _};
    let base64_string = "SGVsbG8=";
    let decoded = general_purpose::STANDARD.decode(base64_string).unwrap();
    assert_eq!(decoded, b"Hello");
}

#[test]
fn test_invalid_hex() {
    let hex_string = "ZZZZ";
    let result = hex::decode(hex_string);
    assert!(result.is_err());
}

#[test]
fn test_invalid_base64() {
    use base64::{engine::general_purpose, Engine as _};
    let base64_string = "!!!invalid!!!";
    let result = general_purpose::STANDARD.decode(base64_string);
    assert!(result.is_err());
}
