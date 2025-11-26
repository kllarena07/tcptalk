#!/bin/bash

# tcptalk client setup
# Source this script to export tcptalk command to your current shell
# Usage: source ./setup_client.sh

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Path to the binary
BINARY_PATH="$SCRIPT_DIR/client/target/debug/tcptalk-client"

# Check if binary exists, if not build it
if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found. Building..."
    cd "$SCRIPT_DIR/client"
    cargo build
fi

# Export tcptalk command to PATH (client directory)
export PATH="$PATH:$SCRIPT_DIR/client"

echo "tcptalk command exported to PATH"
echo "To make this permanent, add this line to your ~/.bashrc or ~/.zshrc:"
echo "export PATH=\"\$PATH:$SCRIPT_DIR/client\""
echo ""
echo "Usage:"
echo "  tcptalk [username] [ip_address] [-p port]"
echo ""
echo "Examples:"
echo "  tcptalk alice                    # Connect to 0.0.0.0:2133"
echo "  tcptalk alice 127.0.0.1          # Connect to 127.0.0.1:2133"
echo "  tcptalk alice 192.168.1.100 -p 9090  # Connect to 192.168.1.100:9090"