use std::io::{Read, Write};
use std::time::Duration;

/// Mock Serial Device Simulator
///
/// This program simulates a serial device for testing the Terminal Access Server.
/// It responds to commands and periodically sends simulated sensor data.

#[derive(Debug, Clone)]
enum DeviceType {
    IoTSensor,
    EmbeddedMcu,
    IndustrialPlc,
}

impl DeviceType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "iot" | "sensor" => Some(DeviceType::IoTSensor),
            "mcu" | "embedded" => Some(DeviceType::EmbeddedMcu),
            "plc" | "industrial" => Some(DeviceType::IndustrialPlc),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        match self {
            DeviceType::IoTSensor => "IoT Sensor",
            DeviceType::EmbeddedMcu => "Embedded MCU",
            DeviceType::IndustrialPlc => "Industrial PLC",
        }
    }

    fn get_telemetry(&self, count: u32) -> String {
        match self {
            DeviceType::IoTSensor => {
                let temp = 20.0 + (count as f32 * 0.1).sin() * 5.0;
                let humidity = 50.0 + (count as f32 * 0.05).cos() * 10.0;
                format!(
                    "{{\"temperature\":{:.2},\"humidity\":{:.2},\"timestamp\":{}}}\n",
                    temp, humidity, count
                )
            }
            DeviceType::EmbeddedMcu => {
                let adc = (512.0 + (count as f32 * 0.1).sin() * 200.0) as u16;
                format!("ADC:{},COUNT:{}\n", adc, count)
            }
            DeviceType::IndustrialPlc => {
                let pressure = 100.0 + (count as f32 * 0.2).sin() * 20.0;
                let status = if count % 10 < 8 { "OK" } else { "WARN" };
                format!(
                    "PRESSURE:{:.2},STATUS:{},CYCLE:{}\n",
                    pressure, status, count
                )
            }
        }
    }

    fn process_command(&self, command: &str) -> String {
        let cmd = command.trim().to_uppercase();
        match self {
            DeviceType::IoTSensor => match cmd.as_str() {
                "STATUS" | "STATUS?" => "STATUS:OK\n".to_string(),
                "VERSION" | "VERSION?" => "VERSION:1.0.0\n".to_string(),
                "ID" | "ID?" => "ID:IOT-SENSOR-001\n".to_string(),
                "HELP" | "HELP?" => {
                    "COMMANDS: STATUS, VERSION, ID, TEMP, HUMIDITY, HELP\n".to_string()
                }
                "TEMP" | "TEMP?" => "TEMP:23.45\n".to_string(),
                "HUMIDITY" | "HUMIDITY?" => "HUMIDITY:58.2\n".to_string(),
                _ => format!("ERROR:UNKNOWN_COMMAND:{}\n", cmd),
            },
            DeviceType::EmbeddedMcu => match cmd.as_str() {
                "STATUS" | "STATUS?" => "OK\n".to_string(),
                "VERSION" | "VERSION?" => "MCU v2.1.0\n".to_string(),
                "ID" | "ID?" => "ARDUINO-MEGA-2560\n".to_string(),
                "HELP" | "HELP?" => {
                    "AVAILABLE: STATUS, VERSION, ID, READ, RESET, HELP\n".to_string()
                }
                "READ" | "READ?" => "ADC0:512,ADC1:768,ADC2:256\n".to_string(),
                "RESET" => "RESETTING...\nOK\n".to_string(),
                _ => format!("ERR:{}\n", cmd),
            },
            DeviceType::IndustrialPlc => match cmd.as_str() {
                "STATUS" | "STATUS?" => "PLC:RUNNING,MODE:AUTO\n".to_string(),
                "VERSION" | "VERSION?" => "PLC-5000 v3.2.1\n".to_string(),
                "ID" | "ID?" => "PLC-5000-SN:98765\n".to_string(),
                "HELP" | "HELP?" => {
                    "COMMANDS: STATUS, VERSION, ID, PRESSURE, STOP, START, HELP\n".to_string()
                }
                "PRESSURE" | "PRESSURE?" => "PRESSURE:105.3 PSI\n".to_string(),
                "STOP" => "SYSTEM:STOPPED\n".to_string(),
                "START" => "SYSTEM:STARTED\n".to_string(),
                _ => format!("ERR:INVALID_CMD:{}\n", cmd),
            },
        }
    }
}

fn print_usage() {
    println!("Mock Serial Device Simulator");
    println!();
    println!("Usage:");
    println!("  mock_device <port> <device_type> [options]");
    println!();
    println!("Arguments:");
    println!("  <port>        Serial port path (e.g., /dev/ttyUSB0, COM3)");
    println!("  <device_type> Type of device to simulate:");
    println!("                - iot/sensor     : IoT temperature/humidity sensor");
    println!("                - mcu/embedded   : Arduino-like microcontroller");
    println!("                - plc/industrial : Industrial PLC controller");
    println!();
    println!("Options:");
    println!(
        "  --baud <rate>      Baud rate (default: 115200 for iot, 9600 for mcu, 19200 for plc)"
    );
    println!("  --telemetry <sec>  Send telemetry every N seconds (default: 5)");
    println!("  --echo             Echo received data back");
    println!("  --verbose          Print debug information");
    println!();
    println!("Examples:");
    println!("  # IoT sensor on /dev/ttyUSB0");
    println!("  mock_device /dev/ttyUSB0 iot");
    println!();
    println!("  # Embedded MCU on COM3 with telemetry every 2 seconds");
    println!("  mock_device COM3 mcu --telemetry 2");
    println!();
    println!("  # Industrial PLC with verbose output");
    println!("  mock_device /dev/ttyS0 plc --verbose");
    println!();
    println!("Commands you can send to the device:");
    println!("  STATUS, VERSION, ID, HELP (and device-specific commands)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage();
        return;
    }

    let port_name = &args[1];
    let device_type = match DeviceType::from_str(&args[2]) {
        Some(dt) => dt,
        None => {
            eprintln!("Error: Invalid device type '{}'", args[2]);
            eprintln!("Valid types: iot, sensor, mcu, embedded, plc, industrial");
            std::process::exit(1);
        }
    };

    // Parse options
    let mut baud_rate = match device_type {
        DeviceType::IoTSensor => 115200,
        DeviceType::EmbeddedMcu => 9600,
        DeviceType::IndustrialPlc => 19200,
    };
    let mut telemetry_interval = 5;
    let mut echo_mode = false;
    let mut verbose = false;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--baud" => {
                if i + 1 < args.len() {
                    baud_rate = args[i + 1].parse().unwrap_or(baud_rate);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--telemetry" => {
                if i + 1 < args.len() {
                    telemetry_interval = args[i + 1].parse().unwrap_or(telemetry_interval);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--echo" => {
                echo_mode = true;
                i += 1;
            }
            "--verbose" => {
                verbose = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    println!("=== Mock Serial Device Simulator ===");
    println!("Device Type: {}", device_type.name());
    println!("Port: {}", port_name);
    println!("Baud Rate: {}", baud_rate);
    println!("Telemetry Interval: {}s", telemetry_interval);
    println!("Echo Mode: {}", echo_mode);
    println!("Verbose: {}", verbose);
    println!();

    // Open serial port
    let mut port = match serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error opening port {}: {}", port_name, e);
            eprintln!();
            eprintln!("On Linux/macOS, you may need to create virtual serial ports:");
            eprintln!("  socat -d -d pty,raw,echo=0 pty,raw,echo=0");
            eprintln!();
            eprintln!("On macOS, you can also use:");
            eprintln!("  brew install socat");
            std::process::exit(1);
        }
    };

    println!("✓ Serial port opened successfully");
    println!("✓ Device is ready and listening...");
    println!();
    println!("Press Ctrl+C to stop");
    println!("----------------------------------------");
    println!();

    let mut buffer = [0u8; 256];
    let mut telemetry_counter = 0u32;
    let mut last_telemetry = std::time::Instant::now();
    let telemetry_duration = Duration::from_secs(telemetry_interval);

    loop {
        // Check if it's time to send telemetry
        if last_telemetry.elapsed() >= telemetry_duration {
            let data = device_type.get_telemetry(telemetry_counter);
            if verbose {
                print!("📤 TELEMETRY: {}", data);
            }
            if let Err(e) = port.write_all(data.as_bytes()) {
                eprintln!("Error sending telemetry: {}", e);
            }
            telemetry_counter += 1;
            last_telemetry = std::time::Instant::now();
        }

        // Read incoming data
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                let received = String::from_utf8_lossy(&buffer[..n]);

                if verbose {
                    println!("📥 RECEIVED ({} bytes): {:?}", n, received.trim());
                }

                if echo_mode {
                    if let Err(e) = port.write_all(&buffer[..n]) {
                        eprintln!("Error echoing data: {}", e);
                    }
                }

                // Process commands
                for line in received.lines() {
                    if !line.trim().is_empty() {
                        let response = device_type.process_command(line);

                        if verbose {
                            print!("📤 RESPONSE: {}", response);
                        } else {
                            print!("← {}", line.trim());
                            print!(" → {}", response.trim());
                            println!();
                        }

                        if let Err(e) = port.write_all(response.as_bytes()) {
                            eprintln!("Error sending response: {}", e);
                        }
                    }
                }
            }
            Ok(_) => {
                // No data, continue
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is expected, continue
            }
            Err(e) => {
                eprintln!("Error reading from port: {}", e);
                break;
            }
        }

        // Small delay to prevent CPU spinning
        std::thread::sleep(Duration::from_millis(10));
    }
}
