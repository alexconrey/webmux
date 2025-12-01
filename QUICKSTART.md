# Quick Start Guide

Get up and running with Terminal Access Server in 5 minutes using mock devices.

## Prerequisites

- Rust and Cargo installed
- `socat` installed (for virtual serial ports)

## 5-Minute Setup

### 1. Install socat

**macOS:**
```bash
brew install socat
```

**Linux:**
```bash
sudo apt-get install socat  # Debian/Ubuntu
# or
sudo yum install socat      # RHEL/CentOS
```

### 2. Clone and Build

```bash
cd terminal-access-server
cargo build --release
```

### 3. Start Mock Environment

```bash
# Create virtual serial ports
./scripts/setup-virtual-ports.sh

# Start 3 mock devices
./scripts/start-mock-devices.sh

# Start the server
cargo run --release -- config.virtual.yaml
```

You should see:
```
✓ Server is ready and listening on 127.0.0.1:8080
```

### 4. Test It

Open a new terminal and try these commands:

```bash
# Health check
curl http://localhost:8080/health

# List connected devices
curl http://localhost:8080/api/connections

# Get IoT sensor status
curl -X POST http://localhost:8080/api/connections/iot_sensor/send \
  -H "Content-Type: application/json" \
  -d '{"data": "STATUS", "format": "text"}'

# Get statistics
curl http://localhost:8080/api/connections/iot_sensor/stats
```

### 5. View Logs

Watch real-time serial communication:

```bash
tail -f logs/iot_sensor.log
```

You'll see:
```
[2025-11-30 15:30:45.123] iot_sensor | RX | 6 bytes | HEX: 53 54 41 54 55 53 | ASCII: STATUS
[2025-11-30 15:30:45.124] iot_sensor | TX | 10 bytes | HEX: 53 54 41 54 55 53 3a 4f 4b 0a | ASCII: STATUS:OK.
```

### 6. Try Different Devices

```bash
# Embedded MCU - Get version
curl -X POST http://localhost:8080/api/connections/embedded_mcu/send \
  -H "Content-Type: application/json" \
  -d '{"data": "VERSION", "format": "text"}'

# Industrial PLC - Get pressure
curl -X POST http://localhost:8080/api/connections/industrial_plc/send \
  -H "Content-Type: application/json" \
  -d '{"data": "PRESSURE", "format": "text"}'
```

### 7. Clean Up

When you're done:

```bash
# Press Ctrl+C to stop the server
# Then run:
./scripts/stop-mock-devices.sh
```

## What Just Happened?

You created:

1. **3 Virtual Serial Port Pairs**
   - `/tmp/ttyVIOT0` ↔ `/tmp/ttyVIOT1`
   - `/tmp/ttyVMCU0` ↔ `/tmp/ttyVMCU1`
   - `/tmp/ttyVPLC0` ↔ `/tmp/ttyVPLC1`

2. **3 Mock Devices** (running on `*1` ports)
   - IoT sensor - sends JSON telemetry
   - Embedded MCU - sends ADC readings
   - Industrial PLC - sends pressure data

3. **The Server** (connected to `*0` ports)
   - REST API on port 8080
   - WebSocket streaming
   - Automatic logging

## Next Steps

### Try WebSocket Streaming

```javascript
// In browser console or Node.js
const ws = new WebSocket('ws://localhost:8080/api/connections/iot_sensor/ws');

ws.onmessage = (event) => {
  event.data.text().then(text => console.log('Received:', text));
};

ws.onopen = () => {
  ws.send('TEMP');  // Request temperature
};
```

### Send Hex Data

```bash
# Send "STATUS" as hex
curl -X POST http://localhost:8080/api/connections/iot_sensor/send \
  -H "Content-Type: application/json" \
  -d '{"data": "535441545553", "format": "hex"}'
```

### View All Device Commands

Each mock device responds to different commands:

**IoT Sensor:** STATUS, VERSION, ID, TEMP, HUMIDITY, HELP
**Embedded MCU:** STATUS, VERSION, ID, READ, RESET, HELP
**Industrial PLC:** STATUS, VERSION, ID, PRESSURE, STOP, START, HELP

Try them all:

```bash
curl -X POST http://localhost:8080/api/connections/iot_sensor/send \
  -H "Content-Type: application/json" \
  -d '{"data": "HELP", "format": "text"}'
```

## Troubleshooting

### "socat: command not found"
Install socat (see step 1)

### "Error opening port"
Virtual ports not created. Run:
```bash
./scripts/setup-virtual-ports.sh
```

### "Connection not found"
Mock devices aren't running. Check:
```bash
pgrep -f mock_device
```

If no output, restart:
```bash
./scripts/start-mock-devices.sh
```

### Server won't start
Port 8080 might be in use. Edit `config.virtual.yaml` and change the port.

## Documentation

- [README.md](README.md) - Full documentation
- [MOCK_DEVICES.md](MOCK_DEVICES.md) - Mock device details
- [TESTING.md](TESTING.md) - Test suite documentation
- [API Reference](README.md#api-reference) - All API endpoints

## Real Hardware

Ready to use real serial devices?

1. Copy `config.example.yaml` to `config.yaml`
2. Edit the port paths (`/dev/ttyUSB0`, `COM3`, etc.)
3. Set the correct baud rate and serial parameters
4. Run: `cargo run --release`

That's it! Enjoy exploring Terminal Access Server! 🚀
