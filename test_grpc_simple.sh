#!/bin/bash
# Simplified gRPC test - just check if we can connect

set -e

TOKEN_ID="${MODAL_TOKEN_ID}"
TOKEN_SECRET="${MODAL_TOKEN_SECRET}"

if [ -z "$TOKEN_ID" ] || [ -z "$TOKEN_SECRET" ]; then
    echo "Error: MODAL_TOKEN_ID and MODAL_TOKEN_SECRET must be set"
    exit 1
fi

echo "=== Simple gRPC Connection Test ==="
echo ""

# Test 1: Just try to connect and see what error we get
echo "Test 1: Attempting gRPC call with verbose output..."
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $TOKEN_ID" \
    -H "x-modal-token-secret: $TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -H "x-modal-libmodal-version: modal-rust/0.1.0" \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello 2>&1

echo ""
echo "=== Key things to check: ==="
echo "1. Does it say 'frame with invalid size'? (same as Rust error)"
echo "2. Does it connect but fail on the RPC call?"
echo "3. Does it fail at TLS/HTTP2 negotiation?"
echo "4. What's the exact error message?"

