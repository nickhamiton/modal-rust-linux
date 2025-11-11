# Testing gRPC Connection to Modal API

## Prerequisites

1. **Install grpcurl on your Ubuntu VPS:**
   ```bash
   # Option 1: Using apt (if available)
   sudo apt-get update
   sudo apt-get install -y grpcurl
   
   # Option 2: Download binary
   wget https://github.com/fullstorydev/grpcurl/releases/download/v1.8.9/grpcurl_1.8.9_linux_x86_64.tar.gz
   tar -xzf grpcurl_1.8.9_linux_x86_64.tar.gz
   sudo mv grpcurl /usr/local/bin/
   ```

2. **Set your Modal credentials:**
   ```bash
   export MODAL_TOKEN_ID="your-token-id"
   export MODAL_TOKEN_SECRET="your-token-secret"
   ```

## Run the test script

```bash
./test_grpc.sh
```

## Manual testing commands

If you want to test manually:

### 1. Test basic connectivity
```bash
grpcurl -plaintext=false \
    -H "x-modal-token-id: $MODAL_TOKEN_ID" \
    -H "x-modal-token-secret: $MODAL_TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    api.modal.com:443 list
```

### 2. Test with verbose output
```bash
grpcurl -v -plaintext=false \
    -H "x-modal-token-id: $MODAL_TOKEN_ID" \
    -H "x-modal-token-secret: $MODAL_TOKEN_SECRET" \
    -H "x-modal-client-type: 7" \
    -H "x-modal-client-version: 1.0.0" \
    api.modal.com:443 list
```

### 3. Check HTTP/2 support
```bash
curl -v --http2 https://api.modal.com:443 2>&1 | grep -i "http/2\|alpn"
```

## What to look for

1. **Connection success**: If `grpcurl list` works, the connection is fine
2. **Error messages**: Note any specific error messages about:
   - Frame size
   - HTTP/2 protocol
   - TLS/SSL issues
   - Authentication errors

3. **HTTP/2 negotiation**: Check if ALPN negotiation works correctly

## Expected results

- If grpcurl works: The issue is likely Tonic-specific configuration
- If grpcurl fails with same error: The issue is with our headers/configuration
- If grpcurl fails differently: We can see what the actual problem is

