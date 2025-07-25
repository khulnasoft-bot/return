use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use anyhow::Result;
use log::{info, error};

use super::CONFIG_DIR;

/// Top-level preferences struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    #[serde(default)]
    pub general: GeneralPreferences,
    #[serde(default)]
    pub ui: UiPreferences,
    #[serde(default)]
    pub terminal: TerminalPreferences,
    #[serde(default)]
    pub editor: EditorPreferences,
    #[serde(default)]
    pub keybindings: KeybindingPreferences,
    #[serde(default)]
    pub ai: AiPreferences,
    #[serde(default)]
    pub privacy: PrivacyPreferences,
    #[serde(default)]
    pub performance: PerformancePreferences,
    #[serde(default)]
    pub collaboration: CollaborationPreferences,
    #[serde(default)]
    pub cloud_sync: CloudSyncPreferences,
    #[serde(default)]
    pub drive: DrivePreferences,
    #[serde(default)]
    pub workflows: WorkflowPreferences,
    #[serde(default)]
    pub indexing: IndexingPreferences,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            general: GeneralPreferences::default(),
            ui: UiPreferences::default(),
            terminal: TerminalPreferences::default(),
            editor: EditorPreferences::default(),
            keybindings: KeybindingPreferences::default(),
            ai: AiPreferences::default(),
            privacy: PrivacyPreferences::default(),
            performance: PerformancePreferences::default(),
            collaboration: CollaborationPreferences::default(),
            cloud_sync: CloudSyncPreferences::default(),
            drive: DrivePreferences::default(),
            workflows: WorkflowPreferences::default(),
            indexing: IndexingPreferences::default(),
        }
    }
}

impl UserPreferences {
    const FILE_NAME: &'static str = "preferences.yaml";

    pub async fn load() -> Result<Self> {
        let path = CONFIG_DIR.join(Self::FILE_NAME);
        if path.exists() {
            info!("Loading preferences from: {:?}", path);
            let content = fs::read_to_string(&path).await?;
            let prefs: Self = serde_yaml::from_str(&content)?;
            Ok(prefs)
        } else {
            info!("Preferences file not found at {:?}. Creating default preferences.", path);
            let default_prefs = Self::default();
            default_prefs.save().await?; // Save default preferences
            Ok(default_prefs)
        }
    }

    pub async fn save(&self) -> Result<()> {
        let path = CONFIG_DIR.join(Self::FILE_NAME);
        let content = serde_yaml::to_string(self)?;
        fs::write(&path, content).await?;
        info!("Preferences saved to: {:?}", path);
        Ok(())
    }
}

// --- Nested Preference Structs ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralPreferences {
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,
    #[serde(default = "default_telemetry_enabled")]
    pub telemetry_enabled: bool,
    #[serde(default = "default_startup_command")]
    pub startup_command: String,
}

impl Default for GeneralPreferences {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            auto_update: default_auto_update(),
            telemetry_enabled: default_telemetry_enabled(),
            startup_command: default_startup_command(),
        }
    }
}

fn default_font_size() -> u16 { 14 }
fn default_auto_update() -> bool { true }
fn default_telemetry_enabled() -> bool { true }
fn default_startup_command() -> String { "".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiPreferences {
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
    #[serde(default = "default_sync_with_os_theme")]
    pub sync_with_os_theme: bool,
    #[serde(default = "default_app_icon")]
    pub app_icon: String,
    #[serde(default = "default_open_new_windows_custom_size")]
    pub open_new_windows_custom_size: bool,
    #[serde(default = "default_window_opacity")]
    pub window_opacity: f32,
    #[serde(default = "default_window_blur_radius")]
    pub window_blur_radius: f32,
    #[serde(default = "default_input_type")]
    pub input_type: InputType,
    #[serde(default = "default_input_position")]
    pub input_position: InputPosition,
    #[serde(default = "default_dim_inactive_panes")]
    pub dim_inactive_panes: bool,
    #[serde(default = "default_focus_follows_mouse")]
    pub focus_follows_mouse: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme_name: default_theme_name(),
            sync_with_os_theme: default_sync_with_os_theme(),
            app_icon: default_app_icon(),
            open_new_windows_custom_size: default_open_new_windows_custom_size(),
            window_opacity: default_window_opacity(),
            window_blur_radius: default_window_blur_radius(),
            input_type: default_input_type(),
            input_position: default_input_position(),
            dim_inactive_panes: default_dim_inactive_panes(),
            focus_follows_mouse: default_focus_follows_mouse(),
        }
    }
}

fn default_theme_name() -> String { "nord".to_string() }
fn default_sync_with_os_theme() -> bool { false }
fn default_app_icon() -> String { "Default".to_string() }
fn default_open_new_windows_custom_size() -> bool { false }
fn default_window_opacity() -> f32 { 1.0 }
fn default_window_blur_radius() -> f32 { 0.0 }
fn default_input_type() -> InputType { InputType::Universal }
fn default_input_position() -> InputPosition { InputPosition::PinToBottom }
fn default_dim_inactive_panes() -> bool { false }
fn default_focus_follows_mouse() -> bool { false }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputType {
    Universal,
    Classic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputPosition {
    PinToBottom,
    // Other positions could be added here
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalPreferences {
    #[serde(default = "default_shell")]
    pub shell: String,
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: u32,
    #[serde(default = "default_bell_enabled")]
    pub bell_enabled: bool,
}

impl Default for TerminalPreferences {
    fn default() -> Self {
        Self {
            shell: default_shell(),
            scrollback_lines: default_scrollback_lines(),
            bell_enabled: default_bell_enabled(),
        }
    }
}

fn default_shell() -> String {
    #[cfg(target_os = "windows")]
    { "powershell.exe".to_string() }
    #[cfg(target_os = "macos")]
    { "/bin/zsh".to_string() }
    #[cfg(target_os = "linux")]
    { "/bin/bash".to_string() }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    { "sh".to_string() }
}
fn default_scrollback_lines() -> u32 { 10000 }
fn default_bell_enabled() -> bool { false }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorPreferences {
    #[serde(default = "default_font_ligatures")]
    pub font_ligatures: bool,
    #[serde(default = "default_tab_size")]
    pub tab_size: u8,
    #[serde(default = "default_line_numbers")]
    pub line_numbers: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            font_ligatures: default_font_ligatures(),
            tab_size: default_tab_size(),
            line_numbers: default_line_numbers(),
        }
    }
}

fn default_font_ligatures() -> bool { false }
fn default_tab_size() -> u8 { 4 }
fn default_line_numbers() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeybindingPreferences {
    #[serde(default)]
    pub bindings: HashMap<String, String>, // Action -> Key combination
}

impl Default for KeybindingPreferences {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("copy".to_string(), "Cmd+C".to_string());
        bindings.insert("paste".to_string(), "Cmd+V".to_string());
        bindings.insert("new_tab".to_string(), "Cmd+T".to_string());
        Self { bindings }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentPermissionLevel {
    AgentDecides,
    Always,
    Never,
}

impl Default for AgentPermissionLevel {
    fn default() -> Self {
        AgentPermissionLevel::AgentDecides
    }
}

impl std::fmt::Display for AgentPermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            AgentPermissionLevel::AgentDecides => "Agent decides",
            AgentPermissionLevel::Always => "Always",
            AgentPermissionLevel::Never => "Never",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiPreferences {
    #[serde(default = "default_ai_provider_type")]
    pub ai_provider_type: String,
    #[serde(default = "default_ai_api_key")]
    pub ai_api_key: Option<String>,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default = "default_fallback_ai_provider_type")]
    pub fallback_ai_provider_type: Option<String>,
    #[serde(default = "default_fallback_ai_model")]
    pub fallback_ai_model: Option<String>,
    #[serde(default = "default_redact_sensitive_info")]
    pub redact_sensitive_info: bool,
    #[serde(default = "default_local_only_ai_mode")]
    pub local_only_ai_mode: bool,
    #[serde(default = "default_enable_graphql_api")]
    pub enable_graphql_api: bool,

    // New AI preferences from the image
    #[serde(default = "default_active_ai_next_command")]
    pub active_ai_next_command: bool,
    #[serde(default = "default_active_ai_prompt_suggestions")]
    pub active_ai_prompt_suggestions: bool,
    #[serde(default = "default_active_ai_shared_block_title_generation")]
    pub active_ai_shared_block_title_generation: bool,
    #[serde(default = "default_base_model")]
    pub base_model: String,
    #[serde(default = "default_show_model_picker_in_prompt")]
    pub show_model_picker_in_prompt: bool,
    #[serde(default = "default_planning_model")]
    pub planning_model: String,
    #[serde(default = "default_permission_apply_code_diffs")]
    pub permission_apply_code_diffs: AgentPermissionLevel,
    #[serde(default = "default_permission_read_files")]
    pub permission_read_files: AgentPermissionLevel,
    #[serde(default = "default_permission_create_plans")]
    pub permission_create_plans: AgentPermissionLevel,
    #[serde(default = "default_permission_execute_commands")]
    pub permission_execute_commands: AgentPermissionLevel,
}

impl Default for AiPreferences {
    fn default() -> Self {
        Self {
            ai_provider_type: default_ai_provider_type(),
            ai_api_key: default_ai_api_key(),
            ai_model: default_ai_model(),
            fallback_ai_provider_type: default_fallback_ai_provider_type(),
            fallback_ai_model: default_fallback_ai_model(),
            redact_sensitive_info: default_redact_sensitive_info(),
            local_only_ai_mode: default_local_only_ai_mode(),
            enable_graphql_api: default_enable_graphql_api(),
            active_ai_next_command: default_active_ai_next_command(),
            active_ai_prompt_suggestions: default_active_ai_prompt_suggestions(),
            active_ai_shared_block_title_generation: default_active_ai_shared_block_title_generation(),
            base_model: default_base_model(),
            show_model_picker_in_prompt: default_show_model_picker_in_prompt(),
            planning_model: default_planning_model(),
            permission_apply_code_diffs: default_permission_apply_code_diffs(),
            permission_read_files: default_permission_read_files(),
            permission_create_plans: default_permission_create_plans(),
            permission_execute_commands: default_permission_execute_commands(),
        }
    }
}

fn default_ai_provider_type() -> String { "openai".to_string() }
fn default_ai_api_key() -> Option<String> { None }
fn default_ai_model() -> String { "gpt-4o".to_string() }
fn default_fallback_ai_provider_type() -> Option<String> { None }
fn default_fallback_ai_model() -> Option<String> { None }
fn default_redact_sensitive_info() -> bool { true }
fn default_local_only_ai_mode() -> bool { false }
fn default_enable_graphql_api() -> bool { false }

fn default_active_ai_next_command() -> bool { true }
fn default_active_ai_prompt_suggestions() -> bool { true }
fn default_active_ai_shared_block_title_generation() -> bool { true }
fn default_base_model() -> String { "auto (claude 4 sonnet)".to_string() }
fn default_show_model_picker_in_prompt() -> bool { true }
fn default_planning_model() -> String { "o3".to_string() }
fn default_permission_apply_code_diffs() -> AgentPermissionLevel { AgentPermissionLevel::AgentDecides }
fn default_permission_read_files() -> AgentPermissionLevel { AgentPermissionLevel::AgentDecides }
fn default_permission_create_plans() -> AgentPermissionLevel { AgentPermissionLevel::Never }
fn default_permission_execute_commands() -> AgentPermissionLevel { AgentPermissionLevel::AgentDecides }


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivacyPreferences {
    #[serde(default = "default_command_history_retention_days")]
    pub command_history_retention_days: u16,
    #[serde(default = "default_ai_interaction_logging")]
    pub ai_interaction_logging: bool,
}

impl Default for PrivacyPreferences {
    fn default() -> Self {
        Self {
            command_history_retention_days: default_command_history_retention_days(),
            ai_interaction_logging: default_ai_interaction_logging(),
        }
    }
}

fn default_command_history_retention_days() -> u16 { 365 }
fn default_ai_interaction_logging() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformancePreferences {
    #[serde(default = "default_max_concurrent_commands")]
    pub max_concurrent_commands: u8,
    #[serde(default = "default_enable_gpu_acceleration")]
    pub enable_gpu_acceleration: bool,
}

impl Default for PerformancePreferences {
    fn default() -> Self {
        Self {
            max_concurrent_commands: default_max_concurrent_commands(),
            enable_gpu_acceleration: default_enable_gpu_acceleration(),
        }
    }
}

fn default_max_concurrent_commands() -> u8 { 10 }
fn default_enable_gpu_acceleration() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollaborationPreferences {
    #[serde(default = "default_enable_session_sharing")]
    pub enable_session_sharing: bool,
    #[serde(default = "default_default_share_mode")]
    pub default_share_mode: ShareMode,
}

impl Default for CollaborationPreferences {
    fn default() -> Self {
        Self {
            enable_session_sharing: default_enable_session_sharing(),
            default_share_mode: default_default_share_mode(),
        }
    }
}

fn default_enable_session_sharing() -> bool { false }
fn default_default_share_mode() -> ShareMode { ShareMode::ReadOnly }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShareMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudSyncPreferences {
    #[serde(default = "default_enable_cloud_sync")]
    pub enable_cloud_sync: bool,
    #[serde(default = "default_sync_interval_minutes")]
    pub sync_interval_minutes: u16,
    #[serde(default = "default_sync_on_startup")]
    pub sync_on_startup: bool,
}

impl Default for CloudSyncPreferences {
    fn default() -> Self {
        Self {
            enable_cloud_sync: default_enable_cloud_sync(),
            sync_interval_minutes: default_sync_interval_minutes(),
            sync_on_startup: default_sync_on_startup(),
        }
    }
}

fn default_enable_cloud_sync() -> bool { false }
fn default_sync_interval_minutes() -> u16 { 60 }
fn default_sync_on_startup() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentProfiles {
    #[serde(default = "default_active_profile")]
    pub active_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, EnvironmentProfile>,
}

impl Default for EnvironmentProfiles {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("default".to_string(), EnvironmentProfile::default());
        Self {
            active_profile: Some("default".to_string()),
            profiles,
        }
    }
}

fn default_active_profile() -> Option<String> { Some("default".to_string()) }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentProfile {
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

impl Default for EnvironmentProfile {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrivePreferences {
    #[serde(default = "default_enable_drive_integration")]
    pub enable_drive_integration: bool,
    #[serde(default = "default_default_drive_path")]
    pub default_drive_path: String,
}

impl Default for DrivePreferences {
    fn default() -> Self {
        Self {
            enable_drive_integration: default_enable_drive_integration(),
            default_drive_path: default_default_drive_path(),
        }
    }
}

fn default_enable_drive_integration() -> bool { false }
fn default_default_drive_path() -> String { "/mnt/neoterm_drive".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowPreferences {
    #[serde(default = "default_enable_workflow_suggestions")]
    pub enable_workflow_suggestions: bool,
    #[serde(default = "default_workflow_storage_path")]
    pub workflow_storage_path: String,
}

impl Default for WorkflowPreferences {
    fn default() -> Self {
        Self {
            enable_workflow_suggestions: default_enable_workflow_suggestions(),
            workflow_storage_path: default_workflow_storage_path(),
        }
    }
}

fn default_enable_workflow_suggestions() -> bool { true }
fn default_workflow_storage_path() -> String { "workflows".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexingPreferences {
    #[serde(default = "default_index_new_folders_by_default")]
    pub index_new_folders_by_default: bool,
}

impl Default for IndexingPreferences {
    fn default() -> Self {
        Self {
            index_new_folders_by_default: default_index_new_folders_by_default(),
        }
    }
}

fn default_index_new_folders_by_default() -> bool { true }


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    async fn setup_test_env() -> PathBuf {
        let test_dir = PathBuf::from("./test_config_prefs");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).await.unwrap();
        }
        fs::create_dir_all(&test_dir).await.unwrap();
        // Temporarily override CONFIG_DIR for tests
        unsafe {
            let config_dir_mut = &mut *(&*CONFIG_DIR as *const PathBuf as *mut PathBuf);
            *config_dir_mut = test_dir.clone();
        }
        test_dir
    }

    async fn cleanup_test_env(test_dir: PathBuf) {
        let _ = fs::remove_dir_all(&test_dir).await;
        // Restore original CONFIG_DIR if necessary (complex with Lazy, usually done via explicit paths)
    }

    #[tokio::test]
    async fn test_user_preferences_load_and_save() {
        let test_dir = setup_test_env().await;

        // Test loading default and saving
        let mut prefs = UserPreferences::load().await.unwrap();
        assert_eq!(prefs.general.font_size, 14);
        assert_eq!(prefs.ui.theme_name, "nord");
        assert_eq!(prefs.indexing.index_new_folders_by_default, true);
        assert_eq!(prefs.ai.active_ai_next_command, true); // Test new AI field

        // Modify and save
        prefs.general.font_size = 16;
        prefs.ui.theme_name = "dark_mode".to_string();
        prefs.indexing.index_new_folders_by_default = false;
        prefs.ai.active_ai_next_command = false; // Modify new AI field
        prefs.save().await.unwrap();

        // Load again and verify changes
        let loaded_prefs = UserPreferences::load().await.unwrap();
        assert_eq!(loaded_prefs.general.font_size, 16);
        assert_eq!(loaded_prefs.ui.theme_name, "dark_mode");
        assert_eq!(loaded_prefs.indexing.index_new_folders_by_default, false);
        assert_eq!(loaded_prefs.ai.active_ai_next_command, false); // Verify new AI field

        cleanup_test_env(test_dir).await;
    }

    #[tokio::test]
    async fn test_default_values() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.general.font_size, 14);
        assert_eq!(prefs.ui.theme_name, "nord");
        assert_eq!(prefs.terminal.scrollback_lines, 10000);
        assert_eq!(prefs.editor.tab_size, 4);
        assert_eq!(prefs.ai.ai_provider_type, "openai");
        assert_eq!(prefs.privacy.command_history_retention_days, 365);
        assert_eq!(prefs.performance.max_concurrent_commands, 10);
        assert_eq!(prefs.collaboration.enable_session_sharing, false);
        assert_eq!(prefs.cloud_sync.sync_interval_minutes, 60);
        assert_eq!(prefs.drive.enable_drive_integration, false);
        assert_eq!(prefs.workflows.enable_workflow_suggestions, true);
        assert_eq!(prefs.indexing.index_new_folders_by_default, true);
        // New AI fields defaults
        assert_eq!(prefs.ai.active_ai_next_command, true);
        assert_eq!(prefs.ai.active_ai_prompt_suggestions, true);
        assert_eq!(prefs.ai.active_ai_shared_block_title_generation, true);
        assert_eq!(prefs.ai.base_model, "auto (claude 4 sonnet)".to_string());
        assert_eq!(prefs.ai.show_model_picker_in_prompt, true);
        assert_eq!(prefs.ai.planning_model, "o3".to_string());
        assert_eq!(prefs.ai.permission_apply_code_diffs, AgentPermissionLevel::AgentDecides);
        assert_eq!(prefs.ai.permission_read_files, AgentPermissionLevel::AgentDecides);
        assert_eq!(prefs.ai.permission_create_plans, AgentPermissionLevel::Never);
        assert_eq!(prefs.ai.permission_execute_commands, AgentPermissionLevel::AgentDecides);
    }

    #[tokio::test]
    async fn test_environment_profiles_default() {
        let env_profiles = EnvironmentProfiles::default();
        assert_eq!(env_profiles.active_profile, Some("default".to_string()));
        assert!(env_profiles.profiles.contains_key("default"));
        assert!(env_profiles.profiles.get("default").unwrap().variables.is_empty());
    }
}
