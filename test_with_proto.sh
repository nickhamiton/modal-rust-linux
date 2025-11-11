#!/bin/bash
# Test gRPC with proto file from the libmodal repo

TOKEN_ID="${MODAL_TOKEN_ID}"
TOKEN_SECRET="${MODAL_TOKEN_SECRET}"

if [ -z "$TOKEN_ID" ] || [ -z "$TOKEN_SECRET" ]; then
    echo "Error: MODAL_TOKEN_ID and MODAL_TOKEN_SECRET must be set"
    exit 1
fi

# Check if proto file exists (try multiple possible locations)
PROTO_FILE=""
for path in "libmodal/modal-client/modal_proto/api.proto" "modal-client-main/modal_proto/api.proto" "libmodal/modal-go/proto/modal_proto/api.proto"; do
    if [ -f "$path" ]; then
        PROTO_FILE="$path"
        PROTO_DIR=$(dirname "$path")
        break
    fi
done

if [ -z "$PROTO_FILE" ]; then
    echo "Proto file not found. Tried:"
    echo "  - libmodal/modal-client/modal_proto/api.proto"
    echo "  - modal-client-main/modal_proto/api.proto"
    echo "  - libmodal/modal-go/proto/modal_proto/api.proto"
    exit 1
fi

echo "Using proto file: $PROTO_FILE"
echo "Proto directory: $PROTO_DIR"
echo ""

echo "=== Testing gRPC with Proto File ==="
echo ""

# Test 1: ClientHello (no auth needed)
echo "=== Test 1: ClientHello (no auth) ==="
grpcurl -v -plaintext=false \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -proto "$PROTO_FILE" \
    -import-path "$PROTO_DIR" \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello 2>&1 | head -50

echo ""
echo "=== Test 2: ClientHello with auth headers ==="
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $TOKEN_ID" \
    -H "x-modal-token-secret: $TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -H "x-modal-libmodal-version: modal-rust/0.1.0" \
    -proto "$PROTO_FILE" \
    -import-path "$PROTO_DIR" \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello 2>&1 | head -50

echo ""
echo "=== Test 3: FunctionGet (requires real app/function) ==="
echo "This will likely fail with 'not found' but should show if connection works..."
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $TOKEN_ID" \
    -H "x-modal-token-secret: $TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -H "x-modal-libmodal-version: modal-rust/0.1.0" \
    -proto "$PROTO_FILE" \
    -import-path "$PROTO_DIR" \
    -d '{"app_name": "test-app", "object_tag": "test-function", "environment_name": ""}' \
    api.modal.com:443 modal.client.ModalClient/FunctionGet 2>&1 | head -50

echo ""
echo "=== Key Observations ==="
echo "1. If ClientHello works → Connection is fine, issue is in our Rust code"
echo "2. If we see 'frame with invalid size' → Same issue as Rust (unlikely now)"
echo "3. If we see 'not found' or 'unauthorized' → Connection works, just wrong params"

