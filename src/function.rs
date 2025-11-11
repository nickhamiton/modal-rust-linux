use crate::client::Client;
use crate::error::Result;
use serde::{de::DeserializeOwned, Serialize};

/// A Modal Function represents a serverless function deployed on Modal
///
/// Functions can be invoked remotely and will execute in Modal's cloud infrastructure.
#[derive(Clone)]
pub struct Function {
    pub(crate) client: Client,
    pub(crate) function_id: String,
    pub(crate) name: Option<String>,
}

impl Function {
    /// Create a Function from a function ID
    pub fn from_id(function_id: String, client: Client) -> Self {
        Self {
            client,
            function_id,
            name: None,
        }
    }

    /// Look up a function by app name and function name
    pub async fn from_name(
        app_name: &str,
        function_name: &str,
        client: Option<Client>,
    ) -> Result<Self> {
        let mut client = if let Some(c) = client {
            c
        } else {
            Client::from_env().await?
        };

        let function_id = client.function_get(app_name, function_name).await?;

        Ok(Self {
            client,
            function_id,
            name: Some(function_name.to_string()),
        })
    }

    /// Call the function remotely with serializable arguments
    ///
    /// This is a high-level convenience method that handles serialization automatically.
    pub async fn call<T: Serialize, R: DeserializeOwned>(&mut self, args: &T) -> Result<R> {
        self.client.call_function(&self.function_id, args).await
    }

    /// Call the function remotely with pre-serialized CBOR bytes
    pub async fn call_raw(&mut self, args_cbor: Vec<u8>) -> Result<Vec<u8>> {
        self.client.call_function_sync(&self.function_id, args_cbor).await
    }

    /// Get the function ID
    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    /// Get the function name (if available)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

