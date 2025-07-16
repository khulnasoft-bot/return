use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use log::{info, error};
use super::CONFIG_DIR;
use super::theme::Theme;
use super::yaml_theme::YamlTheme;

pub struct YamlThemeManager {
    theme_dir: PathBuf,
    themes: HashMap<String, Theme>,
}

impl YamlThemeManager {
    pub fn new() -> Self {
        let theme_dir = CONFIG_DIR.join("themes");
        Self {
            theme_dir,
            themes: HashMap::new(),
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        info!("YAML theme manager initialized. Theme directory: {:?}", self.theme_dir);
        fs::create_dir_all(&self.theme_dir).await?;
        self.load_default_themes().await?;
        self.load_user_themes().await?;
        Ok(())
    }

    async fn load_default_themes(&mut self) -> Result<()> {
        let defaults = vec![
            ("gruvbox-dark.yaml", include_str!("../../themes/gruvbox-dark.yaml")),
            ("nord.yaml", include_str!("../../themes/nord.yaml")),
        ];

        for (filename, content) in defaults {
            let theme_path = self.theme_dir.join(filename);
            if !theme_path.exists() {
                fs::write(&theme_path, content).await?;
            }
            let theme_contents = fs::read_to_string(&theme_path).await?;
            match YamlTheme::from_yaml(&theme_contents) {
                Ok(yaml_theme) => {
                    let theme: Theme = yaml_theme.into();
                    self.themes.insert(theme.name.clone(), theme);
                },
                Err(e) => {
                    error!("Failed to load default theme from {:?}: {}", theme_path, e);
                }
            }
        }
        info!("Loaded {} default themes.", self.themes.len());
        Ok(())
    }

    async fn load_user_themes(&mut self) -> Result<()> {
        let mut entries = fs::read_dir(&self.theme_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                let content = fs::read_to_string(&path).await?;
                match YamlTheme::from_yaml(&content) {
                    Ok(yaml_theme) => {
                        let theme: Theme = yaml_theme.into();
                        self.themes.insert(theme.name.clone(), theme);
                    },
                    Err(e) => {
                        error!("Failed to load user theme from {:?}: {}", path, e);
                    }
                }
            }
        }
        info!("Loaded {} user themes.", self.themes.len());
        Ok(())
    }

    pub async fn get_theme(&self, name: &str) -> Result<Theme> {
        self.themes.get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Theme '{}' not found.", name))
    }

    pub async fn list_themes(&self) -> Result<Vec<Theme>> {
        Ok(self.themes.values().cloned().collect())
    }

    pub async fn save_theme(&mut self, yaml_theme: YamlTheme) -> Result<()> {
        let path = self.theme_dir.join(format!("{}.yaml", yaml_theme.name));
        let contents = yaml_theme.to_yaml()?;
        fs::write(&path, contents).await?;
        info!("Theme '{}' saved to {:?}", yaml_theme.name, path);
        self.themes.insert(yaml_theme.name.clone(), yaml_theme.into()); // Update in-memory cache
        Ok(())
    }

    pub async fn delete_theme(&mut self, name: &str) -> Result<()> {
        let path = self.theme_dir.join(format!("{}.yaml", name));
        if path.exists() {
            fs::remove_file(&path).await?;
            info!("Theme '{}' deleted from {:?}", name, path);
        }
        self.themes.remove(name);
        Ok(())
    }
}

pub fn init() {
    info!("YAML theme manager module loaded");
}
