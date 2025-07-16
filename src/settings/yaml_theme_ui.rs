use iced::{
    widget::{column, row, text, button, text_input, scrollable},
    Element, Length, Color, alignment,
};
use std::collections::HashMap;
use crate::config::yaml_theme::{YamlTheme, parse_color, color_to_hex, YamlThemeError};
use log::info;

#[derive(Debug, Clone)]
pub enum YamlThemeUiMessage {
    NameChanged(String),
    DescriptionChanged(String),
    ColorChanged(String, String), // (color_key, new_hex_value)
    SyntaxHighlightingChanged(String, String),
    UiElementChanged(String, String),
    AddColorField(String), // (field_type: "colors", "syntax_highlighting", "ui_elements")
    RemoveColorField(String, String), // (field_type, color_key)
    Save,
    Cancel,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct YamlThemeUi {
    theme: YamlTheme,
    original_theme: YamlTheme, // To allow reverting changes
    error_message: Option<String>,
}

impl YamlThemeUi {
    pub fn new(theme: YamlTheme) -> Self {
        Self {
            theme: theme.clone(),
            original_theme: theme,
            error_message: None,
        }
    }

    pub fn update(&mut self, message: YamlThemeUiMessage) {
        self.error_message = None; // Clear previous errors
        match message {
            YamlThemeUiMessage::NameChanged(name) => {
                self.theme.name = name;
            }
            YamlThemeUiMessage::DescriptionChanged(desc) => {
                self.theme.description = desc;
            }
            YamlThemeUiMessage::ColorChanged(key, value) => {
                if parse_color(&value).is_ok() {
                    self.theme.colors.insert(key, value);
                } else {
                    self.error_message = Some(format!("Invalid color format for '{}'", key));
                }
            }
            YamlThemeUiMessage::SyntaxHighlightingChanged(key, value) => {
                if parse_color(&value).is_ok() {
                    self.theme.syntax_highlighting.insert(key, value);
                } else {
                    self.error_message = Some(format!("Invalid color format for syntax highlighting '{}'", key));
                }
            }
            YamlThemeUiMessage::UiElementChanged(key, value) => {
                if parse_color(&value).is_ok() {
                    self.theme.ui_elements.insert(key, value);
                } else {
                    self.error_message = Some(format!("Invalid color format for UI element '{}'", key));
                }
            }
            YamlThemeUiMessage::AddColorField(field_type) => {
                let new_key = format!("new_color_{}", uuid::Uuid::new_v4().to_string()[..4].to_string());
                match field_type.as_str() {
                    "colors" => { self.theme.colors.insert(new_key, "#FFFFFF".to_string()); },
                    "syntax_highlighting" => { self.theme.syntax_highlighting.insert(new_key, "#FFFFFF".to_string()); },
                    "ui_elements" => { self.theme.ui_elements.insert(new_key, "#FFFFFF".to_string()); },
                    _ => {}
                }
            }
            YamlThemeUiMessage::RemoveColorField(field_type, key) => {
                match field_type.as_str() {
                    "colors" => { self.theme.colors.remove(&key); },
                    "syntax_highlighting" => { self.theme.syntax_highlighting.remove(&key); },
                    "ui_elements" => { self.theme.ui_elements.remove(&key); },
                    _ => {}
                }
            }
            YamlThemeUiMessage::Save => {
                match self.theme.validate() {
                    Ok(_) => {
                        self.original_theme = self.theme.clone();
                        info!("Theme saved (mock).");
                        // In a real app, you'd persist self.theme
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Validation Error: {}", e));
                    }
                }
            }
            YamlThemeUiMessage::Cancel => {
                self.theme = self.original_theme.clone();
                info!("Theme changes cancelled.");
            }
            YamlThemeUiMessage::Error(msg) => {
                self.error_message = Some(msg);
            }
        }
    }

    pub fn view(&self) -> Element<YamlThemeUiMessage> {
        let header = text(format!("Editing Theme: {}", self.theme.name)).size(24).color(Color::BLACK);

        let name_input = row![
            text("Name:").width(Length::Fixed(100.0)),
            text_input("Theme Name", &self.theme.name)
                .on_input(YamlThemeUiMessage::NameChanged)
                .width(Length::Fill)
        ].spacing(10);

        let description_input = row![
            text("Description:").width(Length::Fixed(100.0)),
            text_input("Theme Description", &self.theme.description)
                .on_input(YamlThemeUiMessage::DescriptionChanged)
                .width(Length::Fill)
        ].spacing(10);

        let color_section = |title: &str, map: &HashMap<String, String>, field_type: &str| {
            let color_fields: Vec<Element<YamlThemeUiMessage>> = map.iter()
                .map(|(key, value)| {
                    let current_color = parse_color(value).unwrap_or_default();
                    let preview_color = iced::Color::from_rgb(current_color.r, current_color.g, current_color.b);
                    row![
                        text(key).width(Length::Fixed(120.0)),
                        text_input("Hex or RGB", value)
                            .on_input(move |s| match field_type {
                                "colors" => YamlThemeUiMessage::ColorChanged(key.clone(), s),
                                "syntax_highlighting" => YamlThemeUiMessage::SyntaxHighlightingChanged(key.clone(), s),
                                "ui_elements" => YamlThemeUiMessage::UiElementChanged(key.clone(), s),
                                _ => unreachable!(),
                            })
                            .width(Length::FillPortion(0.6)),
                        container()
                            .width(Length::Fixed(30.0))
                            .height(Length::Fixed(30.0))
                            .style(iced::widget::container::Appearance {
                                background: Some(iced::Background::Color(preview_color)),
                                border_radius: 4.0.into(),
                                border_width: 1.0,
                                border_color: Color::BLACK,
                                ..Default::default()
                            }),
                        button(text("X"))
                            .on_press(YamlThemeUiMessage::RemoveColorField(field_type.to_string(), key.clone()))
                            .padding(5)
                    ].spacing(5).align_items(alignment::Horizontal::Center).into()
                })
                .collect();

            column![
                row![
                    text(title).size(18),
                    button(text("Add Field"))
                        .on_press(YamlThemeUiMessage::AddColorField(field_type.to_string()))
                        .padding(5)
                ].spacing(10),
                column(color_fields).spacing(5)
            ].spacing(10).into()
        };

        let colors_section = color_section("Colors", &self.theme.colors, "colors");
        let syntax_section = color_section("Syntax Highlighting", &self.theme.syntax_highlighting, "syntax_highlighting");
        let ui_section = color_section("UI Elements", &self.theme.ui_elements, "ui_elements");

        let error_display = if let Some(msg) = &self.error_message {
            text(msg).color(Color::RED).size(14).into()
        } else {
            column![].into()
        };

        let controls = row![
            button(text("Save")).on_press(YamlThemeUiMessage::Save).padding(10),
            button(text("Cancel")).on_press(YamlThemeUiMessage::Cancel).padding(10),
        ].spacing(10);

        scrollable(
            column![
                header,
                name_input,
                description_input,
                colors_section,
                syntax_section,
                ui_section,
                error_display,
                controls,
            ]
            .spacing(20)
            .padding(20)
        )
        .into()
    }
}

pub fn init() {
    info!("YAML theme UI module loaded");
}
