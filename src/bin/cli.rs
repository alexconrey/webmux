use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{SinkExt, StreamExt};
use serde_json;
use std::io::{self, Write};
use tokio::select;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Parser, Debug)]
#[command(name = "webmux-cli")]
#[command(about = "WebMux CLI - Connect to serial devices through WebMux server", long_about = None)]
struct Args {
    /// WebMux server host
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// WebMux server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Serial device/connection name
    #[arg(short, long)]
    device: String,

    /// Use TLS/WSS connection
    #[arg(short = 's', long)]
    tls: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Construct WebSocket URL
    let protocol = if args.tls { "wss" } else { "ws" };
    let ws_url = format!(
        "{}://{}:{}/api/connections/{}/ws",
        protocol, args.host, args.port, args.device
    );

    println!("Connecting to WebMux server: {}", ws_url);
    println!("Device: {}", args.device);
    println!("Press Ctrl+C to disconnect\n");

    // Connect to WebSocket
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .context("Failed to connect to WebMux server")?;

    println!("Connected! Type to send data to the device.\n");

    let (mut write, mut read) = ws_stream.split();

    // Set up terminal for raw mode
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let result: Result<()> = async {
        let mut input_buffer = String::new();

        loop {
            select! {
                // Handle incoming WebSocket messages
                Some(msg) = read.next() => {
                    match msg? {
                        Message::Text(text) => {
                            // Parse JSON response
                            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
                                if let Some(data) = response.get("data").and_then(|d| d.as_str()) {
                                    print!("{}", data);
                                    io::stdout().flush()?;
                                }
                            }
                        }
                        Message::Binary(data) => {
                            // Handle binary data
                            let text = String::from_utf8_lossy(&data);
                            print!("{}", text);
                            io::stdout().flush()?;
                        }
                        Message::Close(_) => {
                            println!("\r\nConnection closed by server");
                            break;
                        }
                        _ => {}
                    }
                }

                // Handle keyboard input
                _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {
                    if event::poll(std::time::Duration::from_millis(0))? {
                        match event::read()? {
                            Event::Key(KeyEvent { code, modifiers, .. }) => {
                                match (code, modifiers) {
                                    // Ctrl+C to exit
                                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                        println!("\r\nDisconnecting...");
                                        break;
                                    }
                                    // Enter key - send the buffered command
                                    (KeyCode::Enter, _) => {
                                        if !input_buffer.is_empty() {
                                            // Send the complete command with newline
                                            write.send(Message::Text(format!("{}\r\n", input_buffer))).await?;
                                            input_buffer.clear();
                                        } else {
                                            // Just send newline
                                            write.send(Message::Text("\r\n".to_string())).await?;
                                        }
                                        print!("\r\n");
                                        io::stdout().flush()?;
                                    }
                                    // Backspace - remove from buffer
                                    (KeyCode::Backspace, _) => {
                                        if !input_buffer.is_empty() {
                                            input_buffer.pop();
                                            print!("\x08 \x08");
                                            io::stdout().flush()?;
                                        }
                                    }
                                    // Regular character - add to buffer
                                    (KeyCode::Char(c), _) => {
                                        input_buffer.push(c);
                                        print!("{}", c);
                                        io::stdout().flush()?;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
    .await;

    // Clean up terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default_host_and_port() {
        let args = Args::try_parse_from(["webmux-cli", "--device", "test_device"]).unwrap();
        assert_eq!(args.host, "127.0.0.1");
        assert_eq!(args.port, 8080);
        assert_eq!(args.device, "test_device");
        assert_eq!(args.tls, false);
    }

    #[test]
    fn test_args_custom_host() {
        let args =
            Args::try_parse_from(["webmux-cli", "-H", "192.168.1.100", "-d", "sensor"]).unwrap();
        assert_eq!(args.host, "192.168.1.100");
        assert_eq!(args.port, 8080);
        assert_eq!(args.device, "sensor");
    }

    #[test]
    fn test_args_custom_port() {
        let args = Args::try_parse_from(["webmux-cli", "-p", "9000", "-d", "mcu"]).unwrap();
        assert_eq!(args.host, "127.0.0.1");
        assert_eq!(args.port, 9000);
        assert_eq!(args.device, "mcu");
    }

    #[test]
    fn test_args_tls_enabled() {
        let args = Args::try_parse_from(["webmux-cli", "-s", "-d", "plc"]).unwrap();
        assert_eq!(args.tls, true);
        assert_eq!(args.device, "plc");
    }

    #[test]
    fn test_args_long_form() {
        let args = Args::try_parse_from([
            "webmux-cli",
            "--host",
            "example.com",
            "--port",
            "443",
            "--device",
            "industrial_plc",
            "--tls",
        ])
        .unwrap();
        assert_eq!(args.host, "example.com");
        assert_eq!(args.port, 443);
        assert_eq!(args.device, "industrial_plc");
        assert_eq!(args.tls, true);
    }

    #[test]
    fn test_args_missing_device() {
        let result = Args::try_parse_from(["webmux-cli"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_invalid_port() {
        let result = Args::try_parse_from(["webmux-cli", "-p", "70000", "-d", "test"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_url_construction() {
        let protocol = "ws";
        let host = "127.0.0.1";
        let port = 8080;
        let device = "iot_sensor";
        let ws_url = format!("{}://{}:{}/api/connections/{}/ws", protocol, host, port, device);
        assert_eq!(ws_url, "ws://127.0.0.1:8080/api/connections/iot_sensor/ws");
    }

    #[test]
    fn test_websocket_url_construction_wss() {
        let protocol = "wss";
        let host = "example.com";
        let port = 443;
        let device = "embedded_mcu";
        let ws_url = format!("{}://{}:{}/api/connections/{}/ws", protocol, host, port, device);
        assert_eq!(
            ws_url,
            "wss://example.com:443/api/connections/embedded_mcu/ws"
        );
    }

    #[test]
    fn test_input_buffer_behavior() {
        let mut buffer = String::new();
        buffer.push('T');
        buffer.push('E');
        buffer.push('S');
        buffer.push('T');
        assert_eq!(buffer, "TEST");

        buffer.pop();
        assert_eq!(buffer, "TES");

        buffer.push('T');
        assert_eq!(buffer, "TEST");

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_command_with_newline() {
        let command = "STATUS";
        let command_with_newline = format!("{}\r\n", command);
        assert_eq!(command_with_newline, "STATUS\r\n");
    }
}
