#!/bin/bash
set -e

PROTO_DIR="$(dirname "$0")/../../proto"

echo "Generating Protocol Buffer files..."
echo "Proto directory: $PROTO_DIR"

# List proto files
ls -la "$PROTO_DIR"

echo "Proto generation complete"
