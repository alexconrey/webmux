# WebMux

A high-performance web-based serial port multiplexer written in Rust for managing multiple serial connections. Built with Axum and xterm.js, WebMux provides a browser-based terminal interface with RESTful API endpoints and WebSocket support for real-time serial communication with IoT devices, embedded systems, and industrial equipment.

## Features

- **Multiple Serial Connections**: Manage any number of serial port connections simultaneously
- **RESTful API**: Easy-to-use HTTP endpoints for serial communication
- **WebSocket Support**: Real-time bidirectional streaming of serial data
- **Flexible Configuration**: YAML-based configuration for all connection parameters
- **Optional Logging**: Per-connection logging with timestamps and hex/ASCII output
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Configurable Serial Parameters**: Supports various baud rates, data bits, stop bits, parity, and flow control settings

## Quick Start

### Option 1: Docker (Recommended - Works on All Platforms)

The fastest way to test with mock devices:

```bash
# Start everything with one command
docker-compose up --build

# Server runs at http://localhost:8080
# Includes 3 mock devices (IoT sensor, MCU, PLC)
```

See [DOCKER.md](DOCKER.md) for complete Docker documentation.

### Option 2: Native Installation

**Prerequisites:**
- Rust 1.70 or later
- Cargo (comes with Rust)

**Build:**
```bash
git clone <repository-url>
cd webmux
cargo build --release
```

The compiled binary will be available at `target/release/webmux`.

## Configuration

Create a `config.yaml` file in your project directory. See [config.example.yaml](config.example.yaml) for a complete example.

### Basic Configuration Structure

```yaml
# Web server settings
server:
  host: "127.0.0.1"
  port: 8080

# Serial connection definitions
serial_connections:
  - name: "device_01"
    port: "/dev/ttyUSB0"      # Linux/macOS
    # port: "COM3"            # Windows
    baud_rate: 115200
    data_bits: 8              # 5, 6, 7, or 8
    stop_bits: 1              # 1 or 2
    parity: "none"            # none, odd, or even
    flow_control: "none"      # none, software, or hardware
    enabled: true
    logging:
      enabled: false
      path: "./logs/device_01.log"
    description: "My IoT Device"
```

### Serial Port Configuration Options

| Parameter | Description | Valid Values |
|-----------|-------------|--------------|
| `name` | Unique identifier for the connection | Any string |
| `port` | Serial port path | `/dev/ttyUSB0`, `COM3`, etc. |
| `baud_rate` | Communication speed | 9600, 19200, 38400, 57600, 115200, etc. |
| `data_bits` | Number of data bits | 5, 6, 7, 8 |
| `stop_bits` | Number of stop bits | 1, 2 |
| `parity` | Parity checking | `none`, `odd`, `even` |
| `flow_control` | Flow control method | `none`, `software`, `hardware` |
| `enabled` | Enable/disable connection | `true`, `false` |
| `logging.enabled` | Enable logging for this connection | `true`, `false` |
| `logging.path` | Path to log file | Any valid file path |
| `description` | Human-readable description | Any string |

## Running the Server

### With Default Config

```bash
cargo run --release
```

This looks for `config.yaml` in the current directory.

### With Custom Config Path

```bash
cargo run --release -- /path/to/your/config.yaml
```

### Environment Variables

Set the log level using the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run --release
RUST_LOG=info cargo run --release  # Default
RUST_LOG=warn cargo run --release
```

## Testing Without Physical Devices

Don't have serial hardware? No problem! Use the built-in mock device simulator to test the web interface:

```bash
# 1. Set up virtual serial ports
./scripts/setup-virtual-ports.sh

# 2. Start mock devices (in background)
./scripts/start-mock-devices.sh

# 3. Run the server with virtual device config
cargo run --release -- config.virtual.yaml

# 4. Test the API
curl http://localhost:8080/api/connections
curl -X POST http://localhost:8080/api/connections/iot_sensor/send \
  -H "Content-Type: application/json" \
  -d '{"data": "STATUS", "format": "text"}'

# 5. Clean up when done
./scripts/stop-mock-devices.sh
```

See [MOCK_DEVICES.md](MOCK_DEVICES.md) for complete documentation on mock devices, including:
- Three simulated device types (IoT sensor, MCU, PLC)
- Supported commands for each device
- WebSocket testing
- Troubleshooting guide

## API Reference

### Health Check

Check if the server is running.

```http
GET /health
```

**Response:** `200 OK` with body `"OK"`

---

### List All Connections

Get a list of all configured serial connections.

```http
GET /api/connections
```

**Response:**
```json
[
  {
    "name": "device_01"
  },
  {
    "name": "device_02"
  }
]
```

---

### Get Connection Info

Check if a specific connection exists.

```http
GET /api/connections/:name
```

**Response:**
```json
{
  "name": "device_01",
  "exists": true
}
```

---

### Send Data to Connection

Send data to a serial port.

```http
POST /api/connections/:name/send
Content-Type: application/json
```

**Request Body:**
```json
{
  "data": "Hello, Device!",
  "format": "text"
}
```

**Formats:**
- `text` - Plain text (default)
- `hex` - Hexadecimal string (e.g., "48656c6c6f" or "48 65 6c 6c 6f")
- `base64` - Base64 encoded data

**Response:** `200 OK` with body `"Data sent"`

**Example with curl:**
```bash
# Send text
curl -X POST http://localhost:8080/api/connections/device_01/send \
  -H "Content-Type: application/json" \
  -d '{"data": "Hello!", "format": "text"}'

# Send hex data
curl -X POST http://localhost:8080/api/connections/device_01/send \
  -H "Content-Type: application/json" \
  -d '{"data": "48656c6c6f", "format": "hex"}'
```

---

### Get Connection Statistics

Get statistics about a connection.

```http
GET /api/connections/:name/stats
```

**Response:**
```json
{
  "name": "device_01",
  "port": "/dev/ttyUSB0",
  "bytes_received": 1024,
  "bytes_sent": 512,
  "is_connected": true,
  "uptime_seconds": 3600
}
```

---

### WebSocket Stream

Establish a WebSocket connection for real-time bidirectional communication.

```
WS /api/connections/:name/ws
```

**Behavior:**
- Receives data from the serial port as binary WebSocket messages
- Can send data to the serial port by transmitting binary or text WebSocket messages
- Automatically closes when the serial connection is lost

**JavaScript Example:**
```javascript
const ws = new WebSocket('ws://localhost:8080/api/connections/device_01/ws');

ws.onopen = () => {
  console.log('Connected');
  // Send text data
  ws.send('Hello from browser!');
  // Or send binary data
  const data = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f]);
  ws.send(data);
};

ws.onmessage = (event) => {
  // Received data from serial port
  if (event.data instanceof Blob) {
    event.data.arrayBuffer().then(buffer => {
      const bytes = new Uint8Array(buffer);
      console.log('Received:', bytes);
    });
  }
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

**Python Example:**
```python
import asyncio
import websockets

async def serial_client():
    uri = "ws://localhost:8080/api/connections/device_01/ws"
    async with websockets.connect(uri) as websocket:
        # Send data
        await websocket.send(b"Hello from Python!")

        # Receive data
        while True:
            data = await websocket.recv()
            print(f"Received: {data.hex()}")

asyncio.run(serial_client())
```

## Logging

When logging is enabled for a connection, all received and transmitted data is written to the specified log file with the following format:

```
[2025-11-30 15:30:45.123] device_01 | RX | 12 bytes | HEX: 48 65 6c 6c 6f 20 57 6f 72 6c 64 0a | ASCII: Hello World.
[2025-11-30 15:30:46.456] device_01 | TX | 5 bytes | HEX: 48 65 6c 6c 6f | ASCII: Hello
```

- **Timestamp**: Millisecond precision
- **Connection Name**: Identifier from config
- **Direction**: RX (received) or TX (transmitted)
- **Byte Count**: Number of bytes
- **HEX**: Hexadecimal representation
- **ASCII**: ASCII representation (non-printable chars shown as '.')

## Project Structure

```
webmux/
   src/
      main.rs              # Application entry point
      config/              # Configuration parsing
         mod.rs
      serial/              # Serial port management
         mod.rs
         connection.rs
      web/                 # Web server and API
         mod.rs
         handlers.rs
      logging/             # Serial data logging
          mod.rs
   Cargo.toml               # Dependencies
   config.example.yaml      # Example configuration
   PLAN.md                  # Project plan
   README.md                # This file
```

## Use Cases

### IoT Devices
Monitor and control IoT sensors and actuators over serial connections with real-time web access.

### Embedded Systems
Debug and communicate with microcontrollers, Arduino boards, and other embedded devices through a web interface.

### Industrial Equipment
Interface with PLCs, industrial controllers, and other serial-based equipment in manufacturing environments.

### Remote Serial Access
Provide network access to serial devices, enabling remote monitoring and control over HTTP/WebSocket.

## Security Considerations

- **No Authentication**: This server does not include authentication. Deploy behind a reverse proxy with authentication if exposing to untrusted networks.
- **CORS Enabled**: CORS is permissive by default. Adjust in [src/web/mod.rs](src/web/mod.rs:35) for production use.
- **Local Binding**: Default config binds to `127.0.0.1`. Change to `0.0.0.0` only if you need external access.

## Troubleshooting

### Permission Denied on Serial Port (Linux)

Add your user to the `dialout` group:
```bash
sudo usermod -a -G dialout $USER
# Log out and log back in for changes to take effect
```

### Port Already in Use

If the web server port is already in use, change the `server.port` value in your config file.

### Serial Port Not Found

- **Linux**: Check `/dev/ttyUSB*` or `/dev/ttyACM*`
- **macOS**: Check `/dev/tty.usb*` or `/dev/cu.usb*`
- **Windows**: Check Device Manager for COM port numbers

List available serial ports:
```bash
# Linux/macOS
ls /dev/tty*

# Or use a tool like
cargo install serialport-rs
serialport-rs list
```

## Development

### Run in Development Mode

```bash
cargo run
```

### Run Tests

The project includes comprehensive tests (42 tests total):

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture
```

See [TESTING.md](TESTING.md) for detailed testing documentation.

### Build for Release

```bash
cargo build --release
```

## Dependencies

Major dependencies include:

- **axum** - Web framework
- **tokio** - Async runtime
- **serialport** / **tokio-serial** - Serial port communication
- **serde** / **serde_yaml** - Configuration parsing
- **tracing** - Logging and diagnostics

See [Cargo.toml](Cargo.toml) for the complete list.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Roadmap

Future enhancements may include:

- [ ] Authentication and authorization
- [ ] TLS/SSL support
- [ ] Dynamic connection management (add/remove without restart)
- [ ] Serial port discovery API
- [ ] Rate limiting
- [ ] Connection health monitoring and auto-reconnect
- [ ] Metrics and monitoring endpoints

## Support

For issues, questions, or suggestions, please open an issue on the project repository.
