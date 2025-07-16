use iced::{
    widget::{column, row, text, button, pick_list, scrollable},
    Element, Length, Color, alignment, Command,
};
use std::collections::HashMap;
use crate::config::theme::Theme; // Assuming this is the `iced::Theme` wrapper
use crate::config::yaml_theme::{YamlTheme, YamlThemeUi, YamlThemeUiMessage};
use crate::config::ConfigManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

#[derive(Debug, Clone)]
pub enum ThemeEditorMessage {
    SelectTheme(String),
    EditSelectedTheme, // Renamed for clarity
    YamlThemeUi(YamlThemeUiMessage), // Message from the YamlThemeUi sub-component
    SaveEditedTheme(YamlTheme), // Message to save the edited theme
    LoadThemes, // Trigger to reload the list of themes
    Error(String),
}

pub struct ThemeEditor {
    config_manager: Arc<ConfigManager>,
    available_theme_names: Vec<String>, // Store just names for pick_list
    selected_theme_name: String,
    yaml_theme_ui: Option<YamlThemeUi>, // This will hold the editor for the currently selected theme
    error_message: Option<String>,
}

impl ThemeEditor {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self {
            config_manager,
            available_theme_names: Vec::new(),
            selected_theme_name: "nord".to_string(), // Default to 'nord' or first available
            yaml_theme_ui: None,
            error_message: None,
        }
    }

    pub async fn load_initial_state(&mut self) {
        // Load current theme preference
        let prefs = self.config_manager.get_preferences().read().await;
        self.selected_theme_name = prefs.ui.theme_name.clone();
        drop(prefs); // Release lock

        // Load all available themes
        self.load_themes_list().await;
    }

    async fn load_themes_list(&mut self) {
        let theme_manager = self.config_manager.get_theme_manager().read().await;
        match theme_manager.list_themes().await {
            Ok(names) => {
                self.available_theme_names = names;
                // Ensure selected_theme_name is still valid, or pick a default
                if !self.available_theme_names.contains(&self.selected_theme_name) {
                    if let Some(first_theme) = self.available_theme_names.first() {
                        self.selected_theme_name = first_theme.clone();
                    } else {
                        self.selected_theme_name = "default".to_string(); // Fallback if no themes
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
                self.yaml_theme_ui = None; // Close editor when selecting a new theme
                let config_manager_clone = self.config_manager.clone();
                Command::perform(
                    async move {
                        let mut prefs = config_manager_clone.get_preferences().write().await;
                        prefs.ui.theme_name = name;
                        if let Err(e) = config_manager_clone.save_preferences().await {
                            ThemeEditorMessage::Error(format!("Failed to save theme preference: {}", e))
                        } else {
                            info!("Theme preference updated.");
                            ThemeEditorMessage::LoadThemes // Trigger reload of themes list (to update current selection)
                        }
                    },
                    |msg| msg
                )
            }
            ThemeEditorMessage::EditSelectedTheme => {
                let theme_manager_arc = self.config_manager.get_theme_manager();
                let selected_name = self.selected_theme_name.clone();
                Command::perform(
                    async move {
                        let theme_manager = theme_manager_arc.read().await;
                        match theme_manager.get_theme(&selected_name).await {
                            Ok(yaml_theme) => {
                                // Return the loaded YamlTheme to be used to create YamlThemeUi
                                Some(yaml_theme)
                            },
                            Err(e) => {
                                error!("Failed to load theme for editing: {}", e);
                                None
                            },
                        }
                    },
                    |result| {
                        if let Some(yaml_theme) = result {
                            // Create the YamlThemeUi instance here
                            ThemeEditorMessage::YamlThemeUi(YamlThemeUiMessage::LoadTheme(yaml_theme))
                        } else {
                            ThemeEditorMessage::Error("Failed to load theme for editing.".to_string())
                        }
                    }
                )
            }
            ThemeEditorMessage::YamlThemeUi(msg) => {
                // If the message is to load a theme, create the YamlThemeUi
                if let YamlThemeUiMessage::LoadTheme(yaml_theme) = msg {
                    self.yaml_theme_ui = Some(YamlThemeUi::new(yaml_theme));
                    Command::none()
                } else if let Some(ui) = &mut self.yaml_theme_ui {
                    ui.update(msg.clone());
                    match msg {
                        YamlThemeUiMessage::Save => {
                            if ui.error_message.is_none() {
                                let edited_yaml_theme = ui.theme.clone();
                                return Command::perform(
                                    async move {
                                        ThemeEditorMessage::SaveEditedTheme(edited_yaml_theme)
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
                    Command::none()
                } else {
                    // This case should ideally not be reached if logic is correct
                    error!("Received YamlThemeUiMessage but yaml_theme_ui is None.");
                    Command::none()
                }
            }
            ThemeEditorMessage::SaveEditedTheme(yaml_theme) => {
                let theme_manager_arc = self.config_manager.get_theme_manager();
                Command::perform(
                    async move {
                        let theme_manager = theme_manager_arc.write().await;
                        match theme_manager.save_theme(yaml_theme).await {
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
                        let prefs = config_manager_clone.get_preferences().read().await;
                        let current_theme_name = prefs.ui.theme_name.clone();
                        drop(prefs); // Release lock
                        
                        let theme_manager = config_manager_clone.get_theme_manager().read().await;
                        match theme_manager.list_themes().await {
                            Ok(themes) => {
                                // This message should trigger a full reload of the theme list and re-select the current one
                                ThemeEditorMessage::SelectTheme(current_theme_name)
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

        let theme_selector = row![
            text("Select Theme:").width(Length::Fixed(120.0)),
            pick_list(
                self.available_theme_names.clone(),
                Some(self.selected_theme_name.clone()),
                ThemeEditorMessage::SelectTheme,
            )
            .width(Length::Fill),
            button(text("Edit Selected Theme"))
                .on_press(ThemeEditorMessage::EditSelectedTheme)
                .padding(8)
        ].spacing(10).align_items(alignment::Horizontal::Center);

        let editor_content: Element<ThemeEditorMessage> = if let Some(ui) = &self.yaml_theme_ui {
            ui.view().map(ThemeEditorMessage::YamlThemeUi)
        } else {
            column![
                text("Select a theme and click 'Edit Selected Theme' to modify its properties.").size(16).color(Color::from_rgb(0.5, 0.5, 0.5))
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
