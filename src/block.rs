use iced::{
    widget::{column, container, row, text, button, scrollable, text_input},
    Element, Length, Color, alignment,
};
use uuid::Uuid;
use chrono::{DateTime, Local};
use crate::workflows::Workflow;
use log::info;

/// Represents the content type of a UI block in the Iced GUI.
#[derive(Debug, Clone)]
pub enum BlockContent {
    /// Represents a command execution block with input, output, status, and error state.
    Command {
        input: String,
        output: Vec<(String, bool)>, // (content, is_stdout)
        status: String,
        error: bool,
        start_time: DateTime<Local>,
        end_time: Option<DateTime<Local>>,
    },
    /// Represents a message from the AI agent or the user.
    AgentMessage {
        content: String,
        is_user: bool,
        timestamp: DateTime<Local>,
    },
    /// Represents an informational message.
    Info {
        title: String,
        message: String,
        timestamp: DateTime<Local>,
    },
    /// Represents an error message.
    Error {
        message: String,
        timestamp: DateTime<Local>,
    },
    /// Represents an AI-suggested workflow.
    WorkflowSuggestion {
        workflow: Workflow,
    },
    /// Represents an interactive prompt from the AI agent requiring user input.
    AgentPrompt {
        prompt_id: String,
        message: String,
        input_value: String, // Current value in the input field for this prompt
    },
    // Add other block types as needed (e.g., Code, Image, Workflow)
}

/// Represents a UI block in the Iced GUI.
#[derive(Debug, Clone)]
pub struct Block {
    pub id: String,
    pub content: BlockContent,
    pub collapsed: bool,
    pub status: Option<String>, // For streaming updates
}

impl Block {
    /// Creates a new command block.
    pub fn new_command(input: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::Command {
                input,
                output: Vec::new(),
                status: "Running...".to_string(),
                error: false,
                start_time: Local::now(),
                end_time: None,
            },
            collapsed: false,
            status: Some("Running...".to_string()),
        }
    }

    /// Creates a new agent message block (from the AI).
    pub fn new_agent_message(content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::AgentMessage {
                content,
                is_user: false,
                timestamp: Local::now(),
            },
            collapsed: false,
            status: None, // Status will be set during streaming
        }
    }

    /// Creates a new user message block (sent by the user to the AI).
    pub fn new_user_message(content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::AgentMessage {
                content,
                is_user: true,
                timestamp: Local::now(),
            },
            collapsed: false,
            status: None,
        }
    }

    /// Creates a new informational block.
    pub fn new_info(title: String, message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::Info {
                title,
                message,
                timestamp: Local::now(),
            },
            collapsed: false,
            status: None,
        }
    }

    /// Creates a new error block.
    pub fn new_error(message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::Error {
                message,
                timestamp: Local::now(),
            },
            collapsed: false,
            status: Some("Error".to_string()),
        }
    }

    /// Creates a new output block (typically for initial output display).
    pub fn new_output(initial_output: String) -> Self {
        let mut block = Self::new_command("".to_string()); // Use command block for output
        if let BlockContent::Command { output, .. } = &mut block.content {
            output.push((initial_output, true));
        }
        block
    }

    /// Creates a new workflow suggestion block.
    pub fn new_workflow_suggestion(workflow: Workflow) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::WorkflowSuggestion { workflow },
            collapsed: false,
            status: Some("Suggested Workflow".to_string()),
        }
    }

    /// Creates a new agent prompt block, requiring user input.
    pub fn new_agent_prompt(prompt_id: String, message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::AgentPrompt {
                prompt_id,
                message,
                input_value: String::new(),
            },
            collapsed: false,
            status: Some("Agent Input Required".to_string()),
        }
    }

    /// Adds a line of output to a command block.
    pub fn add_output_line(&mut self, line: String, is_stdout: bool) {
        if let BlockContent::Command { output, .. } = &mut self.content {
            output.push((line, is_stdout));
        }
    }

    /// Sets the status of the block.
    pub fn set_status(&mut self, status: String) {
        match &mut self.content {
            BlockContent::Command { status: s, end_time, .. } => {
                *s = status.clone();
                *end_time = Some(Local::now());
            },
            BlockContent::AgentMessage { .. } | BlockContent::Info { .. } | BlockContent::Error { .. } |
            BlockContent::WorkflowSuggestion { .. } | BlockContent::AgentPrompt { .. } => {
                // For other block types, update the general status field
            }
        }
        self.status = Some(status);
    }

    /// Sets the error state of a command block.
    pub fn set_error(&mut self, error: bool) {
        if let BlockContent::Command { error: e, .. } = &mut self.content {
            *e = error;
        }
    }

    /// Toggles the collapsed state of the block.
    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Renders the UI block as an Iced Element.
    /// This function dynamically renders the block based on its `BlockContent` type
    /// and its `collapsed` state.
    pub fn view(&self) -> Element<crate::Message> {
        // Display a truncated ID for identification
        let id_text = text(format!("#{}", &self.id[0..8])).size(12).color(Color::from_rgb(0.5, 0.5, 0.5));
        
        // Button to toggle collapse state
        let toggle_button = button(text(if self.collapsed { "▶" } else { "▼" }))
            .on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::ToggleCollapse))
            .style(iced::widget::button::text::Style::Text);

        // Row for common block actions
        let mut actions_row = row![
            toggle_button,
            id_text,
            button(text("📋")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::Copy)).style(iced::widget::button::text::Style::Text),
            button(text("🔄")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::Rerun)).style(iced::widget::button::text::Style::Text),
            button(text("🗑️")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::Delete)).style(iced::widget::button::text::Style::Text),
            button(text("📤")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::Export)).style(iced::widget::button::text::Style::Text),
            button(text("🤖")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::SendToAI)).style(iced::widget::button::text::Style::Text),
        ];

        // Conditionally show "Fix" button for failed command blocks
        if let BlockContent::Command { error: true, .. } = self.content {
            actions_row = actions_row.push(
                button(text("🛠️ Fix")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::SuggestFix)).style(iced::widget::button::text::Style::Text)
            );
        }

        // Conditionally show "Explain Output" button for command and error blocks
        match self.content {
            BlockContent::Command { .. } | BlockContent::Error { .. } => {
                actions_row = actions_row.push(
                    button(text("❓ Explain")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::ExplainOutput)).style(iced::widget::button::text::Style::Text)
                );
            }
            _ => {}
        }

        // Conditionally show "Accept/Reject" for WorkflowSuggestion blocks
        if let BlockContent::WorkflowSuggestion { .. } = self.content {
            actions_row = actions_row.push(
                button(text("✅ Accept")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::AcceptWorkflow)).style(iced::widget::button::text::Style::Text)
            );
            actions_row = actions_row.push(
                button(text("❌ Reject")).on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::RejectWorkflow)).style(iced::widget::button::text::Style::Text)
            );
        }

        let header = actions_row.spacing(5).align_items(alignment::Horizontal::Center);

        // Render content based on collapsed state and block type
        let content_view: Element<crate::Message> = if self.collapsed {
            // Collapsed view: show a summary
            match &self.content {
                BlockContent::Command { input, status, error, .. } => {
                    row![
                        text(input).size(16).color(Color::BLACK),
                        text(format!("Status: {}", status)).size(14).color(if *error { Color::from_rgb(1.0, 0.0, 0.0) } else { Color::from_rgb(0.0, 0.5, 0.0) }),
                    ].spacing(10).into()
                }
                BlockContent::AgentMessage { content, is_user, .. } => {
                    row![
                        text(if *is_user { "You:" } else { "Agent:" }).size(14).color(Color::from_rgb(0.2, 0.2, 0.8)),
                        text(content.lines().next().unwrap_or("...")).size(16), // Show only first line
                    ].spacing(10).into()
                }
                BlockContent::Info { title, .. } => {
                    row![
                        text(format!("Info: {}", title)).size(16).color(Color::from_rgb(0.0, 0.5, 0.8)),
                    ].spacing(10).into()
                }
                BlockContent::Error { message, .. } => {
                    row![
                        text(format!("Error: {}", message.lines().next().unwrap_or("..."))).size(16).color(Color::from_rgb(1.0, 0.0, 0.0)),
                    ].spacing(10).into()
                }
                BlockContent::WorkflowSuggestion { workflow } => {
                    row![
                        text(format!("Suggested Workflow: {}", workflow.name)).size(16).color(Color::from_rgb(0.0, 0.7, 0.0)),
                        text(workflow.description.as_deref().unwrap_or("No description")).size(14),
                    ].spacing(10).into()
                }
                BlockContent::AgentPrompt { message, .. } => {
                    row![
                        text("Agent Prompt:").size(14).color(Color::from_rgb(0.8, 0.5, 0.0)),
                        text(message.lines().next().unwrap_or("...")).size(16), // Show only first line
                    ].spacing(10).into()
                }
            }
        } else {
            // Expanded view: show full content
            match &self.content {
                BlockContent::Command { input, output, status, error, start_time, end_time } => {
                    // Render command input
                    let input_view = text(input).size(16).color(Color::from_rgb(0.2, 0.2, 0.8));
                    
                    // Render command output, distinguishing stdout/stderr
                    let output_text = output.iter().map(|(line, is_stdout)| {
                        text(line).size(14).color(if *is_stdout { Color::BLACK } else { Color::from_rgb(0.8, 0.0, 0.0) })
                    }).fold(column![], |col, txt| col.push(txt));

                    // Calculate and display command duration
                    let duration = end_time.map(|e| e - *start_time).map(|d| format!("Duration: {}ms", d.num_milliseconds())).unwrap_or_default();

                    column![
                        input_view,
                        scrollable(output_text).height(Length::Shrink).width(Length::Fill),
                        row![
                            text(format!("Status: {}", status)).size(14).color(if *error { Color::from_rgb(1.0, 0.0, 0.0) } else { Color::from_rgb(0.0, 0.5, 0.0) }),
                            text(duration).size(14).color(Color::from_rgb(0.5, 0.5, 0.5)),
                        ].spacing(10)
                    ].spacing(5).into()
                }
                BlockContent::AgentMessage { content, is_user, timestamp } => {
                    column![
                        text(if *is_user { "You:" } else { "Agent:" }).size(14).color(Color::from_rgb(0.2, 0.2, 0.8)),
                        text(content).size(16),
                        text(timestamp.format("%H:%M:%S").to_string()).size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ].spacing(5).into()
                }
                BlockContent::Info { title, message, timestamp } => {
                    column![
                        text(title).size(18).color(Color::from_rgb(0.0, 0.5, 0.8)),
                        text(message).size(16),
                        text(timestamp.format("%H:%M:%S").to_string()).size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ].spacing(5).into()
                }
                BlockContent::Error { message, timestamp } => {
                    column![
                        text("Error!").size(18).color(Color::from_rgb(1.0, 0.0, 0.0)),
                        text(message).size(16),
                        text(timestamp.format("%H:%M:%S").to_string()).size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ].spacing(5).into()
                }
                BlockContent::WorkflowSuggestion { workflow } => {
                    // Display workflow steps
                    let steps_view = workflow.steps.iter().enumerate().map(|(i, step)| {
                        row![
                            text(format!("{}. {}", i + 1, step.name)).size(14).color(Color::from_rgb(0.2, 0.2, 0.2)),
                            text(format!("Type: {:?}", step.step_type)).size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                        ].spacing(5).into()
                    }).fold(column![], |col, elem| col.push(elem));

                    column![
                        text(format!("Suggested Workflow: {}", workflow.name)).size(18).color(Color::from_rgb(0.0, 0.7, 0.0)),
                        text(workflow.description.as_deref().unwrap_or("No description provided.")).size(14),
                        text("Steps:").size(16).color(Color::from_rgb(0.3, 0.3, 0.3)),
                        steps_view,
                    ].spacing(5).into()
                }
                BlockContent::AgentPrompt { prompt_id: _, message, input_value } => {
                    column![
                        text("Agent Prompt:").size(16).color(Color::from_rgb(0.8, 0.5, 0.0)),
                        text(message).size(16),
                        // Text input field for user response
                        text_input("Enter your response...", input_value)
                            .on_input(move |s| crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::AgentPromptInputChanged(s)))
                            .on_submit(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::SubmitAgentPrompt)),
                        // Submit button for the prompt
                        button(text("Submit"))
                            .on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::SubmitAgentPrompt)),
                    ].spacing(5).into()
                }
            }
        };

        // Main container for the block, with styling
        container(
            column![
                header,
                content_view,
            ]
            .spacing(5)
        )
        .padding(10)
        .style(iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::WHITE)),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.8, 0.8, 0.8),
            ..Default::default()
        })
        .into()
    }
}

/// Initializes the block module.
pub fn init() {
    info!("Block module initialized.");
}
