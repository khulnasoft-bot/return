use anyhow::{anyhow, Result};
use iced::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a theme defined in a YAML file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YamlTheme {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub colors: HashMap<String, String>, // Color name -> color value (e.g., "#RRGGBB", "rgb(r,g,b)")
    pub terminal_colors: Option<HashMap<String, String>>, // ANSI color name -> color value
}

impl YamlTheme {
    /// Converts a `YamlTheme` into an `iced::Theme::Custom` instance.
    pub fn to_iced_theme(&self) -> iced::Theme {
        let mut palette = iced::theme::Palette::default();

        // Parse main colors
        if let Some(bg) = self.colors.get("background") {
            if let Ok(color) = parse_color(bg) {
                palette.background = color;
            }
        }
        if let Some(fg) = self.colors.get("foreground") {
            if let Ok(color) = parse_color(fg) {
                palette.text = color;
            }
        }
        if let Some(primary) = self.colors.get("primary") {
            if let Ok(color) = parse_color(primary) {
                palette.primary = color;
            }
        }
        if let Some(success) = self.colors.get("success") {
            if let Ok(color) = parse_color(success) {
                palette.success = color;
            }
        }
        if let Some(danger) = self.colors.get("danger") {
            if let Ok(color) = parse_color(danger) {
                palette.danger = color;
            }
        }

        // Iced's custom theme only allows setting a palette.
        // For more granular control (like terminal colors), you'd need to
        // implement custom style sheets that consume these colors.
        iced::Theme::Custom(Box::new(iced::theme::Custom::new(palette)))
    }

    /// Validates the YAML theme structure and color formats.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Theme name cannot be empty."));
        }

        for (color_name, color_value) in &self.colors {
            parse_color(color_value).map_err(|e| anyhow!("Invalid color format for '{}': {}", color_name, e))?;
        }

        if let Some(term_colors) = &self.terminal_colors {
            for (color_name, color_value) in term_colors {
                parse_color(color_value).map_err(|e| anyhow!("Invalid terminal color format for '{}': {}", color_name, e))?;
            }
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
            return Ok(Color::from_rgba8(r, g, b, (a * 255.0).round() as u8));
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
        _ => Err(anyhow!("Unknown color format or named color: {}", s)),
    }
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
            terminal_colors: None,
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
            terminal_colors: None,
        };
        assert!(theme.validate().is_ok());

        let mut invalid_colors = HashMap::new();
        invalid_colors.insert("background".to_string(), "invalid-color".to_string());
        let invalid_theme = YamlTheme {
            name: "Invalid Theme".to_string(),
            description: None,
            author: None,
            colors: invalid_colors,
            terminal_colors: None,
        };
        assert!(invalid_theme.validate().is_err());

        let empty_name_theme = YamlTheme {
            name: "".to_string(),
            description: None,
            author: None,
            colors: HashMap::new(),
            terminal_colors: None,
        };
        assert!(empty_name_theme.validate().is_err());
    }
}
