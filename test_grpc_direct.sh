#!/bin/bash
# Direct gRPC test - try to make a call and see what happens

TOKEN_ID="${MODAL_TOKEN_ID}"
TOKEN_SECRET="${MODAL_TOKEN_SECRET}"

if [ -z "$TOKEN_ID" ] || [ -z "$TOKEN_SECRET" ]; then
    echo "Error: MODAL_TOKEN_ID and MODAL_TOKEN_SECRET must be set"
    exit 1
fi

echo "=== Direct gRPC Call Test ==="
echo ""
echo "Attempting ClientHello call (this doesn't require proto file)..."
echo ""

# Try ClientHello - it's a simple call that might work even without proto
# The key is to see if we get connection errors or the same "frame with invalid size" error
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $TOKEN_ID" \
    -H "x-modal-token-secret: $TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -H "x-modal-libmodal-version: modal-rust/0.1.0" \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello

echo ""
echo "=== What to look for: ==="
echo "1. If you see 'frame with invalid size' - same error as Rust (Tonic issue)"
echo "2. If you see 'unknown service' or 'method not found' - connection works!"
echo "3. If you see TLS/HTTP2 errors - different issue"
echo "4. If you see authentication errors - headers are wrong"

