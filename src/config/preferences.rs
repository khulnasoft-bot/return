use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use tokio::fs;
use super::CONFIG_DIR;
use log::{info, error};
use serde_yaml;

const PREFERENCES_FILE: &str = "preferences.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub general: GeneralPreferences,
    pub terminal: TerminalPreferences,
    pub editor: EditorPreferences,
    pub ui: UiPreferences,
    pub performance: PerformancePreferences,
    pub privacy: PrivacyPreferences,
    pub ai: AiPreferences,
    pub plugins: PluginConfig,
    pub keybindings: KeyBindings,
    pub workflow_engine: WorkflowEnginePreferences,
    pub integrations: IntegrationPreferences,
    pub development: DevelopmentPreferences,
    pub env_profiles: EnvironmentProfiles,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralPreferences {
    pub startup_behavior: StartupBehavior,
    pub default_shell: Option<String>,
    pub working_directory: WorkingDirectoryBehavior,
    pub auto_update: bool,
    pub telemetry_enabled: bool,
    pub crash_reporting: bool,
    pub confirm_exit: bool,
    pub auto_update_check: bool,
    pub default_environment_profile: Option<String>,
    pub font_size: u16,
    pub theme_name: String,
    pub enable_animations: bool,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            startup_behavior: StartupBehavior::NewSession,
            default_shell: None,
            working_directory: WorkingDirectoryBehavior::Home,
            auto_update: true,
            telemetry_enabled: false,
            crash_reporting: true,
            confirm_exit: true,
            auto_update_check: true,
            default_environment_profile: None,
            font_size: 14,
            theme_name: "gruvbox-dark".to_string(), // Default theme
            enable_animations: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPreferences {
    pub font_size: u16,
    pub font_family: String,
    pub terminal_rows: u16,
    pub terminal_cols: u16,
    pub enable_ligatures: bool,
    pub scrollback_lines: usize,
    pub default_working_directory: Option<String>,
    pub enable_transparency: bool,
    pub transparency_level: f32,
    pub enable_bell: bool,
    pub paste_on_middle_click: bool,
    pub scroll_sensitivity: f32,
    pub mouse_reporting: bool,
    pub copy_on_select: bool,
    pub paste_on_right_click: bool,
    pub confirm_before_closing: bool,
    pub bell_behavior: BellBehavior,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub word_separators: String,
    pub url_detection: bool,
    pub hyperlink_behavior: HyperlinkBehavior,
    pub shell: String,
}

impl Default for TerminalPreferences {
    fn default() -> Self {
        Self {
            font_size: 14,
            font_family: "Fira Code".to_string(),
            terminal_rows: 24,
            terminal_cols: 80,
            enable_ligatures: true,
            scrollback_lines: 1000,
            default_working_directory: None,
            enable_transparency: false,
            transparency_level: 0.9,
            enable_bell: true,
            paste_on_middle_click: false,
            scroll_sensitivity: 1.0,
            mouse_reporting: true,
            copy_on_select: false,
            paste_on_right_click: true,
            confirm_before_closing: true,
            bell_behavior: BellBehavior::Visual,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            word_separators: " \t\n\"'`()[]{}".to_string(),
            url_detection: true,
            hyperlink_behavior: HyperlinkBehavior::CtrlClick,
            shell: if cfg!(windows) {
                "powershell.exe".to_string()
            } else {
                "bash".to_string()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorPreferences {
    pub vim_mode: bool,
    pub auto_suggestions: bool,
    pub syntax_highlighting: bool,
    pub auto_completion: bool,
    pub bracket_matching: bool,
    pub indent_size: usize,
    pub tab_width: usize,
    pub insert_spaces: bool,
    pub trim_whitespace: bool,
    pub auto_save: bool,
    pub word_wrap: bool,
    pub tab_size: u8,
    pub show_line_numbers: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            vim_mode: false,
            auto_suggestions: true,
            syntax_highlighting: true,
            auto_completion: true,
            bracket_matching: true,
            indent_size: 4,
            tab_width: 4,
            insert_spaces: true,
            trim_whitespace: true,
            auto_save: true,
            word_wrap: false,
            tab_size: 4,
            show_line_numbers: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme_name: String,
    pub show_tab_bar: TabBarVisibility,
    pub show_title_bar: bool,
    pub show_menu_bar: bool,
    pub compact_mode: bool,
    pub transparency: f32,
    pub blur_background: bool,
    pub animations_enabled: bool,
    pub reduce_motion: bool,
    pub high_contrast: bool,
    pub zoom_level: f32,
    pub sync_with_os_theme: bool,
    pub app_icon: String,
    pub open_new_windows_custom_size: bool,
    pub window_opacity: f32,
    pub window_blur_radius: f32,
    pub input_type: InputType,
    pub input_position: InputPosition,
    pub dim_inactive_panes: bool,
    pub focus_follows_mouse: bool,
    pub enable_fuzzy_match: bool,
    pub enable_markdown_preview: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePreferences {
    pub gpu_acceleration: bool,
    pub vsync: bool,
    pub max_fps: Option<u32>,
    pub memory_limit: Option<usize>,
    pub background_throttling: bool,
    pub lazy_rendering: bool,
    pub texture_atlas_size: u32,
    pub enable_performance_profiling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferences {
    pub history_enabled: bool,
    pub history_limit: usize,
    pub clear_history_on_exit: bool,
    pub incognito_mode: bool,
    pub log_level: LogLevel,
    pub share_usage_data: bool,
    pub redact_sensitive_info: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiPreferences {
    pub enable_ai_assistant: bool,
    pub ai_api_key: Option<String>,
    pub ai_model: String,
    pub ai_temperature: f32,
    pub ai_max_tokens: u32,
    pub ai_provider_type: String,
    pub fallback_ai_provider_type: Option<String>,
    pub fallback_ai_model: Option<String>,
    pub local_only_ai_mode: bool,
    pub enable_natural_language_detection: bool,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub model: String,
    pub enable_tool_use: bool,
    pub max_conversation_history: usize,
    pub redact_sensitive_info: bool,
    pub local_only_ai_mode: bool,
    pub fallback_provider_type: Option<String>,
    pub fallback_api_key: Option<String>,
    pub fallback_model: Option<String>,
}

impl Default for AiPreferences {
    fn default() -> Self {
        Self {
            enable_ai_assistant: true,
            ai_api_key: None,
            ai_model: "gpt-4o".to_string(),
            ai_temperature: 0.7,
            ai_max_tokens: 500,
            ai_provider_type: "openai".to_string(),
            fallback_ai_provider_type: Some("ollama".to_string()),
            fallback_ai_model: Some("llama2".to_string()),
            local_only_ai_mode: false,
            enable_natural_language_detection: false,
            provider_type: "openai".to_string(),
            api_key: None, // Should be loaded from env or config
            model: "gpt-4o".to_string(),
            enable_tool_use: true,
            max_conversation_history: 20,
            redact_sensitive_info: true,
            local_only_ai_mode: false,
            fallback_provider_type: Some("ollama".to_string()),
            fallback_api_key: None,
            fallback_model: Some("llama2".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: Vec<String>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
    pub auto_update_plugins: bool,
    pub allow_unsigned_plugins: bool,
    pub enable_plugins: bool,
    pub enable_wasm_plugins: bool,
    pub enable_lua_plugins: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub bindings: HashMap<String, KeyBinding>,
    pub keybindings_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: Vec<Modifier>,
    pub action: Action,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Super,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    SplitHorizontal,
    SplitVertical,
    CloseSplit,
    Copy,
    Paste,
    Cut,
    SelectAll,
    Find,
    FindNext,
    FindPrevious,
    ScrollUp,
    ScrollDown,
    ScrollToTop,
    ScrollToBottom,
    ToggleFullscreen,
    ToggleSettings,
    Quit,
    Command(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEnginePreferences {
    pub enable_workflow_engine: bool,
    pub enable_debugger: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences {
    pub enable_cloud_sync: bool,
    pub enable_session_sharing: bool,
    pub enable_drive_integration: bool,
    pub enable_watcher: bool,
    pub enable_websocket_server: bool,
    pub enable_cli_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentPreferences {
    pub enable_virtual_fs: bool,
    pub enable_graphql_api: bool,
    pub enable_syntax_tree: bool,
    pub enable_lpc_support: bool,
    pub enable_mcq_support: bool,
    pub enable_asset_macro: bool,
    pub enable_distribution_packaging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentProfile {
    pub name: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentProfiles {
    pub profiles: HashMap<String, EnvironmentProfile>,
    pub active_profile: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    Universal,
    Classic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputPosition {
    PinToBottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StartupBehavior {
    NewSession,
    RestoreLastSession,
    CustomCommand(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkingDirectoryBehavior {
    Home,
    LastUsed,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BellBehavior {
    None,
    Visual,
    Audio,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HyperlinkBehavior {
    Click,
    CtrlClick,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabBarVisibility {
    Always,
    WhenMultiple,
    Never,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            general: GeneralPreferences::default(),
            terminal: TerminalPreferences::default(),
            editor: EditorPreferences::default(),
            ui: UiPreferences::default(),
            performance: PerformancePreferences::default(),
            privacy: PrivacyPreferences::default(),
            ai: AiPreferences::default(),
            plugins: PluginConfig::default(),
            keybindings: KeyBindings::default(),
            workflow_engine: WorkflowEnginePreferences::default(),
            integrations: IntegrationPreferences::default(),
            development: DevelopmentPreferences::default(),
            env_profiles: EnvironmentProfiles::default(),
        }
    }
}

impl UserPreferences {
    /// Loads preferences from a YAML file, or returns default if not found/invalid.
    pub async fn load_or_default() -> Result<Self> {
        match fs::read_to_string(PREFERENCES_FILE).await {
            Ok(contents) => {
                match serde_yaml::from_str(&contents) {
                    Ok(prefs) => {
                        info!("Preferences loaded from {}", PREFERENCES_FILE);
                        Ok(prefs)
                    }
                    Err(e) => {
                        error!("Failed to parse preferences file: {}. Using default preferences.", e);
                        Ok(Self::default())
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("Preferences file not found. Creating with default preferences.");
                let default_prefs = Self::default();
                default_prefs.save().await?; // Save defaults for future use
                Ok(default_prefs)
            }
            Err(e) => {
                error!("Failed to read preferences file: {}. Using default preferences.", e);
                Ok(Self::default())
            }
        }
    }

    /// Saves the current preferences to a YAML file.
    pub async fn save(&self) -> Result<()> {
        let yaml_string = serde_yaml::to_string(self)?;
        fs::write(PREFERENCES_FILE, yaml_string)
            .await
            .map_err(|e| anyhow!("Failed to save preferences to {}: {}", PREFERENCES_FILE, e))?;
        info!("Preferences saved to {}", PREFERENCES_FILE);
        Ok(())
    }
}

pub fn init() {
    info!("config/preferences module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use std::path::Path;

    const TEST_PREFS_FILE: &str = "test_preferences.yaml";

    async fn cleanup_test_file() {
        let _ = fs::remove_file(TEST_PREFS_FILE).await;
    }

    #[tokio::test]
    async fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.general.font_size, 14);
        assert_eq!(prefs.terminal.scrollback_lines, 1000);
        assert_eq!(prefs.editor.tab_size, 4);
        assert_eq!(prefs.ai.model, "gpt-4o");
    }

    #[tokio::test]
    async fn test_user_preferences_save_and_load() {
        cleanup_test_file().await;

        let mut original_prefs = UserPreferences::default();
        original_prefs.general.font_size = 16;
        original_prefs.terminal.shell = "zsh".to_string();
        original_prefs.ai.model = "claude-3-opus-20240229".to_string();

        // Temporarily change the file name for testing
        let old_file_name = PREFERENCES_FILE;
        unsafe {
            let ptr = PREFERENCES_FILE as *const _ as *mut &str;
            *ptr = TEST_PREFS_FILE;
        }

        original_prefs.save().await.unwrap();

        let loaded_prefs = UserPreferences::load_or_default().await.unwrap();
        assert_eq!(original_prefs, loaded_prefs);

        // Restore original file name
        unsafe {
            let ptr = PREFERENCES_FILE as *const _ as *mut &str;
            *ptr = old_file_name;
        }
        cleanup_test_file().await;
    }

    #[tokio::test]
    async fn test_user_preferences_load_non_existent_file() {
        cleanup_test_file().await;

        // Temporarily change the file name for testing
        let old_file_name = PREFERENCES_FILE;
        unsafe {
            let ptr = PREFERENCES_FILE as *const _ as *mut &str;
            *ptr = TEST_PREFS_FILE;
        }

        let loaded_prefs = UserPreferences::load_or_default().await.unwrap();
        assert_eq!(loaded_prefs, UserPreferences::default());
        // Verify that a default file was created
        assert!(fs::metadata(TEST_PREFS_FILE).await.is_ok());

        // Restore original file name
        unsafe {
            let ptr = PREFERENCES_FILE as *const _ as *mut &str;
            *ptr = old_file_name;
        }
        cleanup_test_file().await;
    }

    #[tokio::test]
    async fn test_user_preferences_load_invalid_file() {
        cleanup_test_file().await;

        // Temporarily change the file name for testing
        let old_file_name = PREFERENCES_FILE;
        unsafe {
            let ptr = PREFERENCES_FILE as *const _ as *mut &str;
            *ptr = TEST_PREFS_FILE;
        }

        fs::write(TEST_PREFS_FILE, "invalid yaml content: - [").await.unwrap();

        let loaded_prefs = UserPreferences::load_or_default().await.unwrap();
        assert_eq!(loaded_prefs, UserPreferences::default());

        // Restore original file name
        unsafe {
            let ptr = PREFERENCES_FILE as *const _ as *mut &str;
            *ptr = old_file_name;
        }
        cleanup_test_file().await;
    }
}
