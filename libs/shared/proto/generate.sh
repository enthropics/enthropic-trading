#!/bin/bash
set -e

PROTO_DIR="$(dirname "$0")/../../proto"
OUT_DIR="$(dirname "$0")"

echo "Generating proto files from $PROTO_DIR"

# Generate TypeScript (using protobufjs)
# npx pbjs -t static-module -w commonjs -o "$OUT_DIR/ts/index.js" "$PROTO_DIR"/*.proto
# npx pbts -o "$OUT_DIR/ts/index.d.ts" "$OUT_DIR/ts/index.js"

# Generate Python
# python -m grpc_tools.protoc -I="$PROTO_DIR" --python_out="$OUT_DIR/python" "$PROTO_DIR"/*.proto

# Generate Rust (using prost-build in build.rs)

echo "Proto generation complete"
