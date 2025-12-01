#!/bin/bash

# Script to create virtual serial port pairs for testing
# This creates pairs of connected virtual serial ports that act like physical serial cables

set -e

echo "=== Virtual Serial Port Setup ==="
echo ""

# Check if socat is installed
if ! command -v socat &> /dev/null; then
    echo "Error: socat is not installed"
    echo ""
    echo "Install it with:"
    echo "  macOS:   brew install socat"
    echo "  Linux:   sudo apt-get install socat"
    echo "  or:      sudo yum install socat"
    echo ""
    exit 1
fi

echo "✓ socat is installed"
echo ""

# Function to create a virtual serial port pair
create_port_pair() {
    local name=$1
    local link1=$2
    local link2=$3

    echo "Creating virtual serial port pair: $name"
    echo "  Device 1: $link1"
    echo "  Device 2: $link2"
    echo ""

    # Kill any existing socat processes for these ports
    pkill -f "socat.*$link1" 2>/dev/null || true
    pkill -f "socat.*$link2" 2>/dev/null || true

    # Create the virtual serial port pair
    socat -d -d \
        pty,raw,echo=0,link=$link1 \
        pty,raw,echo=0,link=$link2 \
        2>&1 | grep -v "starting" &

    sleep 0.5

    if [ -e "$link1" ] && [ -e "$link2" ]; then
        echo "✓ Created successfully"

        # Set permissions
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            chmod 666 $link1 $link2 2>/dev/null || true
        fi
    else
        echo "✗ Failed to create port pair"
        return 1
    fi

    echo ""
}

# Create three virtual serial port pairs matching config.example.yaml
echo "Setting up 3 virtual serial port pairs..."
echo ""

# IoT Sensor pair
create_port_pair \
    "IoT Sensor" \
    "/tmp/ttyVIOT0" \
    "/tmp/ttyVIOT1"

# Embedded MCU pair
create_port_pair \
    "Embedded MCU" \
    "/tmp/ttyVMCU0" \
    "/tmp/ttyVMCU1"

# Industrial PLC pair
create_port_pair \
    "Industrial PLC" \
    "/tmp/ttyVPLC0" \
    "/tmp/ttyVPLC1"

echo "========================================="
echo ""
echo "Virtual serial ports created successfully!"
echo ""
echo "Configuration for config.yaml:"
echo ""
echo "serial_connections:"
echo "  - name: \"iot_sensor\""
echo "    port: \"/tmp/ttyVIOT0\""
echo "    baud_rate: 115200"
echo "    # ... other settings"
echo ""
echo "  - name: \"embedded_mcu\""
echo "    port: \"/tmp/ttyVMCU0\""
echo "    baud_rate: 9600"
echo "    # ... other settings"
echo ""
echo "  - name: \"industrial_plc\""
echo "    port: \"/tmp/ttyVPLC0\""
echo "    baud_rate: 19200"
echo "    # ... other settings"
echo ""
echo "========================================="
echo ""
echo "Now run the mock devices in separate terminals:"
echo ""
echo "  Terminal 1 (IoT Sensor):"
echo "    cargo run --bin mock_device /tmp/ttyVIOT1 iot --verbose"
echo ""
echo "  Terminal 2 (Embedded MCU):"
echo "    cargo run --bin mock_device /tmp/ttyVMCU1 mcu --verbose"
echo ""
echo "  Terminal 3 (Industrial PLC):"
echo "    cargo run --bin mock_device /tmp/ttyVPLC1 plc --verbose"
echo ""
echo "Then start the server:"
echo "    cargo run --release"
echo ""
echo "To stop virtual ports, run:"
echo "    pkill -f 'socat.*ttyV'"
echo ""

while true; do sleep 1; done