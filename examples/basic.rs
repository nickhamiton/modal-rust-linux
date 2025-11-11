//! Basic example of using the Modal Rust SDK
//!
//! This example demonstrates:
//! - Connecting to Modal
//! - Looking up a deployed function
//! - Calling the function with serializable arguments

use modal::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct EchoArgs {
    msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EchoResult {
    echo: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Connecting to Modal...");
    
    // Connect to Modal using credentials from environment or ~/.modal.toml
    let mut client = Client::from_env().await?;
    println!("Connected!");

    // Look up a deployed function
    // Replace "my-app" and "echo" with your actual app and function names
    println!("Looking up function...");
    let function_id = match client.function_get("echo-app", "echo").await {
        Ok(id) => id,
        Err(Error::FunctionNotFound(_)) => {
            println!("Function not found. Make sure you have a function deployed.");
            println!("Example: Deploy a Python function with Modal and then call it from Rust.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    
    println!("Found function: {}", function_id);

    // Call the function
    println!("Calling function...");
    let args = EchoArgs {
        msg: "Hello from Rust!".to_string(),
    };

    // Option 1: Use the high-level call_function method
    match client.call_function::<_, EchoResult>(&function_id, &args).await {
        Ok(result) => {
            println!("Function returned: {:?}", result);
        }
        Err(e) => {
            println!("Error calling function: {}", e);
        }
    }

    // Option 2: Use the Function type for a more ergonomic API
    let mut function = Function::from_id(function_id, client);
    match function.call::<_, EchoResult>(&args).await {
        Ok(result) => {
            println!("Function returned via Function type: {:?}", result);
        }
        Err(e) => {
            println!("Error calling function: {}", e);
        }
    }

    Ok(())
}

