use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use directories::ProjectDirs;
use once_cell::sync::Lazy;

pub mod preferences;
pub mod theme;
pub mod yaml_theme;
pub mod yaml_theme_manager;

pub static PROJECT_DIRS: Lazy<Option<ProjectDirs>> = Lazy::new(|| {
    ProjectDirs::from("com", "NeoTerm", "NeoTerm")
});

pub static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PROJECT_DIRS.as_ref()
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./config")) // Fallback for systems without standard dirs
});

pub static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PROJECT_DIRS.as_ref()
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data")) // Fallback
});

pub static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PROJECT_DIRS.as_ref()
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./cache")) // Fallback
});

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub preferences: UserPreferences,
    // Add other top-level config items here if needed
    // e.g., environment profiles, plugin configurations
    pub env_profiles: preferences::EnvironmentProfiles,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            preferences: UserPreferences::default(),
            env_profiles: preferences::EnvironmentProfiles::default(),
        }
    }
}

impl AppConfig {
    pub async fn load() -> Result<Self> {
        let preferences = UserPreferences::load().await?;
        // Load other config components here if they have separate files
        Ok(Self {
            preferences,
            env_profiles: preferences::EnvironmentProfiles::default(), // For now, use default or load from preferences
        })
    }
}

use preferences::UserPreferences;
use yaml_theme_manager::YamlThemeManager;
use theme::Theme;

/// Manages all application configuration, including user preferences and themes.
#[derive(Debug)]
pub struct ConfigManager {
    preferences: Arc<RwLock<UserPreferences>>,
    theme_manager: Arc<RwLock<YamlThemeManager>>, // Wrap in RwLock as YamlThemeManager has mutable state
}

impl ConfigManager {
    pub async fn new() -> Result<Self> {
        // Ensure config directories exist
        tokio::fs::create_dir_all(&*CONFIG_DIR).await?;
        tokio::fs::create_dir_all(&*DATA_DIR).await?;
        tokio::fs::create_dir_all(&*CACHE_DIR).await?;

        let preferences = Arc::new(RwLock::new(UserPreferences::load().await?));
        let theme_manager = Arc::new(RwLock::new(YamlThemeManager::new())); // Initialize with RwLock

        Ok(Self {
            preferences,
            theme_manager,
        })
    }

    pub async fn init(&self) -> Result<()> {
        info!("Config manager initialized.");
        self.theme_manager.write().await.init().await?; // Call init on the inner YamlThemeManager
        Ok(())
    }

    /// Gets a read-locked reference to the user preferences.
    pub async fn get_preferences(&self) -> UserPreferences {
        self.preferences.read().await.clone()
    }

    /// Updates the user preferences and saves them to disk.
    pub async fn update_preferences(&self, new_prefs: UserPreferences) -> Result<()> {
        let mut prefs = self.preferences.write().await;
        *prefs = new_prefs;
        prefs.save().await?;
        Ok(())
    }

    /// Gets the current theme based on the user preferences.
    pub async fn get_current_theme(&self) -> Result<Theme> {
        let prefs = self.preferences.read().await;
        let yaml_theme = self.theme_manager.read().await.get_theme(&prefs.ui.theme_name).await?;
        Ok(Theme {
            name: yaml_theme.name.clone(),
            iced_theme: yaml_theme.to_iced_theme(),
        })
    }

    /// Gets a read-locked reference to the YAML theme manager.
    pub async fn get_theme_manager(&self) -> Arc<RwLock<YamlThemeManager>> { // Return Arc<RwLock>
        self.theme_manager.clone()
    }
}

pub fn init() {
    info!("Config module initialized.");
    // Accessing lazy statics here to ensure they are initialized early
    let _ = &*CONFIG_DIR;
    let _ = &*DATA_DIR;
    let _ = &*CACHE_DIR;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use std::path::Path;

    async fn setup_test_dirs() -> Result<()> {
        let test_config_dir = PathBuf::from("./test_config");
        let test_data_dir = PathBuf::from("./test_data");
        let test_cache_dir = PathBuf::from("./test_cache");

        if test_config_dir.exists() { fs::remove_dir_all(&test_config_dir).await?; }
        if test_data_dir.exists() { fs::remove_dir_all(&test_data_dir).await?; }
        if test_cache_dir.exists() { fs::remove_dir_all(&test_cache_dir).await?; }

        tokio::fs::create_dir_all(&test_config_dir).await?;
        tokio::fs::create_dir_all(&test_data_dir).await?;
        tokio::fs::create_dir_all(&test_cache_dir).await?;

        // Temporarily override lazy statics for testing
        // This is tricky with `once_cell::sync::Lazy`. A better approach for real tests
        // would be to pass paths explicitly or use a test-specific `ProjectDirs` mock.
        // For now, we'll rely on the default behavior and ensure directories are created.
        Ok(())
    }

    #[tokio::test]
    async fn test_config_manager_new_and_init() -> Result<()> {
        setup_test_dirs().await?;
        let config_manager = ConfigManager::new().await?;
        config_manager.init().await?;

        // Verify preferences are loaded (default if not exists)
        let prefs = config_manager.get_preferences().await;
        assert_eq!(prefs.ui.theme_name, "nord");

        // Verify theme manager is initialized and has themes
        let theme_manager = config_manager.get_theme_manager().await;
        let themes = theme_manager.read().await.list_themes().await?;
        assert!(!themes.is_empty());
        assert!(themes.iter().any(|t| t.name == "nord"));
        assert!(themes.iter().any(|t| t.name == "gruvbox-dark"));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_preferences() -> Result<()> {
        setup_test_dirs().await?;
        let config_manager = ConfigManager::new().await?;
        config_manager.init().await?;

        let mut prefs = config_manager.get_preferences().await;
        prefs.terminal.font_size = 16;
        prefs.ui.theme_name = "gruvbox-dark".to_string();

        config_manager.update_preferences(prefs).await?;

        let updated_prefs = config_manager.get_preferences().await;
        assert_eq!(updated_prefs.terminal.font_size, 16);
        assert_eq!(updated_prefs.ui.theme_name, "gruvbox-dark");

        // Verify theme change is reflected
        let current_theme = config_manager.get_current_theme().await?;
        assert_eq!(current_theme.name, "gruvbox-dark");

        Ok(())
    }
}
