//! Modal Rust SDK
//!
//! A comprehensive Rust client library for Modal.com that allows you to:
//! - Deploy and manage Modal apps
//! - Define and deploy serverless functions
//! - Invoke deployed functions remotely
//! - Manage authentication and connections
//!
//! # Quick Start
//!
//! ```no_run
//! use modal::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct EchoArgs {
//!     msg: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Connect to Modal
//!     let mut client = Client::from_env().await?;
//!
//!     // Look up a deployed function
//!     let function_id = client.function_get("my-app", "echo").await?;
//!
//!     // Call the function
//!     let args = EchoArgs { msg: "hello".to_string() };
//!     let result: EchoArgs = client.call_function(&function_id, &args).await?;
//!     println!("Echo response: {}", result.msg);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod app;
pub mod function;
pub mod proto;
pub mod serialization;
pub mod error;

pub use client::Client;
pub use app::App;
pub use function::Function;
pub use error::{Error, Result};

