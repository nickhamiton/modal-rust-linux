#!/bin/bash
# Test script to debug gRPC connection to Modal API

# Don't exit on error - we want to see all test results
set +e

echo "=== Testing gRPC connection to Modal API ==="
echo ""

# Check if grpcurl is installed
if ! command -v grpcurl &> /dev/null; then
    echo "Installing grpcurl..."
    # For Ubuntu/Debian
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y grpcurl
    # For other systems, try downloading binary
    else
        echo "Please install grpcurl manually: https://github.com/fullstorydev/grpcurl"
        exit 1
    fi
fi

# Get credentials from environment
TOKEN_ID="${MODAL_TOKEN_ID}"
TOKEN_SECRET="${MODAL_TOKEN_SECRET}"

if [ -z "$TOKEN_ID" ] || [ -z "$TOKEN_SECRET" ]; then
    echo "Error: MODAL_TOKEN_ID and MODAL_TOKEN_SECRET must be set"
    exit 1
fi

echo "Testing connection to api.modal.com:443..."
echo ""

# Test 1: Check HTTP/2 connection and TLS negotiation
echo "=== Test 1: Check HTTP/2 and TLS negotiation ==="
curl -v --http2 --tlsv1.2 https://api.modal.com:443 2>&1 | grep -E "(HTTP/2|ALPN|TLS|h2)" | head -10 || echo "Connection test complete"
echo ""

# Test 2: Test gRPC connection with a simple call (ClientHello - no auth needed)
echo "=== Test 2: Test ClientHello RPC (no auth required) ==="
# ClientHello is a simple RPC that doesn't require authentication
grpcurl -plaintext=false \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -proto api.proto \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello 2>&1 || {
    echo "Note: This requires the proto file. Let's try without proto to see connection errors..."
    echo ""
}

# Test 3: Test with authentication headers - this will show us the actual error
echo "=== Test 3: Test connection with auth headers (verbose) ==="
echo "This test will show us if we get the same 'frame with invalid size' error..."
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $TOKEN_ID" \
    -H "x-modal-token-secret: $TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -H "x-modal-libmodal-version: modal-rust/0.1.0" \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello 2>&1
echo ""
echo "--- End of Test 3 output ---"

echo ""
echo "=== Test 4: Check if we can establish TLS connection ==="
openssl s_client -connect api.modal.com:443 -servername api.modal.com -alpn h2 2>&1 | grep -E "(ALPN|Protocol|Cipher)" | head -5 || echo "OpenSSL test complete"

echo ""
echo "=== Tests complete ==="

