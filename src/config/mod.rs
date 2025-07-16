use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod preferences;
pub mod theme;
pub mod yaml_theme;
pub mod yaml_theme_manager;

use preferences::UserPreferences;
use yaml_theme_manager::YamlThemeManager;

/// Manages all application configuration, including user preferences and themes.
pub struct ConfigManager {
    user_preferences: Arc<RwLock<UserPreferences>>,
    yaml_theme_manager: Arc<RwLock<YamlThemeManager>>,
}

impl ConfigManager {
    pub async fn new() -> Result<Self> {
        let user_preferences = Arc::new(RwLock::new(UserPreferences::load_or_default().await?));
        let yaml_theme_manager = Arc::new(RwLock::new(YamlThemeManager::new()));

        // Initialize theme manager (e.g., load default themes)
        yaml_theme_manager.write().await.init().await?;

        Ok(Self {
            user_preferences,
            yaml_theme_manager,
        })
    }

    /// Gets a read-locked reference to the user preferences.
    pub fn get_preferences(&self) -> Arc<RwLock<UserPreferences>> {
        self.user_preferences.clone()
    }

    /// Gets a read-locked reference to the YAML theme manager.
    pub fn get_theme_manager(&self) -> Arc<RwLock<YamlThemeManager>> {
        self.yaml_theme_manager.clone()
    }

    /// Saves the current user preferences to disk.
    pub async fn save_preferences(&self) -> Result<()> {
        self.user_preferences.read().await.save().await
    }
}

pub fn init() {
    info!("config module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    // Helper to clean up test files
    async fn cleanup_test_files() {
        let _ = fs::remove_file("preferences.yaml").await;
        let _ = fs::remove_dir_all("themes").await;
    }

    #[tokio::test]
    async fn test_config_manager_new() {
        cleanup_test_files().await;
        let manager = ConfigManager::new().await.unwrap();

        // Check if preferences are loaded (default in this case)
        let prefs = manager.get_preferences().read().await;
        assert_eq!(prefs.general.font_size, 14);

        // Check if theme manager is initialized (should have default themes)
        let theme_manager = manager.get_theme_manager().read().await;
        assert!(!theme_manager.list_themes().await.is_empty());

        cleanup_test_files().await;
    }

    #[tokio::test]
    async fn test_config_manager_save_preferences() {
        cleanup_test_files().await;
        let manager = ConfigManager::new().await.unwrap();

        // Modify a preference
        manager.get_preferences().write().await.general.font_size = 16;
        manager.save_preferences().await.unwrap();

        // Load a new manager to verify persistence
        let new_manager = ConfigManager::new().await.unwrap();
        let new_prefs = new_manager.get_preferences().read().await;
        assert_eq!(new_prefs.general.font_size, 16);

        cleanup_test_files().await;
    }
}
