use crate::client::Client;
use crate::error::{Error, Result};
use crate::proto::modal::client;
use anyhow::anyhow;
use std::collections::HashMap;

/// A Modal App represents a group of functions and classes deployed together
///
/// Apps serve as:
/// - A unit of deployment for functions and classes
/// - A way to manage and organize your Modal resources
/// - A namespace for function lookups
#[derive(Clone)]
pub struct App {
    pub(crate) client: Client,
    pub(crate) app_id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) tags: HashMap<String, String>,
}

impl App {
    /// Create a new App with a given name
    /// 
    /// Note: You'll need to connect a client before using the app.
    /// Use `App::with_client` or `App::lookup` for a fully initialized app.
    #[deprecated(note = "Use App::with_client() or App::lookup() instead")]
    pub fn new(_name: impl Into<String>) -> Self {
        // This creates an uninitialized app - client must be set before use
        // In practice, you'd want to use App::with_client or App::lookup
        panic!("App::new() creates an uninitialized app. Use App::with_client() or App::lookup() instead.");
    }

    /// Create an App with a client
    pub fn with_client(name: impl Into<String>, client: Client) -> Self {
        Self {
            client,
            app_id: None,
            name: Some(name.into()),
            tags: HashMap::new(),
        }
    }

    /// Look up an existing deployed App by name
    ///
    /// This will find an app that has been deployed (persistent state).
    pub async fn lookup(
        name: &str,
        client: Option<Client>,
    ) -> Result<Self> {
        let client = if let Some(c) = client {
            c
        } else {
            Client::from_env().await?
        };

        // Try AppGetByDeploymentName first (for deployed apps)
        let req = client::AppGetByDeploymentNameRequest {
            name: name.to_string(),
            environment_name: String::new(),
        };
        let mut client = client;
        let resp = client.app_get_by_deployment_name(req).await?;

        if !resp.app_id.is_empty() {
            return Ok(Self {
                client,
                app_id: Some(resp.app_id),
                name: Some(name.to_string()),
                tags: HashMap::new(),
            });
        }

        // If not found, try AppGetOrCreate (for ephemeral/running apps)
        let create_req = client::AppGetOrCreateRequest {
            app_name: name.to_string(),
            environment_name: String::new(),
            object_creation_type: client::ObjectCreationType::Unspecified as i32, // Just lookup, don't create
        };
        let create_resp = client.app_get_or_create(create_req).await?;

        if create_resp.app_id.is_empty() {
            return Err(Error::Other(anyhow!("App '{}' not found", name)));
        }

        Ok(Self {
            client,
            app_id: Some(create_resp.app_id),
            name: Some(name.to_string()),
            tags: HashMap::new(),
        })
    }

    /// Look up an App, creating it if it doesn't exist
    ///
    /// This is useful for creating an App to associate with resources like Sandboxes.
    pub async fn lookup_or_create(
        name: &str,
        client: Option<Client>,
    ) -> Result<Self> {
        let client = if let Some(c) = client {
            c
        } else {
            Client::from_env().await?
        };

        let req = client::AppGetOrCreateRequest {
            app_name: name.to_string(),
            environment_name: String::new(),
            object_creation_type: client::ObjectCreationType::CreateIfMissing as i32,
        };
        let mut client = client;
        let resp = client.app_get_or_create(req).await?;

        if resp.app_id.is_empty() {
            return Err(Error::Other(anyhow!("Failed to create or get app '{}'", name)));
        }

        Ok(Self {
            client,
            app_id: Some(resp.app_id),
            name: Some(name.to_string()),
            tags: HashMap::new(),
        })
    }

    /// Deploy the app to Modal
    ///
    /// This will create or update the app on Modal's servers and publish it.
    /// Functions must be registered with the app before deployment.
    ///
    /// # Arguments
    /// * `deployment_tag` - Optional metadata tag for this deployment
    /// * `function_ids` - Map of function names to function IDs to deploy
    /// * `class_ids` - Map of class names to class IDs to deploy
    pub async fn deploy(
        &mut self,
        deployment_tag: Option<&str>,
        function_ids: Option<HashMap<String, String>>,
        class_ids: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let name = self.name.as_ref().ok_or_else(|| {
            Error::Other(anyhow!("App name is required for deployment"))
        })?;

        // First, create or get the app
        let app_req = client::AppGetOrCreateRequest {
            app_name: name.clone(),
            environment_name: String::new(),
            object_creation_type: client::ObjectCreationType::CreateIfMissing as i32,
        };
        let app_resp = self.client.app_get_or_create(app_req).await?;
        
        self.app_id = Some(app_resp.app_id.clone());
        let app_id = app_resp.app_id;

        // Publish the app with its functions and classes
        // This matches the Python client's AppPublish RPC
        let publish_req = client::AppPublishRequest {
            app_id: app_id.clone(),
            name: name.clone(),
            tags: self.tags.clone(),
            deployment_tag: deployment_tag.unwrap_or("").to_string(),
            app_state: client::AppState::Deployed as i32,
            function_ids: function_ids.unwrap_or_default(),
            class_ids: class_ids.unwrap_or_default(),
            definition_ids: HashMap::new(), // TODO: Track definition IDs if needed
            rollback_version: 0,
            client_version: String::new(),
            commit_info: None,
        };
        let publish_resp = self.client.app_publish(publish_req).await?;

        // Return the deployment URL if available
        Ok(publish_resp.url)
    }

    /// Get the app ID
    pub fn app_id(&self) -> Option<&str> {
        self.app_id.as_deref()
    }

    /// Get the app name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set tags on the app
    ///
    /// Tags can be set before deployment, or updated on a running app.
    pub fn set_tags(&mut self, tags: HashMap<String, String>) {
        self.tags = tags;
    }

    /// Update tags on a deployed app
    ///
    /// This will update the tags on Modal's servers for a running app.
    pub async fn update_tags(&mut self, tags: HashMap<String, String>) -> Result<()> {
        let app_id = self.app_id.as_ref().ok_or_else(|| {
            Error::Other(anyhow!("App must be deployed before updating tags"))
        })?;

        let req = client::AppSetTagsRequest {
            app_id: app_id.clone(),
            tags: tags.clone(),
        };
        self.client.app_set_tags(req).await?;
        
        self.tags = tags;
        Ok(())
    }

    /// Get tags from the app
    pub fn tags(&self) -> &HashMap<String, String> {
        &self.tags
    }

    /// Get tags from Modal's servers
    ///
    /// This fetches the current tags from a deployed app.
    pub async fn get_tags(&mut self) -> Result<HashMap<String, String>> {
        let app_id = self.app_id.as_ref().ok_or_else(|| {
            Error::Other(anyhow!("App must be deployed before getting tags"))
        })?;

        let req = client::AppGetTagsRequest {
            app_id: app_id.clone(),
        };
        let resp = self.client.app_get_tags(req).await?;
        
        Ok(resp.tags)
    }
}

