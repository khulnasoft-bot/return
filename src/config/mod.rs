pub mod preferences;
pub mod theme;
pub mod yaml_theme;
pub mod yaml_theme_manager;

use anyhow::Result;
use preferences::UserPreferences;
use theme::Theme;
use yaml_theme_manager::YamlThemeManager;
use std::path::PathBuf;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use std::sync::Arc;
use log::info;

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

pub struct ConfigManager {
    preferences: Arc<RwLock<UserPreferences>>,
    theme_manager: Arc<YamlThemeManager>,
}

impl ConfigManager {
    pub async fn new() -> Result<Self> {
        // Ensure config directories exist
        tokio::fs::create_dir_all(&*CONFIG_DIR).await?;
        tokio::fs::create_dir_all(&*DATA_DIR).await?;
        tokio::fs::create_dir_all(&*CACHE_DIR).await?;

        let preferences = Arc::new(RwLock::new(UserPreferences::load().await?));
        let theme_manager = Arc::new(YamlThemeManager::new());

        Ok(Self {
            preferences,
            theme_manager,
        })
    }

    pub async fn init(&self) -> Result<()> {
        info!("Config manager initialized.");
        self.theme_manager.init().await?;
        Ok(())
    }

    pub async fn get_preferences(&self) -> UserPreferences {
        self.preferences.read().await.clone()
    }

    pub async fn update_preferences(&self, new_prefs: UserPreferences) -> Result<()> {
        let mut prefs = self.preferences.write().await;
        *prefs = new_prefs;
        prefs.save().await?;
        Ok(())
    }

    pub async fn get_current_theme(&self) -> Result<Theme> {
        let prefs = self.preferences.read().await;
        self.theme_manager.get_theme(&prefs.ui.theme_name).await
    }

    pub async fn get_theme_manager(&self) -> Arc<YamlThemeManager> {
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
