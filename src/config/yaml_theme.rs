use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use log::{warn, error};
use iced::Color;

/// Represents a color value in various formats.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ColorValue {
    Hex(String),
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hsl(f32, f32, f32),
    Named(String),
}

/// Represents a theme defined in a YAML file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YamlTheme {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub colors: HashMap<String, String>, // Color name -> color value (e.g., "#RRGGBB", "rgb(r,g,b)")
    #[serde(default)]
    pub syntax_highlighting: HashMap<String, String>, // For code highlighting
    #[serde(default)]
    pub ui_elements: HashMap<String, String>, // For specific UI components
    #[serde(default)]
    pub terminal_colors: HashMap<String, String>, // ANSI color name -> color value
}

impl Default for YamlTheme {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#282828".to_string());
        colors.insert("foreground".to_string(), "#ebdbb2".to_string());
        colors.insert("primary".to_string(), "#83a598".to_string());
        colors.insert("success".to_string(), "#b8bb26".to_string());
        colors.insert("danger".to_string(), "#fb4934".to_string());

        let mut terminal_colors = HashMap::new();
        terminal_colors.insert("black".to_string(), "#282828".to_string());
        terminal_colors.insert("red".to_string(), "#cc241d".to_string());
        terminal_colors.insert("green".to_string(), "#98971a".to_string());
        terminal_colors.insert("yellow".to_string(), "#d79921".to_string());
        terminal_colors.insert("blue".to_string(), "#458588".to_string());
        terminal_colors.insert("magenta".to_string(), "#b16286".to_string());
        terminal_colors.insert("cyan".to_string(), "#689d6a".to_string());
        terminal_colors.insert("white".to_string(), "#a89984".to_string());
        terminal_colors.insert("bright_black".to_string(), "#928374".to_string());
        terminal_colors.insert("bright_red".to_string(), "#fb4934".to_string());
        terminal_colors.insert("bright_green".to_string(), "#b8bb26".to_string());
        terminal_colors.insert("bright_yellow".to_string(), "#fabd2f".to_string());
        terminal_colors.insert("bright_blue".to_string(), "#83a598".to_string());
        terminal_colors.insert("bright_magenta".to_string(), "#d3869b".to_string());
        terminal_colors.insert("bright_cyan".to_string(), "#8ec07c".to_string());
        terminal_colors.insert("bright_white".to_string(), "#ebdbb2".to_string());
        terminal_colors.insert("cursor".to_string(), "#ebdbb2".to_string());
        terminal_colors.insert("selection".to_string(), "#504945".to_string());


        Self {
            name: "Default Theme".to_string(),
            description: Some("A default dark theme.".to_string()),
            author: Some("NeoTerm".to_string()),
            colors,
            syntax_highlighting: HashMap::new(), // Default empty
            ui_elements: HashMap::new(), // Default empty
            terminal_colors,
        }
    }
}

impl YamlTheme {
    /// Converts the `YamlTheme` into an `iced::Theme::Custom`.
    pub fn to_iced_theme(&self) -> iced::Theme {
        let mut palette = iced::theme::Palette {
            background: parse_color(self.colors.get("background").unwrap_or(&"#000000".to_string())).unwrap_or(Color::BLACK),
            text: parse_color(self.colors.get("foreground").unwrap_or(&"#FFFFFF".to_string())).unwrap_or(Color::WHITE),
            primary: parse_color(self.colors.get("primary").unwrap_or(&"#83a598".to_string())).unwrap_or(Color::from_rgb(0.5, 0.5, 0.5)),
            success: parse_color(self.colors.get("success").unwrap_or(&"#b8bb26".to_string())).unwrap_or(Color::from_rgb(0.0, 1.0, 0.0)),
            danger: parse_color(self.colors.get("danger").unwrap_or(&"#fb4934".to_string())).unwrap_or(Color::from_rgb(1.0, 0.0, 0.0)),
        };

        // You can extend this to map more specific colors to iced's palette or custom styles
        // For example, mapping terminal_colors to specific UI elements if needed.

        iced::Theme::Custom(Box::new(iced::theme::Custom::new(palette)))
    }

    /// Validates the theme configuration.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Theme name cannot be empty."));
        }

        for (key, color_str) in &self.colors {
            parse_color(color_str).map_err(|e| anyhow!("Invalid color value for main color '{}': {}", key, e))?;
        }
        for (key, color_str) in &self.syntax_highlighting {
            parse_color(color_str).map_err(|e| anyhow!("Invalid color value for syntax highlighting '{}': {}", key, e))?;
        }
        for (key, color_str) in &self.ui_elements {
            parse_color(color_str).map_err(|e| anyhow!("Invalid color value for UI element '{}': {}", key, e))?;
        }
        for (key, color_str) in &self.terminal_colors {
            parse_color(color_str).map_err(|e| anyhow!("Invalid color value for terminal color '{}': {}", key, e))?;
        }

        Ok(())
    }
}

/// Parses a color string into an `iced::Color`.
/// Supports hex (#RRGGBB, #RRGGBBAA), rgb(r,g,b), rgba(r,g,b,a), hsl(h,s,l), and common named colors.
pub fn parse_color(color_str: &str) -> Result<Color> {
    let s = color_str.trim();

    // Hex colors
    if s.starts_with('#') {
        let hex = &s[1..];
        return match hex.len() {
            6 => Color::from_rgb_hex(hex).map_err(|_| anyhow!("Invalid hex color: {}", s)),
            8 => Color::from_rgba_hex(hex).map_err(|_| anyhow!("Invalid hex color: {}", s)),
            _ => Err(anyhow!("Invalid hex color length: {}", s)),
        };
    }

    // RGB/RGBA
    if s.starts_with("rgb(") || s.starts_with("rgba(") {
        let parts: Vec<&str> = s
            .trim_start_matches("rgb(")
            .trim_start_matches("rgba(")
            .trim_end_matches(')')
            .split(',')
            .map(|p| p.trim())
            .collect();

        if parts.len() == 3 {
            let r = parts[0].parse::<u8>()?;
            let g = parts[1].parse::<u8>()?;
            let b = parts[2].parse::<u8>()?;
            return Ok(Color::from_rgb8(r, g, b));
        } else if parts.len() == 4 {
            let r = parts[0].parse::<u8>()?;
            let g = parts[1].parse::<u8>()?;
            let b = parts[2].parse::<u8>()?;
            let a = parts[3].parse::<f32>()?; // Alpha is 0.0-1.0
            return Ok(Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a));
        } else {
            return Err(anyhow!("Invalid RGB/RGBA format: {}", s));
        }
    }

    // HSL (simplified, Iced doesn't have direct HSL constructor for Color)
    // This would require converting HSL to RGB. For now, we'll just error.
    if s.starts_with("hsl(") {
        return Err(anyhow!("HSL color format not directly supported by iced::Color: {}", s));
    }

    // Named colors (a very small subset for demonstration)
    match s.to_lowercase().as_str() {
        "black" => Ok(Color::BLACK),
        "white" => Ok(Color::WHITE),
        "red" => Ok(Color::from_rgb(1.0, 0.0, 0.0)),
        "green" => Ok(Color::from_rgb(0.0, 1.0, 0.0)),
        "blue" => Ok(Color::from_rgb(0.0, 0.0, 1.0)),
        "yellow" => Ok(Color::from_rgb(1.0, 1.0, 0.0)),
        "cyan" => Ok(Color::from_rgb(0.0, 1.0, 1.0)),
        "magenta" => Ok(Color::from_rgb(1.0, 0.0, 1.0)),
        "gray" => Ok(Color::from_rgb(0.5, 0.5, 0.5)),
        "lightgray" => Ok(Color::from_rgb(0.8, 0.8, 0.8)),
        "darkgray" => Ok(Color::from_rgb(0.3, 0.3, 0.3)),
        "transparent" => Ok(Color::TRANSPARENT),
        _ => Err(anyhow!("Unknown color format or named color: {}", s)),
    }
}

/// Converts an `iced::Color` to a hex string (#RRGGBB).
pub fn color_to_hex(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}",
            (color.r * 255.0).round() as u8,
            (color.g * 255.0).round() as u8,
            (color.b * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_color_hex_rgb() {
        assert_eq!(parse_color("#FF0000").unwrap(), Color::from_rgb(1.0, 0.0, 0.0));
        assert_eq!(parse_color("#00FF00").unwrap(), Color::from_rgb(0.0, 1.0, 0.0));
        assert_eq!(parse_color("#0000FF").unwrap(), Color::from_rgb(0.0, 0.0, 1.0));
        assert_eq!(parse_color("#123456").unwrap(), Color::from_rgb8(0x12, 0x34, 0x56));
    }

    #[test]
    fn test_parse_color_hex_rgba() {
        assert_eq!(parse_color("#FF000080").unwrap(), Color::from_rgba(1.0, 0.0, 0.0, 0.5));
        assert_eq!(parse_color("#000000FF").unwrap(), Color::BLACK);
        assert_eq!(parse_color("#FFFFFF00").unwrap(), Color::from_rgba(1.0, 1.0, 1.0, 0.0));
    }

    #[test]
    fn test_parse_color_rgb() {
        assert_eq!(parse_color("rgb(255,0,0)").unwrap(), Color::from_rgb(1.0, 0.0, 0.0));
        assert_eq!(parse_color("rgb(0, 255, 0)").unwrap(), Color::from_rgb(0.0, 1.0, 0.0));
        assert_eq!(parse_color("rgb(10,20,30)").unwrap(), Color::from_rgb8(10, 20, 30));
    }

    #[test]
    fn test_parse_color_rgba() {
        assert_eq!(parse_color("rgba(255,0,0,0.5)").unwrap(), Color::from_rgba(1.0, 0.0, 0.0, 0.5));
        assert_eq!(parse_color("rgba(0, 0, 0, 1.0)").unwrap(), Color::BLACK);
        assert_eq!(parse_color("rgba(255, 255, 255, 0.0)").unwrap(), Color::from_rgba(1.0, 1.0, 1.0, 0.0));
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("black").unwrap(), Color::BLACK);
        assert_eq!(parse_color("WHITE").unwrap(), Color::WHITE);
        assert_eq!(parse_color("red").unwrap(), Color::from_rgb(1.0, 0.0, 0.0));
        assert_eq!(parse_color("lightgray").unwrap(), Color::from_rgb(0.8, 0.8, 0.8));
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("#FF00").is_err()); // Too short hex
        assert!(parse_color("rgb(255,0)").is_err()); // Too few rgb components
        assert!(parse_color("rgba(255,0,0)").is_err()); // Too few rgba components
        assert!(parse_color("hsl(10, 50%, 50%)").is_err()); // HSL not supported
        assert!(parse_color("unknowncolor").is_err());
    }

    #[test]
    fn test_yaml_theme_to_iced_theme() {
        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#1E1E1E".to_string());
        colors.insert("foreground".to_string(), "#D4D4D4".to_string());
        colors.insert("primary".to_string(), "rgb(0, 122, 204)".to_string());
        colors.insert("success".to_string(), "green".to_string());

        let theme = YamlTheme {
            name: "Test Theme".to_string(),
            description: None,
            author: None,
            colors,
            syntax_highlighting: HashMap::new(),
            ui_elements: HashMap::new(),
            terminal_colors: HashMap::new(),
        };

        let iced_theme = theme.to_iced_theme();
        if let iced::Theme::Custom(custom_theme) = iced_theme {
            assert_eq!(custom_theme.palette().background, Color::from_rgb8(0x1E, 0x1E, 0x1E));
            assert_eq!(custom_theme.palette().text, Color::from_rgb8(0xD4, 0xD4, 0xD4));
            assert_eq!(custom_theme.palette().primary, Color::from_rgb8(0, 122, 204));
            assert_eq!(custom_theme.palette().success, Color::from_rgb(0.0, 1.0, 0.0));
        } else {
            panic!("Expected a custom theme");
        }
    }

    #[test]
    fn test_yaml_theme_validate() {
        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#1E1E1E".to_string());
        let theme = YamlTheme {
            name: "Valid Theme".to_string(),
            description: None,
            author: None,
            colors,
            syntax_highlighting: HashMap::new(),
            ui_elements: HashMap::new(),
            terminal_colors: HashMap::new(),
        };
        assert!(theme.validate().is_ok());

        let mut invalid_colors = HashMap::new();
        invalid_colors.insert("background".to_string(), "invalid-color".to_string());
        let invalid_theme = YamlTheme {
            name: "Invalid Theme".to_string(),
            description: None,
            author: None,
            colors: invalid_colors,
            syntax_highlighting: HashMap::new(),
            ui_elements: HashMap::new(),
            terminal_colors: HashMap::new(),
        };
        assert!(invalid_theme.validate().is_err());

        let empty_name_theme = YamlTheme {
            name: "".to_string(),
            description: None,
            author: None,
            colors: HashMap::new(),
            syntax_highlighting: HashMap::new(),
            ui_elements: HashMap::new(),
            terminal_colors: HashMap::new(),
        };
        assert!(empty_name_theme.validate().is_err());
    }
}
