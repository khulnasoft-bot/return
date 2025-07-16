use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::info;
use iced::{Color, Theme as IcedTheme};

/// Represents a complete theme for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YamlTheme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub selection_color: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub info_color: Color,
    pub success_color: Color,
    pub terminal_colors: TerminalColors,
    // Add more theme-specific properties as needed
}

/// Represents the 16 ANSI terminal colors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalColors {
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
}

impl Default for YamlTheme {
    fn default() -> Self {
        // A simple light theme as default
        Self {
            name: "Default Light".to_string(),
            background: Color::from_rgb(0.95, 0.95, 0.95), // Light gray
            foreground: Color::BLACK,
            primary: Color::from_rgb(0.2, 0.6, 0.8), // Blue
            secondary: Color::from_rgb(0.8, 0.4, 0.2), // Orange
            accent: Color::from_rgb(0.6, 0.8, 0.2), // Green
            text_color: Color::BLACK,
            border_color: Color::from_rgb(0.8, 0.8, 0.8),
            selection_color: Color::from_rgba(0.2, 0.6, 0.8, 0.2),
            error_color: Color::from_rgb(0.8, 0.2, 0.2),
            warning_color: Color::from_rgb(0.8, 0.6, 0.2),
            info_color: Color::from_rgb(0.2, 0.4, 0.8),
            success_color: Color::from_rgb(0.2, 0.8, 0.2),
            terminal_colors: TerminalColors::default(),
        }
    }
}

impl Default for TerminalColors {
    fn default() -> Self {
        // Standard ANSI colors
        Self {
            black: Color::from_rgb(0.0, 0.0, 0.0),
            red: Color::from_rgb(0.8, 0.0, 0.0),
            green: Color::from_rgb(0.0, 0.8, 0.0),
            yellow: Color::from_rgb(0.8, 0.8, 0.0),
            blue: Color::from_rgb(0.0, 0.0, 0.8),
            magenta: Color::from_rgb(0.8, 0.0, 0.8),
            cyan: Color::from_rgb(0.0, 0.8, 0.8),
            white: Color::from_rgb(0.8, 0.8, 0.8),
            bright_black: Color::from_rgb(0.3, 0.3, 0.3),
            bright_red: Color::from_rgb(1.0, 0.0, 0.0),
            bright_green: Color::from_rgb(0.0, 1.0, 0.0),
            bright_yellow: Color::from_rgb(1.0, 1.0, 0.0),
            bright_blue: Color::from_rgb(0.0, 0.0, 1.0),
            bright_magenta: Color::from_rgb(1.0, 0.0, 1.0),
            bright_cyan: Color::from_rgb(0.0, 1.0, 1.0),
            bright_white: Color::from_rgb(1.0, 1.0, 1.0),
        }
    }
}

/// Represents a simplified theme structure for Iced GUI.
/// This struct acts as an intermediary to convert `YamlTheme` into `iced::Theme`.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub iced_theme: IcedTheme,
    // Add other theme-related properties here if needed, e.g., terminal colors
    // pub terminal_colors: HashMap<String, Color>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            iced_theme: IcedTheme::Light, // Or any other default Iced theme
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
        assert_eq!(theme.iced_theme, IcedTheme::Light);
    }

    #[test]
    fn test_terminal_colors_default() {
        let term_colors = TerminalColors::default();
        assert_eq!(term_colors.red, Color::from_rgb(0.8, 0.0, 0.0));
        assert_eq!(term_colors.bright_blue, Color::from_rgb(0.0, 0.0, 1.0));
    }
}

pub fn init() {
    info!("config/theme module loaded");
}
