use iced::{Element, widget::{column, row, text, button, container, scrollable, pick_list, slider, checkbox, text_input}, Length, Color, alignment};
use crate::{Message, config::*};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::config::preferences::Preferences;
use crate::config::theme::ThemeConfig;
use crate::agent_mode_eval::ai_client::AiConfig;
use anyhow::Result;
use std::sync::Arc;
use crate::config::ConfigManager;
use crate::config::AppConfig;

pub mod theme_editor;
pub mod keybinding_editor;
pub mod yaml_theme_ui;
pub mod appearance_settings; // New module

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    General,
    Appearance, // New tab
    Terminal,
    Editor,
    KeyBindings,
    Themes,
    Plugins,
    AI,
    Privacy,
    Performance,
    Collaboration,
    CloudSync,
    Drive,
    Workflows,
    About,
}

impl Default for SettingsTab {
    fn default() -> Self {
        SettingsTab::General
    }
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    TabSelected(SettingsTab),
    KeybindingEditor(keybinding_editor::KeybindingEditorMessage),
    ThemeEditor(theme_editor::ThemeEditorMessage),
    AppearanceSettings(appearance_settings::AppearanceSettingsMessage), // New message variant
    SaveAll,
    CancelAll,
    // Add other settings messages here
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    pub config: AppConfig,
    selected_tab: SettingsTab,
    keybinding_editor: keybinding_editor::KeybindingEditor,
    theme_editor: theme_editor::ThemeEditor,
    appearance_settings: appearance_settings::AppearanceSettings, // New field
    yaml_theme_ui: yaml_theme_ui::YamlThemeUi, // Assuming this is for advanced YAML editing
    config_manager: Arc<ConfigManager>,
}

impl SettingsView {
    pub fn new(config: AppConfig) -> Self {
        let keybindings = config.preferences.keybindings.clone();
        let config_manager = Arc::new(tokio::runtime::Handle::current().block_on(async {
            ConfigManager::new().await.expect("Failed to initialize ConfigManager for settings")
        }));

        let mut theme_editor = theme_editor::ThemeEditor::new(config_manager.clone());
        tokio::runtime::Handle::current().block_on(async {
            theme_editor.load_initial_state().await;
        });

        let appearance_prefs = config.preferences.ui.clone();

        Self {
            keybinding_editor: keybinding_editor::KeybindingEditor::new(keybindings),
            theme_editor,
            appearance_settings: appearance_settings::AppearanceSettings::new(appearance_prefs),
            yaml_theme_ui: yaml_theme_ui::YamlThemeUi::new(config_manager.clone()),
            config_manager, // Store the dummy config manager
            config,
            selected_tab: SettingsTab::default(),
        }
    }

    pub async fn init(&mut self) {
        // Re-initialize components that need async setup
        self.theme_editor = theme_editor::ThemeEditor::new(self.config_manager.clone());
        self.theme_editor.init().await.unwrap();
        self.yaml_theme_ui.init().await.unwrap();
        // No async init needed for AppearanceSettings currently
    }

    pub fn update(&mut self, message: SettingsMessage) -> Command<SettingsMessage> {
        match message {
            SettingsMessage::TabSelected(tab) => {
                self.selected_tab = tab;
                Command::none()
            }
            SettingsMessage::KeybindingEditor(msg) => {
                self.keybinding_editor.update(msg);
                self.config.preferences.keybindings = self.keybinding_editor.keybindings.clone();
                Command::none()
            }
            SettingsMessage::ThemeEditor(msg) => {
                self.theme_editor.update(msg)
                    .map(SettingsMessage::ThemeEditor)
            }
            SettingsMessage::AppearanceSettings(msg) => {
                self.appearance_settings.update(msg);
                self.config.preferences.ui = self.appearance_settings.preferences.clone();
                Command::none()
            }
            SettingsMessage::SaveAll => {
                let config_clone = self.config.clone();
                Command::perform(
                    async move {
                        let config_manager = ConfigManager::new().await.expect("Failed to init ConfigManager for save");
                        if let Err(e) = config_manager.update_preferences(config_clone.preferences).await {
                            log::error!("Failed to save preferences: {}", e);
                        } else {
                            log::info!("All settings saved successfully.");
                        }
                    },
                    |_| SettingsMessage::CancelAll // Just a dummy message to indicate completion
                )
            }
            SettingsMessage::CancelAll => {
                // Reload original config to discard changes
                self.config = AppConfig::load().unwrap_or_default();
                self.keybinding_editor = keybinding_editor::KeybindingEditor::new(self.config.preferences.keybindings.clone());
                let config_manager = Arc::new(tokio::runtime::Handle::current().block_on(async {
                    ConfigManager::new().await.expect("Failed to initialize ConfigManager for settings reload")
                }));
                self.theme_editor = theme_editor::ThemeEditor::new(config_manager.clone());
                tokio::runtime::Handle::current().block_on(async {
                    self.theme_editor.load_initial_state().await;
                });
                self.appearance_settings = appearance_settings::AppearanceSettings::new(self.config.preferences.ui.clone());
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<SettingsMessage> {
        let header = text("Application Settings").size(30).color(Color::BLACK);

        let sidebar = column![
            self.nav_button(SettingsTab::General, "General"),
            self.nav_button(SettingsTab::Appearance, "Appearance"),
            self.nav_button(SettingsTab::Terminal, "Terminal"),
            self.nav_button(SettingsTab::Editor, "Editor"),
            self.nav_button(SettingsTab::KeyBindings, "Keybindings"),
            self.nav_button(SettingsTab::Themes, "Themes"),
            self.nav_button(SettingsTab::AI, "AI"),
            self.nav_button(SettingsTab::Plugins, "Plugins"),
            self.nav_button(SettingsTab::Workflows, "Workflows"),
            self.nav_button(SettingsTab::CloudSync, "Cloud Sync"),
            self.nav_button(SettingsTab::Collaboration, "Collaboration"),
            self.nav_button(SettingsTab::Privacy, "Privacy"),
            self.nav_button(SettingsTab::Performance, "Performance"),
        ]
        .spacing(5)
        .width(Length::Fixed(180.0));

        let main_content = scrollable(
            column![
                text("General Settings Placeholder").size(20),
                self.appearance_settings.view().map(SettingsMessage::AppearanceSettings),
                text("Terminal Settings Placeholder").size(20),
                text("Editor Settings Placeholder").size(20),
                self.keybinding_editor.view().map(SettingsMessage::KeybindingEditor),
                self.theme_editor.view().map(SettingsMessage::ThemeEditor),
                text("AI Settings Placeholder").size(20),
                text("Plugins Settings Placeholder").size(20),
                text("Workflows Settings Placeholder").size(20),
                text("Cloud Sync Settings Placeholder").size(20),
                text("Collaboration Settings Placeholder").size(20),
                text("Privacy Settings Placeholder").size(20),
                text("Performance Settings Placeholder").size(20),
            ]
            .spacing(20)
            .padding(20)
        )
        .width(Length::Fill);

        let controls = row![
            button(text("Save All")).on_press(SettingsMessage::SaveAll).padding(10),
            button(text("Cancel")).on_press(SettingsMessage::CancelAll).padding(10),
        ].spacing(10).align_items(alignment::Horizontal::Right).width(Length::Fill);

        column![
            header,
            row![
                sidebar,
                main_content,
            ]
            .spacing(20)
            .width(Length::Fill),
            controls,
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn nav_button(&self, tab: SettingsTab, label: &str) -> Element<SettingsMessage> {
        let is_selected = self.selected_tab == tab;
        button(text(label).size(16).color(if is_selected { Color::BLACK } else { Color::WHITE }))
            .on_press(SettingsMessage::TabSelected(tab))
            .style(if is_selected {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Text
            })
            .width(Length::Fill)
            .into()
    }
}

pub fn init() {
    log::info!("settings module loaded");
}
