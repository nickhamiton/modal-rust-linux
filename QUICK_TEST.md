# Quick Test Instructions

Since grpcurl works (no frame size error), the issue is Tonic-specific. Let's confirm with a real RPC call:

## On your VPS, run:

```bash
# Make sure you have the proto file
# Copy it from your local machine or use the one in libmodal directory

# If you have the libmodal directory:
chmod +x test_with_proto.sh
./test_with_proto.sh

# Or manually test:
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $MODAL_TOKEN_ID" \
    -H "x-modal-token-secret: $MODAL_TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    -proto libmodal/modal-client/modal_proto/api.proto \
    -import-path libmodal/modal-client/modal_proto \
    -d '{}' \
    api.modal.com:443 modal.client.ModalClient/ClientHello
```

## What this tells us:

- **If ClientHello works**: Connection is fine, issue is 100% Tonic-specific
- **If we see frame errors**: Then it's something else (unlikely based on current tests)

## Next steps:

Once we confirm grpcurl works with proto, we know:
1. HTTP/2 connection works fine
2. The issue is in Tonic's HTTP/2 implementation
3. We need to find Tonic-specific configuration or workaround

