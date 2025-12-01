use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_config_parsing_valid() {
    let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080

serial_connections:
  - name: "test_device"
    port: "/dev/ttyUSB0"
    baud_rate: 115200
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: true
    logging:
      enabled: false
      path: "./logs/test.log"
    description: "Test device"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = Config::from_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.serial_connections.len(), 1);
    assert_eq!(config.serial_connections[0].name, "test_device");
    assert_eq!(config.serial_connections[0].baud_rate, 115200);
    assert_eq!(config.serial_connections[0].data_bits, DataBits::Eight);
    assert_eq!(config.serial_connections[0].stop_bits, StopBits::One);
    assert_eq!(config.serial_connections[0].parity, Parity::None);
    assert_eq!(config.serial_connections[0].flow_control, FlowControl::None);
    assert_eq!(config.serial_connections[0].enabled, true);
}

#[test]
fn test_config_multiple_connections() {
    let yaml = r#"
server:
  host: "0.0.0.0"
  port: 3000

serial_connections:
  - name: "iot_sensor"
    port: "/dev/ttyUSB0"
    baud_rate: 115200
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: true
    logging:
      enabled: false
      path: "./logs/iot.log"

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
      path: "./logs/mcu.log"

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
      path: "./logs/plc.log"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = Config::from_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.serial_connections.len(), 3);

    // Check IoT sensor
    assert_eq!(config.serial_connections[0].name, "iot_sensor");
    assert_eq!(config.serial_connections[0].baud_rate, 115200);

    // Check embedded MCU
    assert_eq!(config.serial_connections[1].name, "embedded_mcu");
    assert_eq!(config.serial_connections[1].baud_rate, 9600);

    // Check industrial PLC
    assert_eq!(config.serial_connections[2].name, "industrial_plc");
    assert_eq!(config.serial_connections[2].baud_rate, 19200);
    assert_eq!(config.serial_connections[2].stop_bits, StopBits::Two);
    assert_eq!(config.serial_connections[2].parity, Parity::Even);
    assert_eq!(
        config.serial_connections[2].flow_control,
        FlowControl::Hardware
    );
    assert!(config.serial_connections[2].logging.enabled);
}

#[test]
fn test_config_validation_duplicate_names() {
    let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080

serial_connections:
  - name: "duplicate"
    port: "/dev/ttyUSB0"
    baud_rate: 115200
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: true
    logging:
      enabled: false
      path: "./logs/test1.log"

  - name: "duplicate"
    port: "/dev/ttyUSB1"
    baud_rate: 9600
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: true
    logging:
      enabled: false
      path: "./logs/test2.log"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
    let result = config.validate();

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Duplicate connection name"));
}

#[test]
fn test_config_validation_invalid_port() {
    let yaml = r#"
server:
  host: "127.0.0.1"
  port: 0

serial_connections: []
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
    let result = config.validate();

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Server port must be greater than 0"));
}

#[test]
fn test_parity_variants() {
    assert_eq!(
        serialport::Parity::from(Parity::None),
        serialport::Parity::None
    );
    assert_eq!(
        serialport::Parity::from(Parity::Odd),
        serialport::Parity::Odd
    );
    assert_eq!(
        serialport::Parity::from(Parity::Even),
        serialport::Parity::Even
    );
}

#[test]
fn test_data_bits_variants() {
    assert_eq!(
        serialport::DataBits::from(DataBits::Five),
        serialport::DataBits::Five
    );
    assert_eq!(
        serialport::DataBits::from(DataBits::Six),
        serialport::DataBits::Six
    );
    assert_eq!(
        serialport::DataBits::from(DataBits::Seven),
        serialport::DataBits::Seven
    );
    assert_eq!(
        serialport::DataBits::from(DataBits::Eight),
        serialport::DataBits::Eight
    );
}

#[test]
fn test_stop_bits_variants() {
    assert_eq!(
        serialport::StopBits::from(StopBits::One),
        serialport::StopBits::One
    );
    assert_eq!(
        serialport::StopBits::from(StopBits::Two),
        serialport::StopBits::Two
    );
}

#[test]
fn test_flow_control_variants() {
    assert_eq!(
        serialport::FlowControl::from(FlowControl::None),
        serialport::FlowControl::None
    );
    assert_eq!(
        serialport::FlowControl::from(FlowControl::Software),
        serialport::FlowControl::Software
    );
    assert_eq!(
        serialport::FlowControl::from(FlowControl::Hardware),
        serialport::FlowControl::Hardware
    );
}

#[test]
fn test_config_disabled_connection() {
    let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080

serial_connections:
  - name: "disabled_device"
    port: "/dev/ttyUSB0"
    baud_rate: 115200
    data_bits: 8
    stop_bits: 1
    parity: "none"
    flow_control: "none"
    enabled: false
    logging:
      enabled: false
      path: "./logs/test.log"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();
    file.flush().unwrap();

    let config = Config::from_file(file.path().to_str().unwrap()).unwrap();

    assert_eq!(config.serial_connections.len(), 1);
    assert_eq!(config.serial_connections[0].enabled, false);
}
