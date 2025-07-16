use iced::{
    widget::{column, row, text, button, pick_list, scrollable},
    Element, Length, Color, alignment,
};
use std::collections::HashMap;
use crate::config::theme::Theme;
use crate::config::yaml_theme::{YamlTheme, YamlThemeUi, YamlThemeUiMessage};
use crate::config::ConfigManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::info;

#[derive(Debug, Clone)]
pub enum ThemeEditorMessage {
    SelectTheme(String),
    EditCurrentTheme,
    YamlThemeUiMessage(YamlThemeUiMessage),
    SaveTheme(YamlTheme), // Message to save the edited theme
    LoadThemes,
    Error(String),
}

pub struct ThemeEditor {
    config_manager: Arc<ConfigManager>,
    available_themes: Vec<Theme>,
    selected_theme_name: String,
    yaml_theme_ui: Option<YamlThemeUi>,
    error_message: Option<String>,
}

impl ThemeEditor {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        let default_theme_name = "default".to_string();
        Self {
            config_manager,
            available_themes: Vec::new(),
            selected_theme_name: default_theme_name,
            yaml_theme_ui: None,
            error_message: None,
        }
    }

    pub async fn load_initial_state(&mut self) {
        match self.config_manager.get_current_theme().await {
            Ok(theme) => self.selected_theme_name = theme.name,
            Err(e) => self.error_message = Some(format!("Failed to load current theme: {}", e)),
        }
        self.load_themes().await;
    }

    pub async fn load_themes(&mut self) {
        let theme_manager = self.config_manager.get_theme_manager().await;
        match theme_manager.list_themes().await {
            Ok(themes) => {
                self.available_themes = themes;
                if !self.available_themes.iter().any(|t| t.name == self.selected_theme_name) {
                    if let Some(first_theme) = self.available_themes.first() {
                        self.selected_theme_name = first_theme.name.clone();
                    }
                }
            },
            Err(e) => self.error_message = Some(format!("Failed to load available themes: {}", e)),
        }
    }

    pub fn update(&mut self, message: ThemeEditorMessage) -> Command<ThemeEditorMessage> {
        self.error_message = None; // Clear previous errors
        match message {
            ThemeEditorMessage::SelectTheme(name) => {
                self.selected_theme_name = name.clone();
                let config_manager_clone = self.config_manager.clone();
                Command::perform(
                    async move {
                        let mut prefs = config_manager_clone.get_preferences().await;
                        prefs.theme_name = name;
                        if let Err(e) = config_manager_clone.update_preferences(prefs).await {
                            ThemeEditorMessage::Error(format!("Failed to save theme preference: {}", e))
                        } else {
                            info!("Theme preference updated.");
                            ThemeEditorMessage::LoadThemes // Reload themes to reflect change
                        }
                    },
                    |msg| msg
                )
            }
            ThemeEditorMessage::EditCurrentTheme => {
                let theme_manager = self.config_manager.get_theme_manager();
                let selected_name = self.selected_theme_name.clone();
                Command::perform(
                    async move {
                        match theme_manager.await.get_theme(&selected_name).await {
                            Ok(theme) => {
                                let yaml_theme: YamlTheme = theme.into();
                                ThemeEditorMessage::YamlThemeUiMessage(YamlThemeUiMessage::NameChanged(yaml_theme.name.clone())) // Dummy message to trigger UI update
                            },
                            Err(e) => ThemeEditorMessage::Error(format!("Failed to load theme for editing: {}", e)),
                        }
                    },
                    |msg| msg
                )
            }
            ThemeEditorMessage::YamlThemeUiMessage(msg) => {
                if let Some(ui) = &mut self.yaml_theme_ui {
                    ui.update(msg.clone());
                    match msg {
                        YamlThemeUiMessage::Save => {
                            if ui.error_message.is_none() {
                                let edited_yaml_theme = ui.theme.clone();
                                return Command::perform(
                                    async move {
                                        ThemeEditorMessage::SaveTheme(edited_yaml_theme)
                                    },
                                    |m| m
                                );
                            }
                        },
                        YamlThemeUiMessage::Cancel => {
                            self.yaml_theme_ui = None; // Close editor
                        },
                        YamlThemeUiMessage::Error(e) => {
                            self.error_message = Some(e);
                        }
                        _ => {}
                    }
                } else {
                    // If YamlThemeUi is not yet initialized, create it
                    if let YamlThemeUiMessage::NameChanged(name) = msg { // This is a hack to get the theme name
                        let theme_manager = self.config_manager.get_theme_manager();
                        let selected_name = self.selected_theme_name.clone();
                        let config_manager_clone = self.config_manager.clone();
                        return Command::perform(
                            async move {
                                match theme_manager.await.get_theme(&selected_name).await {
                                    Ok(theme) => {
                                        let yaml_theme: YamlTheme = theme.into();
                                        ThemeEditorMessage::YamlThemeUiMessage(YamlThemeUiMessage::NameChanged(yaml_theme.name.clone())) // Re-send the message to the newly created UI
                                    },
                                    Err(e) => ThemeEditorMessage::Error(format!("Failed to load theme for editing: {}", e)),
                                }
                            },
                            |msg| msg
                        );
                    }
                }
                Command::none()
            }
            ThemeEditorMessage::SaveTheme(yaml_theme) => {
                let theme_manager = self.config_manager.get_theme_manager();
                Command::perform(
                    async move {
                        match theme_manager.await.save_theme(yaml_theme).await {
                            Ok(_) => {
                                info!("Theme saved successfully.");
                                ThemeEditorMessage::LoadThemes // Reload themes to reflect changes
                            },
                            Err(e) => ThemeEditorMessage::Error(format!("Failed to save theme: {}", e)),
                        }
                    },
                    |msg| msg
                )
            }
            ThemeEditorMessage::LoadThemes => {
                let config_manager_clone = self.config_manager.clone();
                Command::perform(
                    async move {
                        let theme_manager = config_manager_clone.get_theme_manager().await;
                        match theme_manager.list_themes().await {
                            Ok(themes) => {
                                let current_theme_name = config_manager_clone.get_preferences().await.theme_name;
                                ThemeEditorMessage::SelectTheme(current_theme_name) // Select current theme after loading
                            },
                            Err(e) => ThemeEditorMessage::Error(format!("Failed to load themes: {}", e)),
                        }
                    },
                    |msg| msg
                )
            }
            ThemeEditorMessage::Error(msg) => {
                self.error_message = Some(msg);
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<ThemeEditorMessage> {
        let header = text("Theme Editor").size(24).color(Color::BLACK);

        let theme_names: Vec<String> = self.available_themes.iter().map(|t| t.name.clone()).collect();

        let theme_selector = row![
            text("Select Theme:").width(Length::Fixed(120.0)),
            pick_list(
                theme_names,
                Some(self.selected_theme_name.clone()),
                ThemeEditorMessage::SelectTheme,
            )
            .width(Length::Fill),
            button(text("Edit Current Theme"))
                .on_press(ThemeEditorMessage::EditCurrentTheme)
                .padding(8)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let editor_content: Element<ThemeEditorMessage> = if let Some(ui) = &self.yaml_theme_ui {
            ui.view().map(ThemeEditorMessage::YamlThemeUiMessage)
        } else {
            column![
                text("No theme selected for editing, or editor closed.").size(16).color(Color::from_rgb(0.5, 0.5, 0.5))
            ].into()
        };

        let error_display = if let Some(msg) = &self.error_message {
            text(msg).color(Color::RED).size(14).into()
        } else {
            column![].into()
        };

        column![
            header,
            theme_selector,
            error_display,
            editor_content,
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

pub fn init() {
    info!("Theme editor module loaded");
}
