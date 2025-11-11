use crate::error::{Error, Result};
use crate::proto::modal::client::modal_client_client::ModalClientClient;
use crate::proto::modal::client::{
    AppGetByDeploymentNameRequest, AppGetByDeploymentNameResponse, AppGetOrCreateRequest,
    AppGetOrCreateResponse, AppGetTagsRequest, AppGetTagsResponse, AppPublishRequest,
    AppPublishResponse, AppSetTagsRequest, FunctionGetOutputsRequest, FunctionGetRequest,
    FunctionInput, FunctionMapRequest, FunctionPutInputsItem, FunctionPutInputsRequest,
};
use crate::serialization::{data_format_to_proto, deserialize_result, get_preferred_data_format, serialize_args};
use anyhow::anyhow;
use reqwest::Client as HttpClient;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;

/// Main client for interacting with Modal's API
///
/// This client handles authentication, serialization, and the RPC protocol details.
#[derive(Clone)]
pub struct Client {
    stub: ModalClientClient<Channel>,
    http: HttpClient,
    max_inline: usize,
    token_id: Option<String>,
    token_secret: Option<String>,
}

impl Client {
    /// Create a client from environment variables or config file
    ///
    /// Lookup order (matching Python client behavior):
    /// 1. Environment variables: `MODAL_TOKEN_ID` and `MODAL_TOKEN_SECRET` (highest priority)
    /// 2. ~/.modal.toml (or %USERPROFILE%\.modal.toml on Windows) - pick the profile with `active = true`,
    ///    or the profile specified by `MODAL_PROFILE`, or the first profile if none are active.
    ///
    /// `MODAL_SERVER_URL` may be provided via env and will override the default `https://api.modal.com:443`.
    pub async fn from_env() -> Result<Self> {
        let server_url = std::env::var("MODAL_SERVER_URL").ok();

        // First check environment variables (highest priority)
        let env_token_id = std::env::var("MODAL_TOKEN_ID").ok();
        let env_token_secret = std::env::var("MODAL_TOKEN_SECRET").ok();
        
        // If both env vars are set, use them directly
        if env_token_id.is_some() && env_token_secret.is_some() {
            return Self::connect(
                server_url.as_deref(),
                env_token_id.as_deref(),
                env_token_secret.as_deref(),
            )
            .await;
        }

        // Otherwise, try to read from config file
        let home = std::env::var("USERPROFILE")
            .ok()
            .or_else(|| std::env::var("HOME").ok());
        
        if let Some(home_dir) = home {
            // Check for custom config path
            let config_path = std::env::var("MODAL_CONFIG_PATH")
                .ok()
                .map(|p| std::path::PathBuf::from(p))
                .unwrap_or_else(|| std::path::Path::new(&home_dir).join(".modal.toml"));
            
            if config_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&config_path) {
                    if let Ok(value) = toml::from_str::<toml::Value>(&contents) {
                        if let Some(table) = value.as_table() {
                            // Determine which profile to use
                            let profile_name = std::env::var("MODAL_PROFILE")
                                .ok()
                                .unwrap_or_else(|| "default".to_string());
                            
                            // Find active profile or specified profile
                            let mut chosen: Option<&toml::value::Table> = None;
                            for (k, v) in table.iter() {
                                if let Some(t) = v.as_table() {
                                    // Check if this is the specified profile
                                    if k == &profile_name {
                                        chosen = Some(t);
                                        break;
                                    }
                                    // Check if this is an active profile
                                    if let Some(active) = t.get("active").and_then(|a| a.as_bool()) {
                                        if active {
                                            chosen = Some(t);
                                            break;
                                        }
                                    }
                                    // Fallback to first profile
                                    if chosen.is_none() {
                                        chosen = Some(t);
                                    }
                                }
                            }

                            if let Some(profile) = chosen {
                                let token_id = profile
                                    .get("token_id")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .or(env_token_id);
                                let token_secret = profile
                                    .get("token_secret")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .or(env_token_secret);
                                
                                // Also check for server_url in config
                                let config_server_url = profile
                                    .get("server_url")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                
                                return Self::connect(
                                    config_server_url.as_deref().or(server_url.as_deref()),
                                    token_id.as_deref(),
                                    token_secret.as_deref(),
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }

        // Final fallback: use environment variables even if only one is set
        Self::connect(
            server_url.as_deref(),
            env_token_id.as_deref(),
            env_token_secret.as_deref(),
        )
        .await
    }

    /// Create a client with explicit configuration
    ///
    /// # Arguments
    /// * `server_url` - The Modal API server URL. Defaults to https://api.modal.com:443
    /// * `token_id` - The Modal token ID for authentication
    /// * `token_secret` - The Modal token secret for authentication
    pub async fn connect(
        server_url: Option<&str>,
        token_id: Option<&str>,
        token_secret: Option<&str>,
    ) -> Result<Self> {
        let server = server_url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("MODAL_SERVER_URL").ok())
            .unwrap_or_else(|| "https://api.modal.com:443".to_string());

        // Ensure the URL has a scheme
        let server = if !server.starts_with("http://") && !server.starts_with("https://") {
            format!("https://{}", server)
        } else {
            server
        };

        let endpoint = Endpoint::from_shared(server.clone())?
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));
        let channel = endpoint.connect().await?;

        let stub = ModalClientClient::new(channel);

        // Validate that we have credentials
        let final_token_id = token_id
            .map(|s| s.to_string())
            .or_else(|| std::env::var("MODAL_TOKEN_ID").ok());
        let final_token_secret = token_secret
            .map(|s| s.to_string())
            .or_else(|| std::env::var("MODAL_TOKEN_SECRET").ok());

        if final_token_id.is_none() || final_token_secret.is_none() {
            return Err(Error::Auth(
                "Token missing. Could not authenticate client. \
                Set MODAL_TOKEN_ID and MODAL_TOKEN_SECRET environment variables, \
                or configure ~/.modal.toml".to_string(),
            ));
        }

        Ok(Self {
            stub,
            http: HttpClient::new(),
            max_inline: 16 * 1024 * 1024, // 16 MiB
            token_id: final_token_id,
            token_secret: final_token_secret,
        })
    }

    /// Create a gRPC request with proper authentication headers
    /// 
    /// This matches the standard Modal client metadata format
    pub fn make_request<T>(&self, msg: T) -> Request<T> {
        let mut req = Request::new(msg);
        // Standard metadata used by other SDKs
        // CLIENT VERSION: Behaves like this Python SDK version
        req.metadata_mut().insert(
            "x-modal-client-version",
            MetadataValue::from_static("1.0.0"),
        );
        // Use CLIENT_TYPE_LIBMODAL = 7 (no Rust-specific type exists yet)
        req.metadata_mut()
            .insert("x-modal-client-type", MetadataValue::from_static("7"));
        // Add libmodal version header (matching Go/JS implementations)
        req.metadata_mut().insert(
            "x-modal-libmodal-version",
            MetadataValue::from_static("modal-go/0.1.0"),
        );

        if let Some(ref id) = self.token_id {
            if let Ok(mv) = MetadataValue::try_from(id.as_str()) {
                req.metadata_mut().insert("x-modal-token-id", mv);
            }
        }
        if let Some(ref secret) = self.token_secret {
            if let Ok(mv) = MetadataValue::try_from(secret.as_str()) {
                req.metadata_mut().insert("x-modal-token-secret", mv);
            }
        }

        req
    }

    /// Look up a deployed function by app name and object tag (function name)
    pub async fn function_get(&mut self, app_name: &str, object_tag: &str) -> Result<String> {
        let req_msg = FunctionGetRequest {
            app_name: app_name.to_string(),
            object_tag: object_tag.to_string(),
            environment_name: String::new(),
        };
        let req = self.make_request(req_msg);
        let resp = self.stub.function_get(req).await?.into_inner();
        if resp.function_id.is_empty() {
            Err(Error::FunctionNotFound(format!(
                "Function '{}' not found in app '{}'",
                object_tag, app_name
            )))
        } else {
            Ok(resp.function_id)
        }
    }

    /// Call a deployed function synchronously with serializable arguments
    ///
    /// This is a high-level convenience method that handles serialization automatically.
    pub async fn call_function<T: Serialize, R: DeserializeOwned>(
        &mut self,
        function_id: &str,
        args: &T,
    ) -> Result<R> {
        let args_cbor = serialize_args(args)?;
        let result_bytes = self.call_function_sync(function_id, args_cbor).await?;
        deserialize_result(&result_bytes)
    }

    /// Call a deployed function synchronously with pre-serialized CBOR bytes
    ///
    /// This follows the control-plane flow: FunctionMap -> FunctionPutInputs (if needed) -> poll FunctionGetOutputs.
    pub async fn call_function_sync(
        &mut self,
        function_id: &str,
        args_cbor: Vec<u8>,
    ) -> Result<Vec<u8>> {
        // Build FunctionInput. Use preferred data format and inline bytes if small enough.
        let data_format = data_format_to_proto(get_preferred_data_format());
        let function_input = FunctionInput {
            args_oneof: Some(
                crate::proto::modal::client::function_input::ArgsOneof::Args(args_cbor.clone()),
            ),
            final_input: false,
            data_format,
            method_name: None,
        };

        let item = FunctionPutInputsItem {
            idx: 0,
            input: Some(function_input),
            r2_failed: false,
            r2_throughput_bytes_s: 0,
        };

        use crate::proto::modal::client::FunctionCallInvocationType as InvokeType;
        use crate::proto::modal::client::FunctionCallType as CallType;

        let map_msg = FunctionMapRequest {
            function_id: function_id.to_string(),
            parent_input_id: String::new(),
            return_exceptions: false,
            function_call_type: CallType::Unary as i32,
            pipelined_inputs: vec![item.clone()],
            function_call_invocation_type: InvokeType::Sync as i32,
            from_spawn_map: false,
        };
        let map_req = self.make_request(map_msg);
        let map_resp = self.stub.function_map(map_req).await?.into_inner();
        let function_call_id = map_resp.function_call_id;

        // If pipelined_inputs empty, we need to call FunctionPutInputs
        if map_resp.pipelined_inputs.is_empty() {
            let put_msg = FunctionPutInputsRequest {
                function_id: function_id.to_string(),
                function_call_id: function_call_id.clone(),
                inputs: vec![item],
            };
            let put_req = self.make_request(put_msg);
            let put_resp = self.stub.function_put_inputs(put_req).await?.into_inner();
            if put_resp.inputs.is_empty() {
                return Err(Error::Other(anyhow!(
                    "FunctionPutInputs returned no inputs - input queue full?"
                )));
            }
        }

        // Poll for outputs
        let mut attempts = 0u32;
        loop {
            let get_msg = FunctionGetOutputsRequest {
                function_call_id: function_call_id.clone(),
                max_values: 1,
                timeout: 5.0,
                last_entry_id: String::from("0-0"),
                clear_on_success: true,
                requested_at: 0.0,
                input_jwts: vec![],
                start_idx: Some(0),
                end_idx: Some(0),
            };
            let get_req = self.make_request(get_msg);
            let resp = self.stub.function_get_outputs(get_req).await?.into_inner();
            if !resp.outputs.is_empty() {
                let item = &resp.outputs[0];
                if let Some(ref result) = item.result.as_ref() {
                    match result.data_oneof {
                        Some(crate::proto::modal::client::function_result::DataOneof::Data(
                            ref data,
                        )) => {
                            return Ok(data.clone());
                        }
                        Some(
                            crate::proto::modal::client::function_result::DataOneof::DataBlobId(
                                ref blob_id,
                            ),
                        ) => {
                            // Fetch blob and return its bytes
                            let blob_req =
                                self.make_request(crate::proto::modal::client::BlobGetRequest {
                                    blob_id: blob_id.clone(),
                                });
                            let blob_resp = self.stub.blob_get(blob_req).await?.into_inner();
                            let download_url = blob_resp.download_url;
                            let resp = self.http.get(&download_url).send().await?;
                            let bytes = resp.bytes().await?.to_vec();
                            return Ok(bytes);
                        }
                        _ => {}
                    }
                    // Result received but no data - check for error
                    if !result.exception.is_empty() {
                        return Err(Error::Other(anyhow!("Remote exception: {}", result.exception)));
                    } else if result.exitcode != 0 {
                        return Err(Error::Other(anyhow!("Remote exit code: {}", result.exitcode)));
                    }
                }
            }
            attempts += 1;
            if attempts > 60 {
                return Err(Error::Other(anyhow!("timeout waiting for function output")));
            }
            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Get the underlying gRPC stub for advanced usage
    pub fn stub(&self) -> &ModalClientClient<Channel> {
        &self.stub
    }

    /// Set tags on an app
    pub async fn app_set_tags(&mut self, req: AppSetTagsRequest) -> Result<()> {
        let req_msg = self.make_request(req);
        self.stub.app_set_tags(req_msg).await?;
        Ok(())
    }

    /// Get tags from an app
    pub async fn app_get_tags(&mut self, req: AppGetTagsRequest) -> Result<AppGetTagsResponse> {
        let req_msg = self.make_request(req);
        let resp = self.stub.app_get_tags(req_msg).await?.into_inner();
        Ok(resp)
    }

    /// Get app by deployment name
    pub async fn app_get_by_deployment_name(
        &mut self,
        req: AppGetByDeploymentNameRequest,
    ) -> Result<AppGetByDeploymentNameResponse> {
        let req_msg = self.make_request(req);
        let resp = self.stub.app_get_by_deployment_name(req_msg).await?.into_inner();
        Ok(resp)
    }

    /// Get or create an app
    pub async fn app_get_or_create(
        &mut self,
        req: AppGetOrCreateRequest,
    ) -> Result<AppGetOrCreateResponse> {
        let req_msg = self.make_request(req);
        let resp = self.stub.app_get_or_create(req_msg).await?.into_inner();
        Ok(resp)
    }

    /// Publish an app
    pub async fn app_publish(&mut self, req: AppPublishRequest) -> Result<AppPublishResponse> {
        let req_msg = self.make_request(req);
        let resp = self.stub.app_publish(req_msg).await?.into_inner();
        Ok(resp)
    }
}

