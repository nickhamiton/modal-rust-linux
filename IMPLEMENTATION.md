# Modal Rust SDK Implementation

This document describes the Rust SDK implementation for Modal.com, based on the Python client source code.

## Overview

The Rust SDK provides a native Rust interface to Modal's serverless function platform, allowing you to:
- Connect to Modal with authentication
- Deploy and manage Modal apps
- Invoke deployed functions remotely
- Manage authentication and connections

## Architecture

The SDK is structured to mirror the Python client's architecture:

### Core Components

1. **Client** (`src/client.rs`)
   - Handles gRPC connections to Modal's API
   - Manages authentication (token ID/secret)
   - Provides metadata headers matching Python client format
   - Supports config file (`~/.modal.toml`) and environment variables
   - Implements function invocation flow

2. **App** (`src/app.rs`)
   - Represents a Modal App (group of functions)
   - Supports `AppGetOrCreate` and `AppPublish` RPCs
   - Manages app tags and metadata
   - Handles app deployment

3. **Function** (`src/function.rs`)
   - Represents a deployed Modal function
   - Supports function lookup and invocation
   - Handles serialization/deserialization

4. **Serialization** (`src/serialization.rs`)
   - Supports CBOR format (preferred)
   - Framework for Pickle format (not yet implemented)
   - Matches Python client's data format handling

5. **Proto** (`src/proto.rs`)
   - Generated Rust types from Modal's protobuf definitions
   - Compiled from `modal-client-main/modal_proto/api.proto`

## Key Features Implemented

### Authentication

The client supports multiple authentication methods, matching the Python client:

1. **Environment Variables** (highest priority):
   - `MODAL_TOKEN_ID`
   - `MODAL_TOKEN_SECRET`
   - `MODAL_SERVER_URL` (optional)

2. **Config File** (`~/.modal.toml` or `%USERPROFILE%\.modal.toml`):
   ```toml
   [default]
   token_id = "ak-..."
   token_secret = "as-..."
   server_url = "https://api.modal.com:443"
   ```

3. **Profile Support**:
   - `MODAL_PROFILE` environment variable to select profile
   - `MODAL_CONFIG_PATH` to override config file location

### Metadata Headers

The client sends metadata headers matching the Python client:
- `x-modal-client-version`: SDK version
- `x-modal-client-type`: `7` (CLIENT_TYPE_LIBMODAL for Rust)
- `x-modal-platform`: OS and architecture
- `x-modal-node`: Hostname
- `x-modal-token-id`: Authentication token ID
- `x-modal-token-secret`: Authentication token secret

### Function Invocation Flow

The SDK follows the same RPC flow as the Python client:

1. **FunctionMap**: Create a function call
   - Returns `function_call_id` and optionally `pipelined_inputs`

2. **FunctionPutInputs** (if needed): Submit function arguments
   - Used when `pipelined_inputs` is empty from FunctionMap
   - Serializes arguments as CBOR (or Pickle)

3. **FunctionGetOutputs**: Poll for results
   - Polls until function completes
   - Handles both inline data and blob storage
   - Returns serialized result

### App Management

Apps can be:
- **Looked up** by name (deployed or ephemeral)
- **Created** if missing (`lookup_or_create`)
- **Deployed** with functions and classes
- **Tagged** with metadata

## Differences from Python Client

1. **Serialization**: 
   - Rust SDK uses CBOR by default (more efficient)
   - Pickle format not yet implemented (would require pickle library)

2. **Function Definition**:
   - Python client can define functions with decorators
   - Rust SDK focuses on invoking already-deployed functions
   - Function creation/deployment is a future enhancement

3. **Async Model**:
   - Python uses `asyncio` with synchronous wrappers
   - Rust uses `tokio` with native async/await

## Usage Examples

### Basic Function Call

```rust
use modal::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Args {
    msg: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to Modal
    let mut client = Client::from_env().await?;
    
    // Look up function
    let function_id = client.function_get("my-app", "echo").await?;
    
    // Call function
    let args = Args { msg: "Hello".to_string() };
    let result: Args = client.call_function(&function_id, &args).await?;
    
    Ok(())
}
```

### Using App and Function Types

```rust
use modal::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Look up app
    let app = App::lookup("my-app", None).await?;
    
    // Look up function
    let mut function = Function::from_name("my-app", "echo", None).await?;
    
    // Call function
    let args = serde_json::json!({"msg": "Hello"});
    let result = function.call(&args).await?;
    
    Ok(())
}
```

## Build Configuration

The SDK uses `build.rs` to compile protobuf definitions:

1. Uses `protoc-bin-vendored` for protoc binary
2. Compiles `modal-client-main/modal_proto/api.proto`
3. Generates Rust types using `tonic-build`

## Future Enhancements

1. **Function Creation/Deployment**:
   - Support for defining and deploying functions from Rust
   - Image building and management
   - Mount and volume management

2. **Pickle Support**:
   - Add pickle serialization library
   - Enable Python-compatible serialization

3. **Retry Logic**:
   - Implement retry policies for transient errors
   - Match Python client's retry behavior

4. **Streaming**:
   - Support for generator functions
   - Streaming input/output

5. **Web Endpoints**:
   - Support for webhook functions
   - HTTP request/response handling

## Testing

To test the SDK:

1. Set up Modal credentials:
   ```bash
   export MODAL_TOKEN_ID="ak-..."
   export MODAL_TOKEN_SECRET="as-..."
   ```

2. Run the example:
   ```bash
   cargo run --example basic
   ```

## References

- Python Modal Client: `modal-client-main/modal/`
- Proto Definitions: `modal-client-main/modal_proto/api.proto`
- Modal API Documentation: https://modal.com/docs
