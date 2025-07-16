use iced::{
    widget::{column, container, row, text, button, scrollable, text_input},
    Element, Length, Color, alignment,
};
use uuid::Uuid;
use chrono::{DateTime, Local, Duration};
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
        working_directory: Option<String>, // New field for working directory
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
    /// Represents a tool call whose arguments are being streamed.
    StreamingToolCall {
        id: String, // The tool_call_id from the AI
        name: String,
        arguments: String, // Accumulate arguments as a string
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
    pub background_color: Option<Color>, // New field for custom background color
}

impl Block {
    /// Creates a new command block.
    pub fn new_command(input: String, working_directory: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: BlockContent::Command {
                input,
                output: Vec::new(),
                status: "Running...".to_string(),
                error: false,
                start_time: Local::now(),
                end_time: None,
                working_directory,
            },
            collapsed: false,
            status: Some("Running...".to_string()),
            background_color: None, // Default to no custom background
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
            background_color: None,
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
            background_color: None,
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
            background_color: None,
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
            background_color: None,
        }
    }

    /// Creates a new output block (typically for initial output display).
    /// This is now deprecated in favor of `new_command` with output added later.
    pub fn new_output(initial_output: String) -> Self {
        let mut block = Self::new_command("".to_string(), None); // Use command block for output
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
            background_color: None,
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
            background_color: None,
        }
    }

    /// Creates a new streaming tool call block.
    /// The `block_id` is a new UUID for the UI element, while `tool_call_id` is the AI's ID for the tool call.
    pub fn new_streaming_tool_call(tool_call_id: String, name: String, initial_arguments: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(), // Use a new block ID for the UI block
            content: BlockContent::StreamingToolCall {
                id: tool_call_id, // This is the tool_call_id from the AI
                name,
                arguments: initial_arguments,
            },
            collapsed: false,
            status: Some("Streaming Tool Call...".to_string()),
            background_color: None,
        }
    }

    /// Creates a new block with a specified background color.
    pub fn new_with_background(content: BlockContent, background_color: Color) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            collapsed: false,
            status: None, // Status will be set by content type or later
            background_color: Some(background_color),
        }
    }

    /// Adds a line of output to a command block.
    pub fn add_output_line(&mut self, line: String, is_stdout: bool) {
        if let BlockContent::Command { output, .. } = &mut self.content {
            output.push((line, is_stdout));
        }
    }

    /// Updates the arguments of a streaming tool call block.
    pub fn update_streaming_tool_call_arguments(&mut self, new_arguments: String) {
        if let BlockContent::StreamingToolCall { arguments, .. } = &mut self.content {
            *arguments = new_arguments;
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
            BlockContent::WorkflowSuggestion { .. } | BlockContent::AgentPrompt { .. } |
            BlockContent::StreamingToolCall { .. } => {
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
        // Determine background color
        let background_color = self.background_color.unwrap_or(Color::from_rgb(0.1, 0.1, 0.1)); // Dark background by default

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
                        text(input).size(16).color(Color::WHITE),
                        text(format!("Status: {}", status)).size(14).color(if *error { Color::from_rgb(1.0, 0.0, 0.0) } else { Color::from_rgb(0.0, 0.8, 0.0) }),
                    ].spacing(10).into()
                }
                BlockContent::AgentMessage { content, is_user, .. } => {
                    row![
                        text(if *is_user { "You:" } else { "Agent:" }).size(14).color(Color::from_rgb(0.5, 0.5, 1.0)),
                        text(content.lines().next().unwrap_or("...")).size(16).color(Color::WHITE), // Show only first line
                    ].spacing(10).into()
                }
                BlockContent::Info { title, .. } => {
                    row![
                        text(format!("Info: {}", title)).size(16).color(Color::from_rgb(0.0, 0.7, 1.0)),
                    ].spacing(10).into()
                }
                BlockContent::Error { message, .. } => {
                    row![
                        text(format!("Error: {}", message.lines().next().unwrap_or("..."))).size(16).color(Color::from_rgb(1.0, 0.0, 0.0)),
                    ].spacing(10).into()
                }
                BlockContent::WorkflowSuggestion { workflow } => {
                    row![
                        text(format!("Suggested Workflow: {}", workflow.name)).size(16).color(Color::from_rgb(0.0, 0.9, 0.0)),
                        text(workflow.description.as_deref().unwrap_or("No description")).size(14).color(Color::WHITE),
                    ].spacing(10).into()
                }
                BlockContent::AgentPrompt { message, .. } => {
                    row![
                        text("Agent Prompt:").size(14).color(Color::from_rgb(1.0, 0.7, 0.0)),
                        text(message.lines().next().unwrap_or("...")).size(16).color(Color::WHITE), // Show only first line
                    ].spacing(10).into()
                }
                BlockContent::StreamingToolCall { name, arguments, .. } => {
                    row![
                        text(format!("Tool Call: {}", name)).size(16).color(Color::from_rgb(1.0, 0.7, 0.0)),
                        text(arguments.lines().next().unwrap_or("...")).size(14).color(Color::WHITE),
                    ].spacing(10).into()
                }
            }
        } else {
            // Expanded view: show full content
            match &self.content {
                BlockContent::Command { input, output, status, error, start_time, end_time, working_directory } => {
                    // Header for command block: path and duration
                    let duration_text = if let (Some(start), Some(end)) = (start_time.checked_add_signed(Duration::zero()), end_time) {
                        let duration = end.signed_duration_since(*start);
                        format!(" ({:.3}s)", duration.num_milliseconds() as f64 / 1000.0)
                    } else {
                        "".to_string()
                    };
                    let path_text = working_directory.as_deref().unwrap_or("~");
                    let command_header = text(format!("{} {}", path_text, duration_text))
                        .size(14)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)); // Light gray for path/duration

                    // Render command input
                    let input_view = text(input).size(16).color(Color::WHITE);
                    
                    // Render command output, distinguishing stdout/stderr
                    let output_text = output.iter().map(|(line, is_stdout)| {
                        text(line).size(14).color(if *is_stdout { Color::WHITE } else { Color::from_rgb(1.0, 0.5, 0.5) }) // Red for stderr
                    }).fold(column![], |col, txt| col.push(txt));

                    column![
                        command_header,
                        input_view,
                        scrollable(output_text).height(Length::Shrink).width(Length::Fill),
                        row![
                            text(format!("Status: {}", status)).size(14).color(if *error { Color::from_rgb(1.0, 0.0, 0.0) } else { Color::from_rgb(0.0, 0.8, 0.0) }),
                        ].spacing(10)
                    ].spacing(5).into()
                }
                BlockContent::AgentMessage { content, is_user, timestamp } => {
                    column![
                        text(if *is_user { "You:" } else { "Agent:" }).size(14).color(Color::from_rgb(0.5, 0.5, 1.0)),
                        text(content).size(16).color(Color::WHITE),
                        text(timestamp.format("%H:%M:%S").to_string()).size(12).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ].spacing(5).into()
                }
                BlockContent::Info { title, message, timestamp } => {
                    column![
                        text(title).size(18).color(Color::from_rgb(0.0, 0.7, 1.0)),
                        text(message).size(16).color(Color::WHITE),
                        text(timestamp.format("%H:%M:%S").to_string()).size(12).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ].spacing(5).into()
                }
                BlockContent::Error { message, timestamp } => {
                    column![
                        text("Error!").size(18).color(Color::from_rgb(1.0, 0.0, 0.0)),
                        text(message).size(16).color(Color::WHITE),
                        text(timestamp.format("%H:%M:%S").to_string()).size(12).color(Color::from_rgb(0.7, 0.7, 0.7)),
                    ].spacing(5).into()
                }
                BlockContent::WorkflowSuggestion { workflow } => {
                    // Display workflow steps
                    let steps_view = workflow.steps.iter().enumerate().map(|(i, step)| {
                        row![
                            text(format!("{}. {}", i + 1, step.name)).size(14).color(Color::WHITE),
                            text(format!("Type: {:?}", step.step_type)).size(12).color(Color::from_rgb(0.7, 0.7, 0.7)),
                        ].spacing(5).into()
                    }).fold(column![], |col, elem| col.push(elem));

                    column![
                        text(format!("Suggested Workflow: {}", workflow.name)).size(18).color(Color::from_rgb(0.0, 0.9, 0.0)),
                        text(workflow.description.as_deref().unwrap_or("No description provided.")).size(14).color(Color::WHITE),
                        text("Steps:").size(16).color(Color::from_rgb(0.5, 0.5, 0.5)),
                        steps_view,
                    ].spacing(5).into()
                }
                BlockContent::AgentPrompt { prompt_id: _, message, input_value } => {
                    column![
                        text("Agent Prompt:").size(16).color(Color::from_rgb(1.0, 0.7, 0.0)),
                        text(message).size(16).color(Color::WHITE),
                        // Text input field for user response
                        text_input("Enter your response...", input_value)
                            .on_input(move |s| crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::AgentPromptInputChanged(s)))
                            .on_submit(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::SubmitAgentPrompt))
                            .style(iced::widget::text_input::Appearance {
                                background: iced::Background::Color(Color::from_rgb(0.2, 0.2, 0.2)),
                                border_radius: 3.0,
                                border_width: 1.0,
                                border_color: Color::from_rgb(0.3, 0.3, 0.3),
                                text_color: Color::WHITE,
                                ..Default::default()
                            }),
                        // Submit button for the prompt
                        button(text("Submit").color(Color::WHITE))
                            .on_press(crate::Message::BlockAction(self.id.clone(), crate::main::BlockMessage::SubmitAgentPrompt))
                            .style(iced::widget::button::text::Style::Text), // Use default button style for now
                    ].spacing(5).into()
                }
                BlockContent::StreamingToolCall { id, name, arguments } => {
                    column![
                        text(format!("Streaming Tool Call (ID: {})", id)).size(18).color(Color::from_rgb(1.0, 0.7, 0.0)),
                        text(format!("Function: {}", name)).size(16).color(Color::WHITE),
                        text("Arguments:").size(14).color(Color::from_rgb(0.7, 0.7, 0.7)),
                        scrollable(text(arguments.clone()).size(14).color(Color::WHITE)).height(Length::Shrink).width(Length::Fill),
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
            background: Some(iced::Background::Color(background_color)),
            border_radius: 5.0,
            border_width: 0.0, // No border for these blocks
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
