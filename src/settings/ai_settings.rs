use iced::{
    widget::{column, row, text, checkbox, pick_list, horizontal_rule},
    Element, Length, Color, alignment,
};
use crate::config::preferences::{AiPreferences, AgentPermissionLevel};
use log::info;

#[derive(Debug, Clone)]
pub enum AiSettingsMessage {
    ToggleNextCommand(bool),
    TogglePromptSuggestions(bool),
    ToggleSharedBlockTitleGeneration(bool),
    BaseModelChanged(String),
    ToggleShowModelPickerInPrompt(bool),
    PlanningModelChanged(String),
    PermissionApplyCodeDiffsChanged(AgentPermissionLevel),
    PermissionReadFilesChanged(AgentPermissionLevel),
    PermissionCreatePlansChanged(AgentPermissionLevel),
    PermissionExecuteCommandsChanged(AgentPermissionLevel),
}

#[derive(Debug, Clone)]
pub struct AiSettings {
    pub preferences: AiPreferences,
    // Available models for pick_list
    pub available_base_models: Vec<String>,
    pub available_planning_models: Vec<String>,
    pub available_permission_levels: Vec<AgentPermissionLevel>,
}

impl AiSettings {
    pub fn new(preferences: AiPreferences) -> Self {
        Self {
            preferences,
            available_base_models: vec![
                "auto (claude 4 sonnet)".to_string(),
                "gpt-4o".to_string(),
                "grok-3".to_string(),
                "llama3".to_string(),
            ],
            available_planning_models: vec![
                "o3".to_string(),
                "o4".to_string(),
            ],
            available_permission_levels: vec![
                AgentPermissionLevel::AgentDecides,
                AgentPermissionLevel::Always,
                AgentPermissionLevel::Never,
            ],
        }
    }

    pub fn update(&mut self, message: AiSettingsMessage) {
        match message {
            AiSettingsMessage::ToggleNextCommand(value) => {
                self.preferences.active_ai_next_command = value;
                info!("Active AI - Next Command: {}", value);
            }
            AiSettingsMessage::TogglePromptSuggestions(value) => {
                self.preferences.active_ai_prompt_suggestions = value;
                info!("Active AI - Prompt Suggestions: {}", value);
            }
            AiSettingsMessage::ToggleSharedBlockTitleGeneration(value) => {
                self.preferences.active_ai_shared_block_title_generation = value;
                info!("Active AI - Shared Block Title Generation: {}", value);
            }
            AiSettingsMessage::BaseModelChanged(value) => {
                self.preferences.base_model = value;
                info!("Base Model changed to: {}", self.preferences.base_model);
            }
            AiSettingsMessage::ToggleShowModelPickerInPrompt(value) => {
                self.preferences.show_model_picker_in_prompt = value;
                info!("Show model picker in prompt: {}", value);
            }
            AiSettingsMessage::PlanningModelChanged(value) => {
                self.preferences.planning_model = value;
                info!("Planning Model changed to: {}", self.preferences.planning_model);
            }
            AiSettingsMessage::PermissionApplyCodeDiffsChanged(value) => {
                self.preferences.permission_apply_code_diffs = value;
                info!("Permission (Apply code diffs) changed to: {:?}", value);
            }
            AiSettingsMessage::PermissionReadFilesChanged(value) => {
                self.preferences.permission_read_files = value;
                info!("Permission (Read files) changed to: {:?}", value);
            }
            AiSettingsMessage::PermissionCreatePlansChanged(value) => {
                self.preferences.permission_create_plans = value;
                info!("Permission (Create plans) changed to: {:?}", value);
            }
            AiSettingsMessage::PermissionExecuteCommandsChanged(value) => {
                self.preferences.permission_execute_commands = value;
                info!("Permission (Execute commands) changed to: {:?}", value);
            }
        }
    }

    pub fn view(&self) -> Element<AiSettingsMessage> {
        let header = text("AI").size(24).color(Color::WHITE);

        let active_ai_section = column![
            text("Active AI").size(18).color(Color::WHITE),
            self.toggle_row(
                "Next Command",
                "Let AI suggest the next command to run based on your command history, outputs, and common workflows.",
                self.preferences.active_ai_next_command,
                AiSettingsMessage::ToggleNextCommand,
            ),
            self.toggle_row(
                "Prompt Suggestions",
                "Let AI suggest natural language prompts, as inline banners, based on recent commands and their outputs.",
                self.preferences.active_ai_prompt_suggestions,
                AiSettingsMessage::TogglePromptSuggestions,
            ),
            self.toggle_row(
                "Shared Block Title Generation",
                "Let AI generate a title for your shared block based on the command and output.",
                self.preferences.active_ai_shared_block_title_generation,
                AiSettingsMessage::ToggleSharedBlockTitleGeneration,
            ),
        ]
        .spacing(15);

        let agents_section = column![
            text("Agents").size(18).color(Color::WHITE),
            text("Set the boundaries for how your Agent operates. Choose what it can access, how much autonomy it has, and when it must ask for your approval. You can also fine-tune behavior around natural language input, codebase awareness, and more.")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
        ]
        .spacing(10);

        let models_section = column![
            text("Models").size(18).color(Color::WHITE),
            self.model_picker_row(
                "Base model",
                "This model serves as the core engine for your terminal. It drives most interactions and invokes other models as necessary.",
                &self.available_base_models,
                &self.preferences.base_model,
                AiSettingsMessage::BaseModelChanged,
                Some((self.preferences.show_model_picker_in_prompt, AiSettingsMessage::ToggleShowModelPickerInPrompt, "Show model picker in prompt")),
            ),
            self.model_picker_row(
                "Planning model",
                "The planning model is responsible for breaking down complex tasks into actionable steps. It creates structured execution plans and decides how to route work between models.",
                &self.available_planning_models,
                &self.preferences.planning_model,
                AiSettingsMessage::PlanningModelChanged,
                None,
            ),
        ]
        .spacing(15);

        let permissions_section = column![
            text("Permissions").size(18).color(Color::WHITE),
            self.permission_row(
                "</> Apply code diffs",
                "The Agent chooses the safest path: acting on its own when confident, and asking for approval when uncertain.",
                &self.available_permission_levels,
                self.preferences.permission_apply_code_diffs.clone(),
                AiSettingsMessage::PermissionApplyCodeDiffsChanged,
            ),
            self.permission_row(
                "📄 Read files",
                "The Agent chooses the safest path: acting on its own when confident, and asking for approval when uncertain.",
                &self.available_permission_levels,
                self.preferences.permission_read_files.clone(),
                AiSettingsMessage::PermissionReadFilesChanged,
            ),
            self.permission_row(
                "📋 Create plans",
                "The Agent will never do this.",
                &self.available_permission_levels,
                self.preferences.permission_create_plans.clone(),
                AiSettingsMessage::PermissionCreatePlansChanged,
            ),
            self.permission_row(
                ">_ Execute commands",
                "The Agent chooses the safest path: acting on its own when confident, and asking for approval when uncertain.",
                &self.available_permission_levels,
                self.preferences.permission_execute_commands.clone(),
                AiSettingsMessage::PermissionExecuteCommandsChanged,
            ),
        ]
        .spacing(15);


        column![
            header,
            active_ai_section,
            horizontal_rule(1), // Separator
            agents_section,
            horizontal_rule(1), // Separator
            models_section,
            horizontal_rule(1), // Separator
            permissions_section,
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn toggle_row<Message>(
        &self,
        title: &str,
        description: &str,
        is_checked: bool,
        on_toggle: fn(bool) -> Message,
    ) -> Element<Message>
    where
        Message: 'static + Clone,
    {
        row![
            column![
                text(title).size(16).color(Color::WHITE),
                text(description).size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .width(Length::Fill)
            .spacing(2),
            checkbox("", is_checked, on_toggle)
                .text_color(Color::WHITE)
                .size(20)
        ]
        .align_items(alignment::Vertical::Center)
        .spacing(10)
        .into()
    }

    fn model_picker_row<Message>(
        &self,
        title: &str,
        description: &str,
        options: &[String],
        selected: &str,
        on_change: fn(String) -> Message,
        checkbox_info: Option<(bool, fn(bool) -> Message, &str)>,
    ) -> Element<Message>
    where
        Message: 'static + Clone,
    {
        let mut content = column![
            text(title).size(16).color(Color::WHITE),
            text(description).size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
        ]
        .spacing(2);

        let mut picker_row = row![
            pick_list(options, Some(selected.to_string()), on_change)
                .width(Length::FillPortion(0.7)),
        ]
        .spacing(10)
        .align_items(alignment::Vertical::Center);

        if let Some((is_checked, on_toggle, checkbox_label)) = checkbox_info {
            picker_row = picker_row.push(
                checkbox(checkbox_label, is_checked, on_toggle)
                    .text_color(Color::WHITE)
                    .size(16)
            );
        }

        content = content.push(picker_row);
        content.into()
    }

    fn permission_row<Message>(
        &self,
        title: &str,
        description: &str,
        options: &[AgentPermissionLevel],
        selected: AgentPermissionLevel,
        on_change: fn(AgentPermissionLevel) -> Message,
    ) -> Element<Message>
    where
        Message: 'static + Clone,
    {
        column![
            text(title).size(16).color(Color::WHITE),
            pick_list(options, Some(selected), on_change)
                .width(Length::Fill),
            row![
                text("ⓘ").size(14).color(Color::from_rgb(0.8, 0.8, 0.2)), // Yellow info icon
                text(description).size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(5)
            .align_items(alignment::Vertical::Center),
        ]
        .spacing(5)
        .into()
    }
}

pub fn init() {
    info!("AI settings module loaded");
}
