use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use super::yaml_theme::YamlTheme;

const THEMES_DIR: &str = "themes";

/// Manages loading, saving, and providing access to YAML-defined themes.
pub struct YamlThemeManager {
    themes: Arc<RwLock<HashMap<String, YamlTheme>>>, // theme_name -> YamlTheme
}

impl YamlThemeManager {
    pub fn new() -> Self {
        Self {
            themes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initializes the theme manager by loading default themes and then user-defined themes.
    pub async fn init(&self) -> Result<()> {
        info!("Initializing YamlThemeManager...");
        let mut themes_write = self.themes.write().await;

        // Load default themes embedded in the binary
        self.load_embedded_themes(&mut themes_write).await?;

        // Load user-defined themes from the themes directory
        self.load_user_themes(&mut themes_write).await?;

        info!("YamlThemeManager initialized. Loaded {} themes.", themes_write.len());
        Ok(())
    }

    async fn load_embedded_themes(&self, themes_map: &mut HashMap<String, YamlTheme>) -> Result<()> {
        info!("Loading embedded themes...");
        // Example: include a default theme directly
        let gruvbox_dark_yaml = include_str!("../../themes/gruvbox-dark.yaml");
        let nord_yaml = include_str!("../../themes/nord.yaml");

        let embedded_themes = vec![
            ("gruvbox-dark", gruvbox_dark_yaml),
            ("nord", nord_yaml),
        ];

        for (name, content) in embedded_themes {
            match serde_yaml::from_str::<YamlTheme>(content) {
                Ok(theme) => {
                    if theme.validate().is_ok() {
                        themes_map.insert(name.to_string(), theme);
                        info!("Loaded embedded theme: {}", name);
                    } else {
                        warn!("Embedded theme '{}' failed validation. Skipping.", name);
                    }
                }
                Err(e) => error!("Failed to parse embedded theme '{}': {}", name, e),
            }
        }
        Ok(())
    }

    async fn load_user_themes(&self, themes_map: &mut HashMap<String, YamlTheme>) -> Result<()> {
        info!("Loading user-defined themes from '{}'...", THEMES_DIR);
        let themes_path = PathBuf::from(THEMES_DIR);

        if !themes_path.exists() {
            info!("Themes directory '{}' does not exist. Creating it.", THEMES_DIR);
            fs::create_dir_all(&themes_path).await?;
            return Ok(());
        }

        let mut entries = fs::read_dir(&themes_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                let theme_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
                match fs::read_to_string(&path).await {
                    Ok(contents) => match serde_yaml::from_str::<YamlTheme>(&contents) {
                        Ok(theme) => {
                            if theme.validate().is_ok() {
                                themes_map.insert(theme_name.clone(), theme);
                                info!("Loaded user theme: {}", theme_name);
                            } else {
                                warn!("User theme '{}' failed validation. Skipping.", theme_name);
                            }
                        }
                        Err(e) => error!("Failed to parse user theme '{}': {}", theme_name, e),
                    },
                    Err(e) => error!("Failed to read user theme file '{}': {}", path.display(), e),
                }
            }
        }
        Ok(())
    }

    /// Returns a list of all available theme names.
    pub async fn list_themes(&self) -> Vec<String> {
        let themes_read = self.themes.read().await;
        themes_read.keys().cloned().collect()
    }

    /// Gets a specific theme by name.
    pub async fn get_theme(&self, name: &str) -> Option<YamlTheme> {
        let themes_read = self.themes.read().await;
        themes_read.get(name).cloned()
    }

    /// Saves a new or updated theme to disk and reloads it into memory.
    pub async fn save_theme(&self, theme: YamlTheme) -> Result<()> {
        theme.validate()?; // Validate before saving

        let file_name = format!("{}.yaml", theme.name);
        let path = Path::new(THEMES_DIR).join(file_name);

        let yaml_string = serde_yaml::to_string(&theme)?;
        fs::write(&path, yaml_string)
            .await
            .map_err(|e| anyhow!("Failed to save theme to {}: {}", path.display(), e))?;

        // Update in-memory map
        self.themes.write().await.insert(theme.name.clone(), theme);
        info!("Theme '{}' saved and reloaded.", theme.name);
        Ok(())
    }

    /// Deletes a theme from disk and removes it from memory.
    pub async fn delete_theme(&self, name: &str) -> Result<()> {
        let path = Path::new(THEMES_DIR).join(format!("{}.yaml", name));

        if path.exists().await {
            fs::remove_file(&path)
                .await
                .map_err(|e| anyhow!("Failed to delete theme file {}: {}", path.display(), e))?;
            self.themes.write().await.remove(name);
            info!("Theme '{}' deleted.", name);
            Ok(())
        } else {
            warn!("Attempted to delete non-existent theme: {}", name);
            Err(anyhow!("Theme '{}' not found.", name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    const TEST_THEMES_DIR: &str = "test_themes";

    async fn setup_test_dir() {
        let _ = fs::remove_dir_all(TEST_THEMES_DIR).await; // Clean up previous runs
        fs::create_dir_all(TEST_THEMES_DIR).await.unwrap();
    }

    async fn cleanup_test_dir() {
        let _ = fs::remove_dir_all(TEST_THEMES_DIR).await;
    }

    // Helper to temporarily redirect THEMES_DIR for tests
    macro_rules! with_test_themes_dir {
        ($body:expr) => {{
            let old_dir = THEMES_DIR;
            unsafe {
                let ptr = THEMES_DIR as *const _ as *mut &str;
                *ptr = TEST_THEMES_DIR;
            }
            let result = $body.await;
            unsafe {
                let ptr = THEMES_DIR as *const _ as *mut &str;
                *ptr = old_dir;
            }
            result
        }};
    }

    #[tokio::test]
    async fn test_yaml_theme_manager_init_and_load_embedded() {
        with_test_themes_dir!({
            setup_test_dir().await;
            let manager = YamlThemeManager::new();
            manager.init().await.unwrap();

            let themes = manager.list_themes().await;
            assert!(!themes.is_empty());
            assert!(themes.contains(&"gruvbox-dark".to_string()));
            assert!(themes.contains(&"nord".to_string()));

            let gruvbox = manager.get_theme("gruvbox-dark").await.unwrap();
            assert_eq!(gruvbox.name, "gruvbox-dark");

            cleanup_test_dir().await;
        })
    }

    #[tokio::test]
    async fn test_yaml_theme_manager_load_user_themes() {
        with_test_themes_dir!({
            setup_test_dir().await;

            let user_theme_content = r#"
name: MyCustomTheme
description: A custom theme
colors:
  background: "#282828"
  foreground: "#ebdbb2"
"#;
            fs::write(Path::new(TEST_THEMES_DIR).join("MyCustomTheme.yaml"), user_theme_content)
                .await
                .unwrap();

            let manager = YamlThemeManager::new();
            manager.init().await.unwrap();

            let themes = manager.list_themes().await;
            assert!(themes.contains(&"MyCustomTheme".to_string()));

            let custom_theme = manager.get_theme("MyCustomTheme").await.unwrap();
            assert_eq!(custom_theme.name, "MyCustomTheme");
            assert_eq!(custom_theme.colors.get("background").unwrap(), "#282828");

            cleanup_test_dir().await;
        })
    }

    #[tokio::test]
    async fn test_yaml_theme_manager_save_and_delete_theme() {
        with_test_themes_dir!({
            setup_test_dir().await;
            let manager = YamlThemeManager::new();
            manager.init().await.unwrap(); // Load embedded themes first

            let new_theme = YamlTheme {
                name: "NewTestTheme".to_string(),
                description: Some("A brand new theme".to_string()),
                author: Some("Test Author".to_string()),
                colors: {
                    let mut map = HashMap::new();
                    map.insert("background".to_string(), "#ABCDEF".to_string());
                    map
                },
                terminal_colors: None,
            };

            manager.save_theme(new_theme.clone()).await.unwrap();

            let themes_after_save = manager.list_themes().await;
            assert!(themes_after_save.contains(&"NewTestTheme".to_string()));
            assert!(manager.get_theme("NewTestTheme").await.is_some());
            assert!(fs::metadata(Path::new(TEST_THEMES_DIR).join("NewTestTheme.yaml")).await.is_ok());

            manager.delete_theme("NewTestTheme").await.unwrap();

            let themes_after_delete = manager.list_themes().await;
            assert!(!themes_after_delete.contains(&"NewTestTheme".to_string()));
            assert!(manager.get_theme("NewTestTheme").await.is_none());
            assert!(fs::metadata(Path::new(TEST_THEMES_DIR).join("NewTestTheme.yaml")).await.is_err()); // File should be gone

            cleanup_test_dir().await;
        })
    }

    #[tokio::test]
    async fn test_yaml_theme_manager_save_invalid_theme() {
        with_test_themes_dir!({
            setup_test_dir().await;
            let manager = YamlThemeManager::new();
            manager.init().await.unwrap();

            let invalid_theme = YamlTheme {
                name: "InvalidTheme".to_string(),
                description: None,
                author: None,
                colors: {
                    let mut map = HashMap::new();
                    map.insert("background".to_string(), "not-a-color".to_string());
                    map
                },
                terminal_colors: None,
            };

            let result = manager.save_theme(invalid_theme.clone()).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Invalid color format"));

            // Ensure the invalid theme was not added to memory or disk
            assert!(manager.get_theme("InvalidTheme").await.is_none());
            assert!(fs::metadata(Path::new(TEST_THEMES_DIR).join("InvalidTheme.yaml")).await.is_err());

            cleanup_test_dir().await;
        })
    }
}
