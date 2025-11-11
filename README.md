# Modal Rust SDK

A comprehensive Rust client library for [Modal.com](https://modal.com) that allows you to deploy and invoke serverless functions from Rust.

## Features

- ✅ **Function Invocation**: Call deployed Modal functions from Rust
- ✅ **App Management**: Look up and manage Modal apps
- ✅ **Authentication**: Automatic credential management via environment variables or config file
- ✅ **Serialization**: Automatic CBOR serialization/deserialization
- ✅ **Type Safety**: Full type safety with Rust's type system

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
modal = { path = "." }  # or use git URL when published
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

## Quick Start

### 1. Set up Authentication

Create a `~/.modal.toml` file (or `%USERPROFILE%\.modal.toml` on Windows):

```toml
[default]
token_id = "ak-..."
token_secret = "as-..."
active = true
```

Or set environment variables:
```bash
export MODAL_TOKEN_ID="ak-..."
export MODAL_TOKEN_SECRET="as-..."
```

### 2. Call a Deployed Function

```rust
use modal::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EchoArgs {
    msg: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to Modal
    let mut client = Client::from_env().await?;
    
    // Look up a deployed function
    let function_id = client.function_get("my-app", "echo").await?;
    
    // Call the function
    let args = EchoArgs { msg: "hello".to_string() };
    let result: EchoArgs = client.call_function(&function_id, &args).await?;
    
    println!("Result: {}", result.msg);
    Ok(())
}
```

### 3. Using the Function Type

```rust
use modal::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::from_env().await?;
    
    // Look up function by name
    let mut function = Function::from_name("my-app", "echo", Some(client)).await?;
    
    // Call it
    let args = EchoArgs { msg: "hello".to_string() };
    let result: EchoArgs = function.call(&args).await?;
    
    Ok(())
}
```

## Architecture

This SDK is based on the official Modal Python client and implements:

- **gRPC Communication**: Uses Tonic for gRPC communication with Modal's API
- **Protocol Buffers**: Generated from Modal's proto definitions
- **CBOR Serialization**: Matches the Python client's serialization format
- **Authentication**: Token-based authentication via headers

## Project Structure

```
src/
├── lib.rs          # Main library entry point
├── client.rs       # Core client for API communication
├── app.rs          # App management
├── function.rs     # Function invocation
├── proto.rs        # Generated protobuf types
├── serialization.rs # CBOR serialization helpers
└── error.rs        # Error types
```

## Development Status

This SDK is currently focused on **function invocation** (calling deployed functions). 

Full deployment support (defining and deploying functions from Rust) is planned but requires additional work to:
- Implement function definition and registration
- Handle image building and mounting
- Support the full deployment workflow

For now, you can:
1. Deploy functions using the Python SDK or Modal CLI
2. Call those deployed functions from Rust using this SDK

## Examples

See the `examples/` directory for more examples.

## License

MIT

