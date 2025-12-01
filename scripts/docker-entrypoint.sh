#!/bin/bash
set -e

echo "=== Terminal Access Server - Docker Container ==="
echo ""

# Create virtual serial ports
echo "Setting up virtual serial ports..."
./scripts/setup-virtual-ports.sh
echo ""

# Start mock devices in background
echo "Starting mock devices..."
mock_device /tmp/ttyVIOT1 iot > /app/logs/mock-iot.log 2>&1 &
echo "  ✓ IoT Sensor (PID: $!)"

mock_device /tmp/ttyVMCU1 mcu > /app/logs/mock-mcu.log 2>&1 &
echo "  ✓ Embedded MCU (PID: $!)"

mock_device /tmp/ttyVPLC1 plc > /app/logs/mock-plc.log 2>&1 &
echo "  ✓ Industrial PLC (PID: $!)"

echo ""
echo "Waiting for mock devices to initialize..."
sleep 2

# Start the server
echo "Starting Terminal Access Server..."
echo "========================================="
echo ""

exec terminal-access-server "$@"
