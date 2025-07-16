use iced::{Element, widget::{column, row, text, button, scrollable, horizontal_rule}, Length, Color, alignment, Command};
use crate::config::*;
use std::sync::Arc;
use log::{info, error};

pub mod theme_editor;
pub mod keybinding_editor;
pub mod yaml_theme_ui;
pub mod appearance_settings;
pub mod indexing_settings;
pub mod ai_settings; // Import the new module

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    General,
    Appearance,
    Terminal,
    Editor,
    KeyBindings,
    Themes,
    Plugins,
    AI, // Add new tab
    Privacy,
    Performance,
    Collaboration,
    CloudSync,
    Drive,
    Workflows,
    Indexing,
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
    AppearanceSettings(appearance_settings::AppearanceSettingsMessage),
    IndexingSettings(indexing_settings::IndexingSettingsMessage),
    AiSettings(ai_settings::AiSettingsMessage), // Add new message
    SaveAll,
    CancelAll,
}

#[derive(Debug)]
pub struct SettingsView {
    pub config: AppConfig,
    selected_tab: SettingsTab,
    keybinding_editor: keybinding_editor::KeybindingEditor,
    theme_editor: theme_editor::ThemeEditor,
    appearance_settings: appearance_settings::AppearanceSettings,
    indexing_settings: indexing_settings::IndexingSettings,
    ai_settings: ai_settings::AiSettings, // Add new field
    config_manager: Arc<ConfigManager>,
}

impl SettingsView {
    pub fn new(config: AppConfig) -> Self {
        let config_manager = Arc::new(tokio::runtime::Handle::current().block_on(async {
            ConfigManager::new().await.expect("Failed to initialize ConfigManager for settings")
        }));

        let keybindings = config.preferences.keybindings.clone();
        let appearance_prefs = config.preferences.ui.clone();
        let indexing_prefs = config.preferences.indexing.clone();
        let ai_prefs = config.preferences.ai.clone(); // Clone AI preferences

        let mut theme_editor = theme_editor::ThemeEditor::new(config_manager.clone());
        tokio::runtime::Handle::current().block_on(async {
            theme_editor.load_initial_state().await;
        });

        Self {
            config,
            selected_tab: SettingsTab::default(),
            keybinding_editor: keybinding_editor::KeybindingEditor::new(keybindings),
            theme_editor,
            appearance_settings: appearance_settings::AppearanceSettings::new(appearance_prefs),
            indexing_settings: indexing_settings::IndexingSettings::new(indexing_prefs),
            ai_settings: ai_settings::AiSettings::new(ai_prefs), // Initialize new settings
            config_manager,
        }
    }

    pub async fn init(&mut self) {
        info!("SettingsView init called (placeholder).");
    }

    pub fn update(&mut self, message: SettingsMessage) -> Command<SettingsMessage> {
        match message {
            SettingsMessage::TabSelected(tab) => {
                self.selected_tab = tab;
                Command::none()
            }
            SettingsMessage::KeybindingEditor(msg) => {
                let command = self.keybinding_editor.update(msg);
                self.config.preferences.keybindings = self.keybinding_editor.keybindings.clone();
                command.map(SettingsMessage::KeybindingEditor)
            }
            SettingsMessage::ThemeEditor(msg) => {
                let command = self.theme_editor.update(msg);
                command.map(SettingsMessage::ThemeEditor)
            }
            SettingsMessage::AppearanceSettings(msg) => {
                self.appearance_settings.update(msg);
                self.config.preferences.ui = self.appearance_settings.preferences.clone();
                Command::none()
            }
            SettingsMessage::IndexingSettings(msg) => {
                self.indexing_settings.update(msg);
                self.config.preferences.indexing = self.indexing_settings.preferences.clone();
                Command::none()
            }
            SettingsMessage::AiSettings(msg) => { // Handle new message
                self.ai_settings.update(msg);
                self.config.preferences.ai = self.ai_settings.preferences.clone();
                Command::none()
            }
            SettingsMessage::SaveAll => {
                let preferences_to_save = self.config.preferences.clone();
                let config_manager_clone = self.config_manager.clone();
                Command::perform(
                    async move {
                        if let Err(e) = config_manager_clone.update_preferences(preferences_to_save).await {
                            error!("Failed to save preferences: {}", e);
                        } else {
                            info!("All settings saved successfully.");
                        }
                    },
                    |_| SettingsMessage::CancelAll
                )
            }
            SettingsMessage::CancelAll => {
                self.config = AppConfig::load().unwrap_or_default();
                self.keybinding_editor = keybinding_editor::KeybindingEditor::new(self.config.preferences.keybindings.clone());
                self.appearance_settings = appearance_settings::AppearanceSettings::new(self.config.preferences.ui.clone());
                self.indexing_settings = indexing_settings::IndexingSettings::new(self.config.preferences.indexing.clone());
                self.ai_settings = ai_settings::AiSettings::new(self.config.preferences.ai.clone()); // Reload new settings
                let config_manager_clone = self.config_manager.clone();
                let mut theme_editor_reloaded = theme_editor::ThemeEditor::new(config_manager_clone);
                tokio::runtime::Handle::current().block_on(async {
                    theme_editor_reloaded.load_initial_state().await;
                });
                self.theme_editor = theme_editor_reloaded;
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<SettingsMessage> {
        let header = text("Application Settings").size(30).color(Color::WHITE);

        let sidebar = column![
            self.nav_button(SettingsTab::General, "General"),
            self.nav_button(SettingsTab::Appearance, "Appearance"),
            self.nav_button(SettingsTab::Terminal, "Terminal"),
            self.nav_button(SettingsTab::Editor, "Editor"),
            self.nav_button(SettingsTab::KeyBindings, "Keybindings"),
            self.nav_button(SettingsTab::Themes, "Themes"),
            self.nav_button(SettingsTab::Plugins, "Plugins"),
            self.nav_button(SettingsTab::AI, "AI"), // Add new nav button
            self.nav_button(SettingsTab::Indexing, "Indexing"),
            self.nav_button(SettingsTab::Privacy, "Privacy"),
            self.nav_button(SettingsTab::Performance, "Performance"),
            self.nav_button(SettingsTab::Collaboration, "Collaboration"),
            self.nav_button(SettingsTab::CloudSync, "Cloud Sync"),
            self.nav_button(SettingsTab::Drive, "Drive"),
            self.nav_button(SettingsTab::Workflows, "Workflows"),
            self.nav_button(SettingsTab::About, "About"),
        ]
        .spacing(5)
        .width(Length::Fixed(180.0));

        let current_tab_content: Element<SettingsMessage> = match self.selected_tab {
            SettingsTab::General => text("General Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::Appearance => self.appearance_settings.view().map(SettingsMessage::AppearanceSettings),
            SettingsTab::Terminal => text("Terminal Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::Editor => text("Editor Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::KeyBindings => self.keybinding_editor.view().map(SettingsMessage::KeybindingEditor),
            SettingsTab::Themes => self.theme_editor.view().map(SettingsMessage::ThemeEditor),
            SettingsTab::Plugins => text("Plugins Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::AI => self.ai_settings.view().map(SettingsMessage::AiSettings), // Render new settings
            SettingsTab::Indexing => self.indexing_settings.view().map(SettingsMessage::IndexingSettings),
            SettingsTab::Privacy => text("Privacy Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::Performance => text("Performance Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::Collaboration => text("Collaboration Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::CloudSync => text("Cloud Sync Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::Drive => text("Drive Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::Workflows => text("Workflows Settings Placeholder").size(20).color(Color::WHITE).into(),
            SettingsTab::About => text("About NeoTerm Placeholder").size(20).color(Color::WHITE).into(),
        };

        let main_content = scrollable(
            column![
                current_tab_content,
            ]
            .spacing(20)
            .padding(20)
        )
        .width(Length::Fill);

        let controls = row![
            button(text("Save All").color(Color::WHITE)).on_press(SettingsMessage::SaveAll).padding(10).style(iced::theme::Button::Primary),
            button(text("Cancel").color(Color::WHITE)).on_press(SettingsMessage::CancelAll).padding(10).style(iced::theme::Button::Destructive),
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
    info!("settings module loaded");
}
