use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use iced;
use anyhow::Result;
use thiserror::Error;
use log::info;

use crate::config::{ThemeConfig, ColorScheme, ColorValue, AnsiColors, Typography, Effects, Spacing};
use super::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlTheme {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub syntax_highlighting: HashMap<String, String>,
    #[serde(default)]
    pub ui_elements: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum YamlThemeError {
    #[error("Invalid color format: {0}")]
    InvalidColorFormat(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

impl YamlTheme {
    pub fn from_yaml(yaml: &str) -> Result<Self, YamlThemeError> {
        let theme: YamlTheme = serde_yaml::from_str(yaml)?;
        theme.validate()?;
        Ok(theme)
    }

    pub fn to_yaml(&self) -> Result<String, YamlThemeError> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn validate(&self) -> Result<(), YamlThemeError> {
        if self.name.is_empty() {
            return Err(YamlThemeError::MissingField("name".to_string()));
        }
        for (key, value) in &self.colors {
            parse_color(value).map_err(|_| YamlThemeError::InvalidColorFormat(key.clone()))?;
        }
        for (key, value) in &self.syntax_highlighting {
            parse_color(value).map_err(|_| YamlThemeError::InvalidColorFormat(key.clone()))?;
        }
        for (key, value) in &self.ui_elements {
            parse_color(value).map_err(|_| YamlThemeError::InvalidColorFormat(key.clone()))?;
        }
        Ok(())
    }

    /// Convert to internal ThemeConfig
    pub fn to_theme_config(&self) -> Result<ThemeConfig, YamlThemeError> {
        let colors = ColorScheme {
            background: parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string()))?,
            surface: self.derive_surface_color()?,
            surface_variant: self.derive_surface_variant_color()?,
            
            text: parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?,
            text_secondary: self.derive_text_secondary()?,
            text_disabled: self.derive_text_disabled()?,
            
            terminal_background: parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string()))?,
            terminal_foreground: parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?,
            terminal_cursor: parse_color(&self.colors.get("cursor").unwrap_or(&"#FFFFFF".to_string())).unwrap_or_default(),
            terminal_selection: parse_color(&self.colors.get("selection").unwrap_or(&"#555555".to_string())).unwrap_or_default(),
            
            ansi_colors: AnsiColors {
                black: parse_color(&self.colors.get("black").unwrap_or(&"#000000".to_string()))?,
                red: parse_color(&self.colors.get("red").unwrap_or(&"#CD3131".to_string()))?,
                green: parse_color(&self.colors.get("green").unwrap_or(&"#0BCB0B".to_string()))?,
                yellow: parse_color(&self.colors.get("yellow").unwrap_or(&"#E5E510".to_string()))?,
                blue: parse_color(&self.colors.get("blue").unwrap_or(&"#2472C8".to_string()))?,
                magenta: parse_color(&self.colors.get("magenta").unwrap_or(&"#BC3FBC".to_string()))?,
                cyan: parse_color(&self.colors.get("cyan").unwrap_or(&"#0ADBBF".to_string()))?,
                white: parse_color(&self.colors.get("white").unwrap_or(&"#E5E5E5".to_string()))?,
                
                bright_black: parse_color(&self.colors.get("bright_black").unwrap_or(&"#666666".to_string()))?,
                bright_red: parse_color(&self.colors.get("bright_red").unwrap_or(&"#F14C4C".to_string()))?,
                bright_green: parse_color(&self.colors.get("bright_green").unwrap_or(&"#17A717".to_string()))?,
                bright_yellow: parse_color(&self.colors.get("bright_yellow").unwrap_or(&"#F5F543".to_string()))?,
                bright_blue: parse_color(&self.colors.get("bright_blue").unwrap_or(&"#3B8EEA".to_string()))?,
                bright_magenta: parse_color(&self.colors.get("bright_magenta").unwrap_or(&"#D670D6".to_string()))?,
                bright_cyan: parse_color(&self.colors.get("bright_cyan").unwrap_or(&"#1ADCEF".to_string()))?,
                bright_white: parse_color(&self.colors.get("bright_white").unwrap_or(&"#FFFFFF".to_string()))?,
            },
            
            primary: parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?,
            secondary: parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string()))?,
            accent: parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?,
            success: parse_color(&self.colors.get("green").unwrap_or(&"#0BCB0B".to_string()))?,
            warning: parse_color(&self.colors.get("yellow").unwrap_or(&"#E5E510".to_string()))?,
            error: parse_color(&self.colors.get("red").unwrap_or(&"#CD3131".to_string()))?,
            
            hover: self.derive_hover_color()?,
            active: self.derive_active_color()?,
            focus: self.derive_focus_color()?,
            disabled: self.derive_disabled_color()?,
            
            border: parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?,
            divider: self.derive_divider_color()?,
        };

        let typography = Typography {
            font_family: "".to_string(),
            font_size: 0.0,
            line_height: 0.0,
            letter_spacing: 0.0,
            ..Typography::default()
        };

        let effects = Effects {
            border_radius: 0.0,
            shadow_blur: 0.0,
            shadow_offset: (0.0, 0.0),
            ..Effects::default()
        };

        Ok(ThemeConfig {
            name: self.name.clone(),
            colors,
            typography,
            spacing: Spacing::default(),
            effects,
            custom_themes: HashMap::new(),
        })
    }

    /// Create from internal ThemeConfig
    pub fn from_theme_config(theme: &ThemeConfig) -> Self {
        Self {
            name: theme.name.clone(),
            description: "".to_string(),
            colors: theme.colors.iter().map(|(k, v)| (k.clone(), color_to_hex(*v))).collect(),
            syntax_highlighting: HashMap::new(),
            ui_elements: HashMap::new(),
        }
    }

    /// Helper methods for deriving colors
    fn derive_surface_color(&self) -> Result<ColorValue, YamlThemeError> {
        let bg = parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string()))?;
        Ok(if self.is_dark_theme() {
            lighten_color(bg, 0.05)
        } else {
            darken_color(bg, 0.02)
        })
    }

    fn derive_surface_variant_color(&self) -> Result<ColorValue, YamlThemeError> {
        let bg = parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string()))?;
        Ok(if self.is_dark_theme() {
            lighten_color(bg, 0.1)
        } else {
            darken_color(bg, 0.05)
        })
    }

    fn derive_text_secondary(&self) -> Result<ColorValue, YamlThemeError> {
        let fg = parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?;
        Ok(ColorValue {
            a: 0.7,
            ..fg
        })
    }

    fn derive_text_disabled(&self) -> Result<ColorValue, YamlThemeError> {
        let fg = parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?;
        Ok(ColorValue {
            a: 0.5,
            ..fg
        })
    }

    fn derive_hover_color(&self) -> Result<ColorValue, YamlThemeError> {
        Ok(if self.is_dark_theme() {
            ColorValue { r: 1.0, g: 1.0, b: 1.0, a: 0.1 }
        } else {
            ColorValue { r: 0.0, g: 0.0, b: 0.0, a: 0.05 }
        })
    }

    fn derive_active_color(&self) -> Result<ColorValue, YamlThemeError> {
        Ok(if self.is_dark_theme() {
            ColorValue { r: 1.0, g: 1.0, b: 1.0, a: 0.2 }
        } else {
            ColorValue { r: 0.0, g: 0.0, b: 0.0, a: 0.1 }
        })
    }

    fn derive_focus_color(&self) -> Result<ColorValue, YamlThemeError> {
        let accent = parse_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string()))?;
        Ok(ColorValue {
            a: 0.5,
            ..accent
        })
    }

    fn derive_disabled_color(&self) -> Result<ColorValue, YamlThemeError> {
        Ok(ColorValue { r: 0.5, g: 0.5, b: 0.5, a: 0.5 })
    }

    fn derive_divider_color(&self) -> Result<ColorValue, YamlThemeError> {
        let bg = parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string()))?;
        Ok(if self.is_dark_theme() {
            lighten_color(bg, 0.15)
        } else {
            darken_color(bg, 0.15)
        })
    }

    fn is_dark_theme(&self) -> bool {
        if let Ok(bg) = parse_color(&self.colors.get("background").unwrap_or(&"#000000".to_string())) {
            // Calculate luminance
            let luminance = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
            luminance < 0.5
        } else {
            true // Default to dark
        }
    }

    pub fn to_iced_theme(&self) -> crate::config::theme::Theme {
        crate::config::theme::Theme {
            background: parse_hex_color(&self.colors.get("background").unwrap_or(&"#000000".to_string())).unwrap_or(iced::Color::BLACK),
            foreground: parse_hex_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string())).unwrap_or(iced::Color::WHITE),
            primary: parse_hex_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string())).unwrap_or(iced::Color::BLUE),
            secondary: parse_hex_color(&self.colors.get("background").unwrap_or(&"#646464".to_string())).unwrap_or(iced::Color::from_rgb8(100, 100, 100)),
            danger: parse_hex_color(&self.colors.get("red").unwrap_or(&"#CD3131".to_string())).unwrap_or(iced::Color::RED),
            text: parse_hex_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string())).unwrap_or(iced::Color::BLACK),
            border: parse_hex_color(&self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string())).unwrap_or(iced::Color::from_rgb8(200, 200, 200)),
        }
    }

    pub fn load_from_str(s: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(s)?)
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }
}

impl From<Theme> for YamlTheme {
    fn from(theme: Theme) -> Self {
        Self {
            name: theme.name,
            description: theme.description,
            colors: theme.colors,
            syntax_highlighting: theme.syntax_highlighting,
            ui_elements: theme.ui_elements,
        }
    }
}

/// Parse color from various formats (hex, rgb, hsl, named)
fn parse_color(hex_color: &str) -> Result<iced::Color, String> {
    if hex_color.starts_with('#') && hex_color.len() == 7 {
        let r = u8::from_str_radix(&hex_color[1..3], 16).map_err(|_| "Invalid red component".to_string())?;
        let g = u8::from_str_radix(&hex_color[3..5], 16).map_err(|_| "Invalid green component".to_string())?;
        let b = u8::from_str_radix(&hex_color[5..7], 16).map_err(|_| "Invalid blue component".to_string())?;
        Ok(iced::Color::from_rgb8(r, g, b))
    } else {
        Err("Invalid color format. Use #RRGGBB".to_string())
    }
}

/// Convert ColorValue to hex string
fn color_to_hex(color: iced::Color) -> String {
    format!("#{:02X}{:02X}{:02X}", (color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8)
}

/// Lighten a color by a factor
fn lighten_color(color: iced::Color, factor: f32) -> iced::Color {
    iced::Color {
        r: (color.r + (1.0 - color.r) * factor).clamp(0.0, 1.0),
        g: (color.g + (1.0 - color.g) * factor).clamp(0.0, 1.0),
        b: (color.b + (1.0 - color.b) * factor).clamp(0.0, 1.0),
        a: color.a,
    }
}

/// Darken a color by a factor
fn darken_color(color: iced::Color, factor: f32) -> iced::Color {
    iced::Color {
        r: (color.r * (1.0 - factor)).clamp(0.0, 1.0),
        g: (color.g * (1.0 - factor)).clamp(0.0, 1.0),
        b: (color.b * (1.0 - factor)).clamp(0.0, 1.0),
        a: color.a,
    }
}

pub fn init() {
    info!("config/yaml_theme module loaded");
}
