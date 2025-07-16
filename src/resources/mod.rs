//! This module manages application resources such as icons, images, sounds,
//! and other static assets. It can handle loading them from disk or embedded
//! sources (e.g., using `asset_macro`).
//!
//! It also includes basic file validation to ensure resource integrity.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Represents the type of a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Image,
    Icon,
    Sound,
    Font,
    Other,
}

/// Represents a single application resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub resource_type: ResourceType,
    pub path: PathBuf, // Path to the resource file
    pub metadata: HashMap<String, String>, // e.g., "format": "png", "size": "1024x768"
}

/// Manages the loading and access of application resources.
pub struct ResourceManager {
    resources: HashMap<String, Resource>,
    resource_dir: PathBuf,
}

impl ResourceManager {
    /// Creates a new `ResourceManager` instance.
    ///
    /// Initializes the resource directory path.
    pub fn new() -> Self {
        let resource_dir = PathBuf::from("./assets"); // Or use a config-defined path
        Self {
            resources: HashMap::new(),
            resource_dir,
        }
    }

    /// Initializes the resource manager.
    ///
    /// This function ensures the resource directory exists and loads any default resources.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `anyhow::Error` if initialization fails.
    pub async fn init(&self) -> Result<()> {
        log::info!("Resource manager initialized. Resource directory: {:?}", self.resource_dir);
        fs::create_dir_all(&self.resource_dir)
            .await
            .context(format!("Failed to create resource directory at {:?}", self.resource_dir))?;
        self.load_default_resources().await?;
        Ok(())
    }

    /// Loads default resources into the manager.
    ///
    /// This function simulates creating and loading a default icon. In a real application,
    /// it would scan a directory or load embedded assets.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an `anyhow::Error` if loading fails.
    async fn load_default_resources(&self) -> Result<()> {
        // Simulate loading some default resources
        // In a real app, you'd scan the resource_dir or use embedded assets.
        let mut resources = self.resources.clone(); // Clone to modify

        let icon_path = self.resource_dir.join("default_icon.png");
        if !icon_path.exists() {
            // Simulate creating a dummy file
            fs::write(&icon_path, b"// Dummy PNG content")
                .await
                .context(format!("Failed to write dummy content to {:?}", icon_path))?;
            // Validate that the file was actually created
            if !icon_path.exists() {
                return Err(anyhow::anyhow!("Failed to create default icon file at {:?}", icon_path));
            }
        }
        resources.insert("default_icon".to_string(), Resource {
            id: "default_icon".to_string(),
            name: "Default App Icon".to_string(),
            resource_type: ResourceType::Icon,
            path: icon_path,
            metadata: [("format".to_string(), "png".to_string())].iter().cloned().collect(),
        });

        log::info!("Loaded {} default resources.", resources.len());
        Ok(())
    }

    /// Retrieves a resource by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the resource.
    ///
    /// # Returns
    ///
    /// An `Option<Resource>` containing a clone of the resource if found, otherwise `None`.
    pub async fn get_resource(&self, id: &str) -> Option<Resource> {
        self.resources.get(id).cloned()
    }

    /// Loads the content of a resource as bytes.
    ///
    /// This function performs a file existence check before attempting to read.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the resource.
    ///
    /// # Returns
    ///
    /// A `Result` containing the resource content as `Vec<u8>` or an `anyhow::Error` if not found or reading fails.
    pub async fn load_resource_bytes(&self, id: &str) -> Result<Vec<u8>> {
        if let Some(resource) = self.resources.get(id) {
            if !resource.path.exists() {
                return Err(anyhow::anyhow!("Resource file '{}' not found at path: {:?}", id, resource.path));
            }
            fs::read(&resource.path)
                .await
                .context(format!("Failed to read resource bytes from {:?}", resource.path))
        } else {
            Err(anyhow::anyhow!("Resource '{}' not found.", id))
        }
    }

    /// Loads the content of a resource as a string (for text-based resources).
    ///
    /// This function performs a file existence check before attempting to read.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the resource.
    ///
    /// # Returns
    ///
    /// A `Result` containing the resource content as `String` or an `anyhow::Error` if not found or reading fails.
    pub async fn load_resource_string(&self, id: &str) -> Result<String> {
        if let Some(resource) = self.resources.get(id) {
            if !resource.path.exists() {
                return Err(anyhow::anyhow!("Resource file '{}' not found at path: {:?}", id, resource.path));
            }
            fs::read_to_string(&resource.path)
                .await
                .context(format!("Failed to read resource string from {:?}", resource.path))
        } else {
            Err(anyhow::anyhow!("Resource '{}' not found.", id))
        }
    }
}

/// Initializes the resources module.
pub fn init() {
    log::info!("Resources module initialized.");
}
