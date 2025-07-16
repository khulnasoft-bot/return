//! NeoTerm: A next-generation terminal with AI assistance.
//!
//! This is the main entry point for the NeoTerm application,
//! handling the core application state, UI rendering, and
//! interactions with various backend managers (AI, commands, workflows, etc.).

use iced::{executor, Application, Command, Element, Settings, Theme};
use iced::widget::{column, container, scrollable, text_input, button, row, text};
use iced::keyboard::{self, KeyCode, Modifiers};
use std::path::PathBuf;
use tokio::sync::{mpsc, RwLock, Mutex};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use futures_util::StreamExt;
use chrono::{DateTime, Local};
use clap::Parser;
use anyhow::Result;
use log::{error, info};
use itertools::Itertools; // For .join() on iterators
use serde_json::Value; // For parsing tool call arguments

// Import modules
mod ai;
mod agent_mode_eval;
mod api;
mod asset_macro;
mod block;
mod cli;
mod cloud;
mod collaboration;
mod command;
mod config;
mod drive;
mod fuzzy_match;
mod graphql;
mod input;
mod integration;
mod languages;
mod lpc;
mod main_loop;
mod markdown_parser;
mod mcq;
mod natural_language_detection;
mod performance;
mod plugins;
mod renderer;
mod serve_wasm;
mod settings;
mod shell;
mod string_offset;
mod sum_tree;
mod syntax_tree;
mod ui;
mod virtual_fs;
mod watcher;
mod websocket;
mod workflows;

// Use statements for key components
use ai::assistant::Assistant;
use ai::context::AIContext; // Import AIContext
use agent_mode_eval::{AgentConfig, AgentMessage, AgentMode};
use cli::{Cli, CliCommand};
use command::CommandManager;
use config::ConfigManager;
use plugins::plugin_manager::PluginManager;
use virtual_fs::VirtualFileSystem;
use watcher::Watcher;
use workflows::manager::WorkflowManager;

use block::{Block, BlockContent};
use shell::ShellManager;
use input::{EnhancedTextInput, Message as InputMessage, HistoryDirection, Direction};
use config::{AppConfig, preferences::UserPreferences};
use crate::{
    ui::command_palette::{CommandPalette, CommandAction},
    ui::ai_sidebar::AISidebar,
    command::pty::{PtyManager, CommandStatus},
    workflows::debugger::WorkflowDebugger,
    plugins::plugin_manager::PluginManager as IcedPluginManager,
    collaboration::session_sharing::SessionSharingManager as IcedSessionSharingManager,
    cloud::sync_manager::{CloudSyncManager as SyncManager, SyncEvent, SyncConfig},
    performance::benchmarks::{PerformanceBenchmarks, BenchmarkSuite, BenchmarkResult},
    cli::{Commands, ConfigCommands, AiCommands, PluginCommands, WorkflowCommands},
};
use command::{CommandEvent};
use drive::{DriveManager, DriveConfig, DriveEvent};
use fuzzy_match::FuzzyMatchManager;
use graphql::build_schema;
use languages::LanguageManager;
use lpc::LpcEngine;
use markdown_parser::MarkdownParser;
use mcq::McqManager;
use natural_language_detection::NaturalLanguageDetector;
use resources::ResourceManager;
use settings::SettingsManager;
use shell::ShellManager as IcedShellManager;
use string_offset::StringOffsetManager;
use sum_tree::SumTreeManager;
use syntax_tree::SyntaxTreeManager;
use virtual_fs::VirtualFileSystem as IcedVirtualFileSystem;
use watcher::{Watcher as IcedWatcher, WatcherEvent};
use websocket::WebSocketServer;
use workflows::executor::{WorkflowExecutor, WorkflowExecutionEvent};
use workflows::manager::WorkflowManager as IcedWorkflowManager;
use collaboration::session_sharing::CollaborationEvent;
use workflows::Workflow;
use serve_wasm::WasmServer; // Import WasmServer

/// The main application state for NeoTerm.
#[derive(Debug, Clone)]
pub struct NeoTerm {
    /// List of UI blocks displayed in the terminal.
    blocks: Vec<Block>,
    /// The enhanced text input bar for commands and AI queries.
    input_bar: EnhancedTextInput,
    
    // Agent mode
    /// The AI agent mode instance.
    agent_mode: Arc<RwLock<AgentMode>>,
    /// Flag indicating if agent mode is currently enabled.
    agent_enabled: bool,
    /// Receiver for streaming messages from the AI agent.
    agent_streaming_rx: Option<mpsc::Receiver<AgentMessage>>,
    /// Map from AI tool_call_id to UI block_id for streaming tool calls.
    streaming_tool_call_blocks: HashMap<String, String>,
    
    // Configuration
    /// The current application configuration.
    config: AppConfig,
    /// Flag indicating if the settings panel is open.
    settings_open: bool,

    // Channels for PTY communication
    /// Sender for PTY messages (e.g., to send commands to PTY).
    pty_tx: mpsc::Sender<PtyMessage>,
    /// Receiver for PTY messages (e.g., to receive output from PTY).
    pty_rx: mpsc::Receiver<PtyMessage>,

    // Workflow execution
    /// The workflow executor instance.
    workflow_executor: Arc<WorkflowExecutor>,
    /// Receiver for workflow execution events.
    workflow_event_rx: mpsc::Receiver<WorkflowExecutionEvent>,

    // Managers (Arc'd for sharing)
    /// Manager for application configuration.
    config_manager: Arc<ConfigManager>,
    /// The AI assistant instance.
    ai_assistant: Arc<RwLock<Assistant>>,
    /// The AI context manager, providing relevant information to the AI.
    ai_context: Arc<RwLock<AIContext>>,
    /// Manager for workflows.
    workflow_manager: Arc<IcedWorkflowManager>,
    /// Manager for plugins.
    plugin_manager: Arc<IcedPluginManager>,
    /// Manager for cloud synchronization.
    sync_manager: Arc<SyncManager>,
    /// Manager for collaboration sessions.
    collaboration_manager: Arc<IcedSessionSharingManager>,
    /// Manager for command execution.
    command_manager: Arc<CommandManager>,
    /// Manager for drive operations.
    drive_manager: Arc<DriveManager>,
    /// Manager for fuzzy matching.
    fuzzy_match_manager: Arc<FuzzyMatchManager>,
    /// GraphQL schema for the API server.
    graphql_schema: Arc<graphql::AppSchema>,
    /// Manager for language-related functionalities.
    language_manager: Arc<LanguageManager>,
    /// LPC (Lars's Programming Language) engine.
    lpc_engine: Arc<LpcEngine>,
    /// Markdown parser.
    markdown_parser: Arc<MarkdownParser>,
    /// Multiple Choice Question manager.
    mcq_manager: Arc<McqManager>,
    /// Natural language detection manager.
    natural_language_detector: Arc<NaturalLanguageDetector>,
    /// Resource manager for static assets.
    resource_manager: Arc<ResourceManager>,
    /// Manager for application settings.
    settings_manager: Arc<SettingsManager>,
    /// Shell manager.
    shell_manager: Arc<IcedShellManager>,
    /// String offset manager.
    string_offset_manager: Arc<StringOffsetManager>,
    /// Sum tree manager.
    sum_tree_manager: Arc<SumTreeManager>,
    /// Syntax tree manager.
    syntax_tree_manager: Arc<SyntaxTreeManager>,
    /// Virtual file system manager.
    virtual_file_system: Arc<IcedVirtualFileSystem>,
    /// File watcher.
    watcher: Arc<IcedWatcher>,
    /// WebSocket server.
    websocket_server: Arc<WebSocketServer>,
    /// WebAssembly server.
    wasm_server: Arc<WasmServer>,
    /// User preferences.
    preferences: UserPreferences,
    /// Results of performance benchmarks.
    benchmark_results: Option<Vec<BenchmarkResult>>,
}

/// Messages that can be sent to the `NeoTerm` application.
#[derive(Debug, Clone)]
pub enum Message {
    /// Message from the input bar.
    Input(InputMessage),
    /// Trigger command execution.
    ExecuteCommand,
    /// Output received from a PTY session.
    PtyOutput(PtyMessage),
    /// Keyboard event.
    KeyboardEvent(keyboard::Event),
    /// Action performed on a UI block.
    BlockAction(String, BlockMessage),
    /// A periodic tick message for UI updates.
    Tick,
    
    // Agent mode messages
    /// Toggle AI agent mode on/off.
    ToggleAgentMode,
    /// Streamed message from the AI agent.
    AgentStream(AgentMessage),
    /// Indicates the AI agent stream has ended.
    AgentStreamEnded,
    /// An error occurred in the AI agent.
    AgentError(String),
    /// A command was generated by the AI.
    CommandGenerated(String),
    /// A suggested fix for a failed command from the AI.
    SuggestedFix(String),
    /// AI usage quota updated.
    UsageQuotaUpdated(String),
    
    // Settings messages
    /// Toggle the settings panel open/closed.
    ToggleSettings,
    /// Message from the settings view.
    SettingsMessage(settings::SettingsMessage),
    
    // Configuration
    /// Application configuration has been loaded.
    ConfigLoaded(AppConfig),
    /// Application configuration has been saved.
    ConfigSaved,

    // Performance Benchmarks
    /// Trigger running performance benchmarks.
    RunBenchmarks,
    /// Results of performance benchmarks.
    BenchmarkResults(BenchmarkSuite),

    // Workflow messages
    /// Event from workflow execution.
    WorkflowExecutionEvent(WorkflowExecutionEvent),
    /// User's response to an agent prompt.
    UserResponseToAgentPrompt(String, String),
}

/// Messages related to PTY (Pseudo-Terminal) operations.
#[derive(Debug, Clone)]
pub enum PtyMessage {
    /// A chunk of output from the PTY.
    OutputChunk {
        block_id: String,
        content: String,
        is_stdout: bool,
    },
    /// Command completed with an exit code.
    Completed {
        block_id: String,
        exit_code: i32,
    },
    /// Command failed with an error message.
    Failed {
        block_id: String,
        error: String,
    },
    /// Command was killed.
    Killed {
        block_id: String,
    },
}

/// Messages related to individual UI blocks.
#[derive(Debug, Clone)]
pub enum BlockMessage {
    /// Rerun the command associated with the block.
    Rerun,
    /// Delete the block from the UI.
    Delete,
    /// Copy the content of the block to clipboard.
    Copy,
    /// Export the content of the block.
    Export,
    /// Toggle the collapsed state of the block.
    ToggleCollapse,
    /// Send the block's content to the AI for analysis.
    SendToAI,
    /// Request AI to suggest a fix for a failed command block.
    SuggestFix,
    /// Request AI to explain the output of a command or error block.
    ExplainOutput,
    /// Accept a suggested workflow.
    AcceptWorkflow,
    /// Reject a suggested workflow.
    RejectWorkflow,
    /// User input changed for an agent prompt block.
    AgentPromptInputChanged(String),
    /// Submit the response for an agent prompt block.
    SubmitAgentPrompt,
    /// Fetch AI usage quota.
    FetchUsageQuota,
}

impl Application for NeoTerm {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    /// Initializes the application state and returns the initial command.
    ///
    /// This function sets up all core managers, AI components, and communication channels.
    /// It also adds initial sample blocks to the UI.
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Initialize channels for inter-module communication
        let (pty_tx, pty_rx) = mpsc::channel(100);
        let (workflow_event_tx, workflow_event_rx) = mpsc::channel(100);
        let (command_event_tx, _command_event_rx) = mpsc::channel(100); // CommandManager sends events, NeoTerm doesn't directly receive them here

        // Initialize core managers
        let config = AppConfig::load().unwrap_or_default();
        let preferences = config.preferences.clone();
        let config_manager = Arc::new(tokio::runtime::Handle::current().block_on(async {
            ConfigManager::new().await.expect("Failed to initialize ConfigManager")
        }));
        
        let command_manager = Arc::new(CommandManager::new(command_event_tx));
        let virtual_file_system = Arc::new(VirtualFileSystem::new());
        let watcher = Arc::new(Watcher::new(mpsc::channel(100).0)); // Dummy sender for watcher events
        let resource_manager = Arc::new(ResourceManager::new());
        let plugin_manager = Arc::new(PluginManager::new(mpsc::unbounded_channel().0)); // Dummy sender for plugin events
        let shell_manager = Arc::new(ShellManager::new());
        let drive_manager = Arc::new(DriveManager::new(Default::default(), mpsc::channel(100).0)); // Dummy sender for drive events
        let websocket_server = Arc::new(WebSocketServer::new());
        let lpc_engine = Arc::new(LpcEngine::new(mpsc::channel(100).0)); // Dummy sender for LPC events
        let mcq_manager = Arc::new(McqManager::new());
        let natural_language_detector = Arc::new(NaturalLanguageDetector::new());
        let syntax_tree_manager = Arc::new(SyntaxTreeManager::new());
        let string_offset_manager = Arc::new(StringOffsetManager::new());
        let sum_tree_manager = Arc::new(SumTreeManager::new());
        let fuzzy_match_manager = Arc::new(FuzzyMatchManager::new());
        let markdown_parser = Arc::new(MarkdownParser::new());
        let language_manager = Arc::new(LanguageManager::new());
        let settings_manager = Arc::new(SettingsManager::new(config_manager.clone()));
        let collaboration_manager = Arc::new(SessionSharingManager::new(mpsc::channel(100).0)); // Dummy sender for collab events
        let sync_manager = Arc::new(SyncManager::new(Default::default(), mpsc::channel(100).0)); // Dummy sender for sync events
        let wasm_server = Arc::new(WasmServer::new());

        // Initialize AI Context
        let ai_context = Arc::new(RwLock::new(AIContext::new(
            virtual_file_system.clone(),
            command_manager.clone(),
            watcher.clone(),
            resource_manager.clone(),
            plugin_manager.clone(),
            shell_manager.clone(),
            drive_manager.clone(),
            websocket_server.clone(),
            lpc_engine.clone(),
            mcq_manager.clone(),
            natural_language_detector.clone(),
            syntax_tree_manager.clone(),
            string_offset_manager.clone(),
            sum_tree_manager.clone(),
            fuzzy_match_manager.clone(),
            markdown_parser.clone(),
            language_manager.clone(),
            settings_manager.clone(),
            collaboration_manager.clone(),
            sync_manager.clone(),
            wasm_server.clone(),
        )));

        // Initialize AI Assistant with AIContext
        let ai_assistant = Arc::new(RwLock::new(tokio::runtime::Handle::current().block_on(async {
            Assistant::new(
                command_manager.clone(),
                virtual_file_system.clone(),
                watcher.clone(),
                ai_context.clone(), // Pass AIContext here
                &preferences.ai_provider_type,
                preferences.ai_api_key.clone(),
                preferences.ai_model.clone(),
                preferences.fallback_ai_provider_type.clone(),
                None, // Fallback API key is not currently stored in preferences, assuming same as primary or none
                preferences.fallback_ai_model.clone(),
                preferences.redact_sensitive_info,
                preferences.local_only_ai_mode,
            ).expect("Failed to initialize AI Assistant")
        })));

        // Initialize AgentMode with the assistant and AIContext
        let agent_config = {
            let mut cfg = AgentConfig::default();
            if let Some(api_key) = std::env::var("OPENAI_API_KEY").ok() {
                cfg.api_key = Some(api_key);
            }
            cfg
        };
        let agent_mode = Arc::new(RwLock::new(AgentMode::new(agent_config, ai_assistant.clone(), ai_context.clone()).expect("Failed to initialize AgentMode")));

        // Start the API server (if enabled in preferences)
        if preferences.enable_graphql_api {
            let agent_mode_clone = agent_mode.clone();
            tokio::spawn(async move {
                api::start_api_server(agent_mode_clone).await;
            });
        }

        // Initialize WorkflowExecutor
        let workflow_executor = Arc::new(WorkflowExecutor::new(
            command_manager.clone(),
            virtual_file_system.clone(),
            agent_mode.clone(),
            resource_manager.clone(),
            plugin_manager.clone(),
            shell_manager.clone(),
            drive_manager.clone(),
            watcher.clone(),
            websocket_server.clone(),
            lpc_engine.clone(),
            mcq_manager.clone(),
            natural_language_detector.clone(),
            syntax_tree_manager.clone(),
            string_offset_manager.clone(),
            sum_tree_manager.clone(),
            fuzzy_match_manager.clone(),
            markdown_parser.clone(),
            language_manager.clone(),
            settings_manager.clone(),
            collaboration_manager.clone(),
            sync_manager.clone(),
            wasm_server.clone(),
        ));
        // Set the event sender for the executor
        let workflow_executor_clone = workflow_executor.clone();
        tokio::spawn(async move {
            let mut executor_lock = workflow_executor_clone.clone();
            executor_lock.set_event_sender(workflow_event_tx);
        });

        let mut neo_term = Self {
            blocks: Vec::new(),
            input_bar: EnhancedTextInput::new(),
            agent_mode,
            agent_enabled: false,
            agent_streaming_rx: None,
            config,
            settings_open: false,
            pty_tx,
            pty_rx,
            workflow_executor,
            workflow_event_rx,
            config_manager,
            ai_assistant,
            ai_context,
            workflow_manager: Arc::new(WorkflowManager::new()),
            plugin_manager,
            sync_manager,
            collaboration_manager,
            command_manager,
            drive_manager,
            fuzzy_match_manager,
            graphql_schema: Arc::new(build_schema()),
            language_manager,
            lpc_engine,
            markdown_parser,
            mcq_manager,
            natural_language_detector,
            resource_manager,
            settings_manager,
            shell_manager,
            string_offset_manager,
            sum_tree_manager,
            syntax_tree_manager,
            virtual_file_system,
            watcher,
            websocket_server,
            wasm_server,
            preferences,
            benchmark_results: None,
            streaming_tool_call_blocks: HashMap::new(), // Initialize new field
        };

        neo_term.add_sample_blocks();

        (
            neo_term,
            Command::none(),
        )
    }

    /// Returns the title of the application window.
    fn title(&self) -> String {
        if self.agent_enabled {
            "NeoTerm - Agent Mode".to_string()
        } else {
            "NeoTerm".to_string()
        }
    }

    /// Updates the application state based on incoming messages.
    ///
    /// This is the central update loop for the Iced application, handling
    /// all user interactions, backend events, and state transitions.
    ///
    /// # Arguments
    ///
    /// * `message` - The `Message` to process.
    ///
    /// # Returns
    ///
    /// An `iced::Command` to be executed by the runtime.
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Input(input_message) => {
                match input_message {
                    InputMessage::Submit => {
                        let command = self.input_bar.value().to_string();
                        self.input_bar.update(InputMessage::Submit);
                        if !command.trim().is_empty() {
                            if command.starts_with('#') || command.starts_with("/ai") {
                                self.handle_ai_command(command, None)
                            } else {
                                self.execute_command(command)
                            }
                        } else {
                            Command::none()
                        }
                    }
                    _ => {
                        self.input_bar.update(input_message);
                        Command::none()
                    }
                }
            }
            Message::ExecuteCommand => {
                Command::none()
            }
            Message::PtyOutput(pty_msg) => {
                if let Some(block) = self.blocks.iter_mut().find(|b| b.id == pty_msg.get_block_id()) {
                    match pty_msg {
                        PtyMessage::OutputChunk { content, is_stdout, .. } => {
                            block.add_output_line(content, is_stdout);
                        }
                        PtyMessage::Completed { exit_code, block_id: _ } => {
                            block.set_status(format!("Completed with exit code: {}", exit_code));
                            if exit_code != 0 {
                                block.set_error(true);
                                // Trigger AI fix suggestion
                                if let BlockContent::Command { input, output, .. } = &block.content {
                                    let original_command = input.clone();
                                    let error_output = output.iter()
                                        .filter(|(_, is_stdout)| !is_stdout) // Filter for stderr
                                        .map(|(s, _)| s.clone())
                                        .join("\n");
                                    let error_msg = if error_output.is_empty() {
                                        format!("Command exited with non-zero code: {}", exit_code)
                                    } else {
                                        format!("Error output:\n{}", error_output)
                                    };

                                    let agent_mode_arc_clone = self.agent_mode.clone();
                                    return Command::perform(
                                        async move {
                                            let mut agent_mode = agent_mode_arc_clone.write().await;
                                            match agent_mode.fix(&original_command, &error_msg).await {
                                                Ok(suggested_command) => Message::SuggestedFix(suggested_command),
                                                Err(e) => Message::AgentError(format!("Failed to get fix suggestion: {}", e)),
                                            }
                                        },
                                        |msg| msg
                                    );
                                }
                            }
                        }
                        PtyMessage::Failed { error, block_id: _ } => {
                            block.set_status(format!("Failed: {}", error));
                            block.set_error(true);
                            // Trigger AI fix suggestion
                            if let BlockContent::Command { input, .. } = &block.content {
                                let original_command = input.clone();
                                let agent_mode_arc_clone = self.agent_mode.clone();
                                return Command::perform(
                                    async move {
                                        let mut agent_mode = agent_mode_arc_clone.write().await;
                                        match agent_mode.fix(&original_command, &error).await {
                                            Ok(suggested_command) => Message::SuggestedFix(suggested_command),
                                            Err(e) => Message::AgentError(format!("Failed to get fix suggestion: {}", e)),
                                        }
                                    },
                                    |msg| msg
                                );
                            }
                        }
                        PtyMessage::Killed { block_id: _ } => {
                            block.set_status("Killed".to_string());
                            block.set_error(true);
                        }
                    }
                }
                Command::none()
            }
            Message::ToggleAgentMode => {
                let agent_mode_arc_clone = self.agent_mode.clone();
                Command::perform(
                    async move {
                        let mut agent_mode = agent_mode_arc_clone.write().await;
                        let enabled = agent_mode.toggle();
                        if enabled {
                            if let Ok(_) = agent_mode.start_conversation().await {
                                Some("Agent mode activated. How can I help you?".to_string())
                            } else {
                                None
                            }
                        } else {
                            Some("Agent mode deactivated.".to_string())
                        }
                    },
                    |msg| {
                        if let Some(content) = msg {
                            Message::AgentStream(AgentMessage::SystemMessage(content))
                        } else {
                            Message::AgentError("Failed to start agent conversation.".to_string())
                        }
                    }
                )
            }
            Message::AgentStream(agent_msg) => {
                match agent_msg {
                    AgentMessage::UserMessage(content) => {
                        let block = Block::new_user_message(content);
                        self.blocks.push(block);
                    }
                    AgentMessage::AgentResponse(content) => {
                        if let Some(last_block) = self.blocks.last_mut() {
                            if let BlockContent::AgentMessage { ref mut content: block_content, .. } = last_block.content {
                                block_content.push_str(&content);
                            } else {
                                // If the last block isn't an agent message, create a new one
                                let mut new_block = Block::new_agent_message(content);
                                new_block.set_status("Streaming...".to_string());
                                self.blocks.push(new_block);
                            }
                        } else {
                            // No blocks yet, create a new agent message block
                            let mut new_block = Block::new_agent_message(content);
                            new_block.set_status("Streaming...".to_string());
                            self.blocks.push(new_block);
                        }
                    }
                    AgentMessage::ToolCall(tool_call) => {
                        let tool_call_id = tool_call.id.clone();
                        let arguments_str = tool_call.function.arguments.to_string(); // Convert Value to string for display

                        if let Some(block_id) = self.streaming_tool_call_blocks.get(&tool_call_id) {
                            // Update existing streaming tool call block
                            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == *block_id) {
                                if let BlockContent::StreamingToolCall { arguments: ref mut current_args, .. } = block.content {
                                    // Update arguments. A more robust solution might parse JSON and merge.
                                    *current_args = arguments_str;
                                    block.set_status(format!("Streaming Tool Call: {}", tool_call.function.name));
                                }
                            }
                        } else {
                            // Create a new streaming tool call block
                            let mut new_block = Block::new_streaming_tool_call(
                                tool_call_id.clone(),
                                tool_call.function.name.clone(),
                                arguments_str.clone(),
                            );
                            let new_block_id = new_block.id.clone();
                            self.blocks.push(new_block);
                            self.streaming_tool_call_blocks.insert(tool_call_id.clone(), new_block_id);
                        }

                        // Check if arguments are complete (e.g., valid JSON object/array)
                        // This is a heuristic to determine if streaming for this tool call is done.
                        if tool_call.function.arguments.is_object() || tool_call.function.arguments.is_array() {
                            if let Some(block_id) = self.streaming_tool_call_blocks.remove(&tool_call_id) {
                                if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
                                    // Transition to a regular Info block
                                    block.content = BlockContent::Info {
                                        title: format!("AI Tool Call: {}", tool_call.function.name),
                                        message: format!("Arguments: {}", tool_call.function.arguments.to_string()),
                                        timestamp: Local::now(),
                                    };
                                    block.set_status("Tool Call Completed".to_string());
                                }
                            }
                        }
                    }
                    AgentMessage::ToolResult(result) => {
                        let block = Block::new_info(
                            "AI Tool Result".to_string(),
                            result
                        );
                        self.blocks.push(block);
                    }
                    AgentMessage::SystemMessage(content) => {
                        let block = Block::new_info("System Message".to_string(), content);
                        self.blocks.push(block);
                    }
                    AgentMessage::Done => {
                        if let Some(last_block) = self.blocks.last_mut() {
                            if let BlockContent::AgentMessage { .. } = last_block.content {
                                last_block.set_status("Completed".to_string());
                            }
                        }
                        // Clear any remaining streaming tool call blocks if the stream ends
                        self.streaming_tool_call_blocks.clear();
                        self.agent_streaming_rx = None; // Mark stream as ended
                    }
                    AgentMessage::WorkflowSuggested(workflow) => {
                        let block = Block::new_workflow_suggestion(workflow);
                        self.blocks.push(block);
                        self.agent_streaming_rx = None; // Workflow suggestion ends the current AI stream
                    }
                    AgentMessage::AgentPromptRequest { prompt_id, message } => {
                        // Find the existing agent prompt block if it exists, or create a new one
                        if let Some(block) = self.blocks.iter_mut().find(|b| {
                            if let BlockContent::AgentPrompt { prompt_id: existing_prompt_id, .. } = &b.content {
                                existing_prompt_id == &prompt_id
                            } else {
                                false
                            }
                        }) {
                            if let BlockContent::AgentPrompt { message: ref mut msg, .. } = block.content {
                                *msg = message; // Update message if needed
                            }
                        } else {
                            let block = Block::new_agent_prompt(prompt_id, message);
                            self.blocks.push(block);
                        }
                    }
                    AgentMessage::AgentPromptResponse { .. } => {
                        // This message is handled internally by AgentMode, not displayed directly
                        Command::none()
                    }
                }
                Command::none()
            }
            Message::AgentStreamEnded => {
                self.agent_streaming_rx = None;
                self.streaming_tool_call_blocks.clear(); // Ensure cleanup
                Command::none()
            }
            Message::AgentError(error) => {
                let block = Block::new_error(format!("Agent error: {}", error));
                self.blocks.push(block);
                self.agent_streaming_rx = None;
                self.streaming_tool_call_blocks.clear(); // Ensure cleanup
                Command::none()
            }
            Message::CommandGenerated(generated_command) => {
                // Auto-fill the input bar with the generated command
                self.input_bar.update(InputMessage::InputChanged(generated_command.clone()));
                // Optionally, add an info block that the command was generated
                let info_block = Block::new_info(
                    "AI Generated Command".to_string(),
                    format!("The command has been auto-filled into the input bar: `{}`. Press Enter to execute.", generated_command)
                );
                self.blocks.push(info_block);
                Command::none()
            }
            Message::SuggestedFix(suggested_command) => {
                // Auto-fill the input bar with the suggested command
                self.input_bar.update(InputMessage::InputChanged(suggested_command.clone()));
                let info_block = Block::new_info(
                    "AI Suggested Fix".to_string(),
                    format!("AI suggested a fix for the last failed command. It has been auto-filled into the input bar: `{}`. Press Enter to execute.", suggested_command)
                );
                self.blocks.push(info_block);
                Command::none()
            }
            Message::UsageQuotaUpdated(quota_info) => {
                let info_block = Block::new_info("AI Usage Quota".to_string(), quota_info);
                self.blocks.push(info_block);
                Command::none()
            }
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                Command::none()
            }
            Message::BlockAction(block_id, action) => {
                self.handle_block_action(block_id, action)
            }
            Message::Tick => {
                Command::none()
            }
            Message::KeyboardEvent(event) => {
                match event {
                    keyboard::Event::KeyPressed { key_code, modifiers, .. } => {
                        match key_code {
                            KeyCode::Up => {
                                self.input_bar.update(InputMessage::HistoryNavigated(HistoryDirection::Up));
                            }
                            KeyCode::Down => {
                                self.input_bar.update(InputMessage::HistoryNavigated(HistoryDirection::Down));
                            }
                            KeyCode::Tab => {
                                self.input_bar.update(InputMessage::NavigateSuggestions(Direction::Down));
                                self.input_bar.update(InputMessage::ApplySuggestion);
                            }
                            KeyCode::F1 => {
                                return Command::perform(async {}, |_| Message::RunBenchmarks);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                Command::none()
            }
            Message::ConfigLoaded(_) => Command::none(),
            Message::ConfigSaved => Command::none(),
            Message::SettingsMessage(msg) => {
                let mut settings_view = settings::SettingsView::new(self.config.clone());
                settings_view.update(msg);
                self.config = settings_view.config;
                Command::none()
            }
            Message::RunBenchmarks => {
                let workflow_manager_clone = self.workflow_manager.clone();
                Command::perform(
                    async move {
                        let mut benchmarks_runner = PerformanceBenchmarks::new(workflow_manager_clone);
                        benchmarks_runner.run_all_benchmarks().await
                    },
                    Message::BenchmarkResults,
                )
            }
            Message::BenchmarkResults(suite) => {
                let summary = suite.get_performance_summary();
                let block = Block::new_info("Performance Benchmark Results".to_string(), summary);
                self.blocks.push(block);
                Command::none()
            }
            Message::WorkflowExecutionEvent(event) => {
                match event {
                    WorkflowExecutionEvent::Started { workflow_id, name } => {
                        let block = Block::new_info("Workflow Started".to_string(), format!("Workflow '{}' (ID: {}) started.", name, workflow_id));
                        self.blocks.push(block);
                    }
                    WorkflowExecutionEvent::StepStarted { workflow_id: _, step_id: _, name } => {
                        let block = Block::new_info("Workflow Step Started".to_string(), format!("Step '{}' started.", name));
                        self.blocks.push(block);
                    }
                    WorkflowExecutionEvent::StepCompleted { workflow_id: _, step_id: _, name, output } => {
                        let block = Block::new_info("Workflow Step Completed".to_string(), format!("Step '{}' completed. Output:\n{}", name, output));
                        self.blocks.push(block);
                    }
                    WorkflowExecutionEvent::StepFailed { workflow_id: _, step_id: _, name, error } => {
                        let block = Block::new_error(format!("Workflow Step '{}' failed: {}", name, error));
                        self.blocks.push(block);
                    }
                    WorkflowExecutionEvent::Completed { workflow_id, name, success } => {
                        let status = if success { "successfully" } else { "with errors" };
                        let block = Block::new_info("Workflow Completed".to_string(), format!("Workflow '{}' (ID: {}) completed {}.", name, workflow_id, status));
                        self.blocks.push(block);
                    }
                    WorkflowExecutionEvent::Error { workflow_id, message } => {
                        let block = Block::new_error(format!("Workflow execution error for ID {}: {}", workflow_id, message));
                        self.blocks.push(block);
                    }
                    WorkflowExecutionEvent::AgentPromptRequest { workflow_id: _, step_id: _, prompt_id, message } => {
                        // Find the existing agent prompt block if it exists, or create a new one
                        if let Some(block) = self.blocks.iter_mut().find(|b| {
                            if let BlockContent::AgentPrompt { prompt_id: existing_prompt_id, .. } = &b.content {
                                existing_prompt_id == &prompt_id
                            } else {
                                false
                            }
                        }) {
                            if let BlockContent::AgentPrompt { message: ref mut msg, .. } = block.content {
                                *msg = message; // Update message if needed
                            }
                        } else {
                            let block = Block::new_agent_prompt(prompt_id, message);
                            self.blocks.push(block);
                        }
                    }
                }
                Command::none()
            }
            Message::UserResponseToAgentPrompt(prompt_id, response) => {
                let agent_mode_arc_clone = self.agent_mode.clone();
                Command::perform(
                    async move {
                        let mut agent_mode = agent_mode_arc_clone.write().await;
                        match agent_mode.handle_agent_prompt_response(prompt_id, response).await {
                            Ok(_) => Message::Tick, // Just a dummy message to trigger update
                            Err(e) => Message::AgentError(format!("Failed to send agent prompt response: {}", e)),
                        }
                    },
                    |msg| msg
                )
            }
        }
    }

    /// Renders the main application UI.
    ///
    /// This function constructs the Iced UI, including the toolbar,
    /// the scrollable list of blocks, and the input bar.
    ///
    /// # Returns
    ///
    /// An `iced::Element` representing the application's view.
    fn view(&self) -> Element<Message> {
        if self.settings_open {
            let mut settings_view = settings::SettingsView::new(self.config.clone());
            return settings_view.view().map(Message::SettingsMessage);
        }

        let blocks_view = scrollable(
            column(
                self.blocks
                    .iter()
                    .map(|block| block.view().map(|msg| Message::BlockAction(block.id.clone(), msg)))
                    .collect::<Vec<_>>()
            )
            .spacing(8)
        )
        .height(iced::Length::Fill);

        let prompt_indicator = if self.agent_enabled {
            "🤖 "
        } else {
            "$ "
        };

        let placeholder = if self.agent_enabled {
            "Ask me anything or enter a command..."
        } else {
            "Enter command..."
        };

        let input_view = self.input_bar.view(prompt_indicator, placeholder).map(Message::Input);

        let toolbar = self.create_toolbar();

        column![toolbar, blocks_view, input_view]
            .spacing(8)
            .padding(16)
            .into()
    }

    /// Defines the application's subscriptions to external events.
    ///
    /// This includes periodic ticks, PTY output, keyboard events,
    /// AI agent streams, and workflow execution events.
    ///
    /// # Returns
    ///
    /// An `iced::Subscription` that the runtime will listen to.
    fn subscription(&self) -> iced::Subscription<Message> {
        let agent_stream_sub = if let Some(rx) = self.agent_streaming_rx.clone() {
            iced::Subscription::unfold(
                "agent_stream",
                rx,
                |mut receiver| async move {
                    match receiver.recv().await {
                        Some(msg) => (Message::AgentStream(msg), receiver),
                        None => (Message::AgentStreamEnded, receiver),
                    }
                },
            )
        } else {
            iced::Subscription::none()
        };

        iced::Subscription::batch(vec![
            iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick),
            self.pty_manager_subscription(),
            keyboard::Event::all().map(Message::KeyboardEvent),
            agent_stream_sub,
            self.workflow_executor_subscription(),
        ])
    }
}

impl NeoTerm {
    /// Creates the application toolbar with various action buttons.
    ///
    /// # Returns
    ///
    /// An `iced::Element` representing the toolbar.
    fn create_toolbar(&self) -> Element<Message> {
        let agent_button = button(
            text(if self.agent_enabled { "🤖 Agent ON" } else { "🤖 Agent OFF" })
        )
        .on_press(Message::ToggleAgentMode);

        let settings_button = button(text("⚙️ Settings"))
            .on_press(Message::ToggleSettings);

        let usage_button = button(text("📊 Usage"))
            .on_press(Message::BlockAction("".to_string(), BlockMessage::FetchUsageQuota));

        let active_profile_name = self.config.env_profiles.active_profile.as_deref().unwrap_or("None");
        let env_profile_indicator = text(format!("Env: {}", active_profile_name)).size(14);

        row![agent_button, settings_button, usage_button, env_profile_indicator]
            .spacing(8)
            .into()
    }

    /// Handles AI-related commands entered by the user.
    ///
    /// This function distinguishes between direct AI chat, command generation requests,
    /// and contextual AI analysis based on a provided block ID.
    ///
    /// # Arguments
    ///
    /// * `command` - The full command string entered by the user (e.g., "#explain this").
    /// * `context_block_id` - An optional ID of a UI block to provide as context to the AI.
    ///
    /// # Returns
    ///
    /// An `iced::Command` to initiate AI interaction.
    fn handle_ai_command(&mut self, command: String, context_block_id: Option<String>) -> Command<Message> {
        let prompt_content = command.trim_start_matches('#').trim_start_matches("/ai").trim().to_string();
        
        let user_block = Block::new_user_message(command.clone());
        self.blocks.push(user_block);
        
        let mut context_blocks = Vec::new();
        if let Some(id) = context_block_id {
            if let Some(block) = self.blocks.iter().find(|b| b.id == id) {
                context_blocks.push(block.clone());
            }
        }

        let agent_mode_arc_clone = self.agent_mode.clone();
        let (tx, rx) = mpsc::channel(100);
        self.agent_streaming_rx = Some(rx);

        if prompt_content.to_lowercase().starts_with("generate command for") ||
           prompt_content.to_lowercase().starts_with("create command to") ||
           prompt_content.to_lowercase().starts_with("command to") {
            
            let natural_language_query = prompt_content
                .trim_start_matches("generate command for")
                .trim_start_matches("create command to")
                .trim_start_matches("command to")
                .trim()
                .to_string();

            Command::perform(
                async move {
                    let mut agent_mode = agent_mode_arc_clone.write().await;
                    match agent_mode.generate_command(&natural_language_query).await {
                        Ok(generated_command) => {
                            Message::CommandGenerated(generated_command)
                        }
                        Err(e) => {
                            Message::AgentError(format!("Failed to generate command: {}", e))
                        }
                    }
                },
                |msg| msg
            )
        } else {
            Command::perform(
                async move {
                    let mut agent_mode = agent_mode_arc_clone.write().await;
                    match agent_mode.send_message(prompt_content, context_blocks).await {
                        Ok(mut stream_rx) => {
                            while let Some(msg) = stream_rx.recv().await {
                                if tx.send(msg).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AgentMessage::Error(format!("Failed to send message to agent: {}", e))).await;
                        }
                    }
                },
                |_| Message::Tick
            )
        }
    }

    /// Handles actions performed on individual UI blocks.
    ///
    /// This function processes various `BlockMessage` types, such as rerunning commands,
    /// deleting blocks, copying content, exporting, toggling collapse, and AI interactions.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The ID of the block on which the action is performed.
    /// * `action` - The `BlockMessage` representing the action.
    ///
    /// # Returns
    ///
    /// An `iced::Command` to update the UI or trigger backend operations.
    fn handle_block_action(&mut self, block_id: String, action: BlockMessage) -> Command<Message> {
        if let Some(block_index) = self.blocks.iter().position(|b| b.id == block_id) {
            let block = &mut self.blocks[block_index];
            match action {
                BlockMessage::Rerun => {
                    if let BlockContent::Command { input, .. } = &block.content {
                        let command = input.clone();
                        self.execute_command(command)
                    } else {
                        Command::none()
                    }
                }
                BlockMessage::Delete => {
                    self.blocks.retain(|b| b.id != block_id);
                    Command::none()
                }
                BlockMessage::Copy => {
                    // Mock implementation for copy to clipboard
                    let content_to_copy = match &block.content {
                        BlockContent::Command { input, output, .. } => {
                            format!("Command: {}\nOutput:\n{}", input, output.iter().map(|(s, _)| s.clone()).join("\n"))
                        },
                        BlockContent::AgentMessage { content, .. } => content.clone(),
                        BlockContent::Info { message, .. } => message.clone(),
                        BlockContent::Error { message, .. } => message.clone(),
                        BlockContent::WorkflowSuggestion { workflow } => format!("{:#?}", workflow),
                        BlockContent::AgentPrompt { message, .. } => message.clone(),
                        BlockContent::StreamingToolCall { name, arguments, .. } => format!("Tool Call: {}\nArguments: {}", name, arguments),
                    };
                    log::info!("Mock Copy: Copied content to clipboard (not actually implemented): {}", content_to_copy);
                    // In a real app, you'd use a platform-specific clipboard API
                    Command::none()
                }
                BlockMessage::Export => {
                    // Mock implementation for export functionality
                    let export_content = match &block.content {
                        BlockContent::Command { input, output, .. } => {
                            format!("Command: {}\nOutput:\n{}", input, output.iter().map(|(s, _)| s.clone()).join("\n"))
                        },
                        BlockContent::AgentMessage { content, .. } => content.clone(),
                        BlockContent::Info { message, .. } => message.clone(),
                        BlockContent::Error { message, .. } => message.clone(),
                        BlockContent::WorkflowSuggestion { workflow } => format!("{:#?}", workflow),
                        BlockContent::AgentPrompt { message, .. } => message.clone(),
                        BlockContent::StreamingToolCall { name, arguments, .. } => format!("Tool Call: {}\nArguments: {}", name, arguments),
                    };
                    log::info!("Mock Export: Exported content (not actually implemented):\n{}", export_content);
                    // In a real app, you'd open a save dialog or write to a file
                    Command::none()
                }
                BlockMessage::ToggleCollapse => {
                    block.toggle_collapse();
                    Command::none()
                }
                BlockMessage::SendToAI => {
                    let block_to_send = block.clone();
                    let user_prompt_for_ai = "Please analyze the provided context."; // A generic prompt
                    
                    // Add a user message block to the UI to indicate AI interaction
                    let user_block = Block::new_user_message(format!("AI: Analyze block #{}", &block_to_send.id[0..8]));
                    self.blocks.push(user_block);

                    let agent_mode_arc_clone = self.agent_mode.clone();
                    let (tx, rx) = mpsc::channel(100);
                    self.agent_streaming_rx = Some(rx);

                    // Send the generic prompt and the specific block as context
                    return Command::perform(
                        async move {
                            let mut agent_mode = agent_mode_arc_clone.write().await;
                            match agent_mode.send_message(user_prompt_for_ai.to_string(), vec![block_to_send]).await {
                                Ok(mut stream_rx) => {
                                    while let Some(msg) = stream_rx.recv().await {
                                        if tx.send(msg).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(AgentMessage::Error(format!("Failed to send message to agent: {}", e))).await;
                                }
                            }
                        },
                        |_| Message::Tick
                    );
                }
                BlockMessage::SuggestFix => {
                    if let BlockContent::Command { input, output, status, error, .. } = &block.content {
                        let original_command = input.clone();
                        let error_output = output.iter()
                            .filter(|(_, is_stdout)| !is_stdout)
                            .map(|(s, _)| s.clone())
                            .join("\n");
                        let error_msg = if *error {
                            if error_output.is_empty() {
                                status.clone()
                            } else {
                                format!("Error output:\n{}", error_output)
                            }
                        } else {
                            "No specific error message available, but command failed.".to_string()
                        };

                        let agent_mode_arc_clone = self.agent_mode.clone();
                        return Command::perform(
                            async move {
                                let mut agent_mode = agent_mode_arc_clone.write().await;
                                match agent_mode.fix(&original_command, &error_msg).await {
                                    Ok(suggested_command) => Message::SuggestedFix(suggested_command),
                                    Err(e) => Message::AgentError(format!("Failed to get fix suggestion: {}", e)),
                                }
                            },
                            |msg| msg
                        );
                    }
                    Command::none()
                }
                BlockMessage::ExplainOutput => {
                    let agent_mode_arc_clone = self.agent_mode.clone();
                    let (command_input, output_content, error_message) = match &block.content {
                        BlockContent::Command { input, output, error, .. } => {
                            let full_output = output.iter().map(|(s, _)| s.clone()).join("\n");
                            let stderr_output = output.iter().filter(|(_, is_stdout)| !is_stdout).map(|(s, _)| s.clone()).join("\n");
                            let err_msg = if *error && !stderr_output.is_empty() {
                                Some(stderr_output)
                            } else if *error {
                                Some("Command failed with non-zero exit code.".to_string())
                            } else {
                                None
                            };
                            (input.clone(), full_output, err_msg)
                        },
                        BlockContent::Error { message, .. } => {
                            ("".to_string(), "".to_string(), Some(message.clone()))
                        },
                        _ => {
                            return Command::none();
                        }
                    };

                    return Command::perform(
                        async move {
                            let mut agent_mode = agent_mode_arc_clone.write().await;
                            match agent_mode.explain_output(&command_input, &output_content, error_message.as_deref()).await {
                                Ok(explanation) => Message::AgentStream(AgentMessage::SystemMessage(explanation)),
                                Err(e) => Message::AgentError(format!("Failed to get explanation: {}", e)),
                            }
                        },
                        |msg| msg
                    );
                }
                BlockMessage::AcceptWorkflow => {
                    if let BlockContent::WorkflowSuggestion { workflow } = &block.content {
                        let workflow_to_execute = workflow.clone();
                        let workflow_executor_clone = self.workflow_executor.clone();
                        let workflow_manager_clone = self.workflow_manager.clone();
                        let block_id_clone = block_id.clone();

                        self.blocks.retain(|b| b.id != block_id_clone);

                        return Command::perform(
                            async move {
                                let mut workflow_manager = workflow_manager_clone.clone(); // Clone for mutable access
                                let save_result = workflow_manager.save_workflow(workflow_to_execute.clone()).await;
                                if let Err(e) = save_result {
                                    log::error!("Failed to save AI-generated workflow: {}", e);
                                    return Message::AgentError(format!("Failed to save workflow: {}", e));
                                }

                                match workflow_executor_clone.execute_workflow(workflow_to_execute, Vec::new()).await {
                                    Ok(_) => Message::Tick,
                                    Err(e) => Message::AgentError(format!("Failed to execute workflow: {}", e)),
                                }
                            },
                            |msg| msg
                        );
                    }
                    Command::none()
                }
                BlockMessage::RejectWorkflow => {
                    self.blocks.retain(|b| b.id != block_id);
                    Command::none()
                }
                BlockMessage::AgentPromptInputChanged(new_value) => {
                    if let BlockContent::AgentPrompt { input_value: ref mut current_value, .. } = &mut block.content {
                        *current_value = new_value;
                    }
                    Command::none()
                }
                BlockMessage::SubmitAgentPrompt => {
                    if let BlockContent::AgentPrompt { prompt_id, input_value, .. } = &block.content {
                        let prompt_id_clone = prompt_id.clone();
                        let response_clone = input_value.clone();
                        
                        self.blocks.retain(|b| b.id != block_id);

                        return Command::perform(
                            async move {
                                Message::UserResponseToAgentPrompt(prompt_id_clone, response_clone)
                            },
                            |msg| msg
                        );
                    }
                    Command::none()
                }
                BlockMessage::FetchUsageQuota => {
                    let ai_assistant_arc_clone = self.ai_assistant.clone();
                    return Command::perform(
                        async move {
                            match ai_assistant_arc_clone.read().await.get_usage_quota().await {
                                Ok(quota_info) => Message::UsageQuotaUpdated(quota_info),
                                Err(e) => Message::AgentError(format!("Failed to fetch usage quota: {}", e)),
                            }
                        },
                        |msg| msg
                    );
                }
            }
        } else {
            Command::none()
        }
    }

    /// Executes a shell command using the command manager.
    ///
    /// This function creates a new command block, adds it to the UI,
    /// and then dispatches the command for execution via a PTY session.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string to execute.
    ///
    /// # Returns
    ///
    /// An `iced::Command` to initiate command execution.
    fn execute_command(&mut self, command: String) -> Command<Message> {
        let command_block = Block::new_command(command.clone());
        let block_id = command_block.id.clone();
        self.blocks.push(command_block);
        
        let env_vars = self.config.env_profiles.active_profile
            .as_ref()
            .and_then(|name| self.config.env_profiles.profiles.get(name))
            .map(|profile| profile.variables.clone())
            .unwrap_or_default(); // Provide a default empty HashMap

        let pty_tx = self.pty_tx.clone();
        let command_manager_clone = self.command_manager.clone();

        Command::perform(
            async move {
                let parts: Vec<&str> = command.split_whitespace().collect();
                if parts.is_empty() {
                    return Message::PtyOutput(PtyMessage::Failed {
                        block_id: block_id.clone(),
                        error: "No command provided.".to_string(),
                    });
                }
                let cmd_executable = parts[0].to_string();
                let cmd_args = parts[1..].iter().map(|s| s.to_string()).collect();

                let cmd_obj = command::Command {
                    id: block_id.clone(),
                    name: cmd_executable.clone(),
                    description: format!("Executed: {}", command),
                    executable: cmd_executable,
                    args: cmd_args,
                    env: env_vars,
                    working_dir: None, // Use current working directory
                    output_format: command::CommandOutputFormat::PlainText,
                };

                match command_manager_clone.execute_command_with_output_channel(cmd_obj, pty_tx.clone()).await {
                    Ok(mut output_receiver) => {
                        while let Some(output) = output_receiver.recv().await {
                            match output.status {
                                CommandStatus::Running => {
                                    if !output.stdout.is_empty() {
                                        let _ = pty_tx.send(PtyMessage::OutputChunk {
                                            block_id: block_id.clone(),
                                            content: output.stdout,
                                            is_stdout: true,
                                        }).await;
                                    }
                                    if !output.stderr.is_empty() {
                                        let _ = pty_tx.send(PtyMessage::OutputChunk {
                                            block_id: block_id.clone(),
                                            content: output.stderr,
                                            is_stdout: false, // Corrected from is_stderr: false
                                        }).await;
                                    }
                                }
                                CommandStatus::Completed(exit_code) => {
                                    let _ = pty_tx.send(PtyMessage::Completed {
                                        block_id: block_id.clone(),
                                        exit_code,
                                    }).await;
                                    break;
                                }
                                CommandStatus::Failed(error) => {
                                    let _ = pty_tx.send(PtyMessage::Failed {
                                        block_id: block_id.clone(),
                                        error,
                                    }).await;
                                    break;
                                }
                                CommandStatus::Killed => {
                                    let _ = pty_tx.send(PtyMessage::Killed {
                                        block_id: block_id.clone(),
                                    }).await;
                                    break;
                                }
                            }
                        }
                    },
                    Err(e) => {
                        let _ = pty_tx.send(PtyMessage::Failed {
                            block_id: block_id.clone(),
                            error: format!("Failed to execute command: {}", e),
                        }).await;
                    }
                }
                Message::Tick // Dummy message to trigger UI update after command finishes
            },
            |msg| msg
        )
    }

    /// Adds initial sample blocks to the UI for demonstration purposes.
    fn add_sample_blocks(&mut self) {
        let welcome_block = Block::new_info(
            "Welcome to NeoPilot Terminal".to_string(),
            "This is a next-generation terminal with AI assistance.\nPress 'F1' to run performance benchmarks.\nUse Up/Down arrows for history, Tab for autocomplete.\nType # or /ai followed by your query to ask the AI."
        );
        
        let sample_command = Block::new_command("$ echo 'Hello, NeoPilot!'".to_string());
        let mut sample_output = Block::new_output("".to_string());
        sample_output.add_output_line("Hello, NeoPilot!".to_string(), true);
        sample_output.set_status("Completed with exit code: 0".to_string());
        
        self.blocks.push(welcome_block);
        self.blocks.push(sample_command);
        self.blocks.push(sample_output);
    }

    /// Creates a subscription for PTY manager events.
    ///
    /// This subscription listens for output and status updates from PTY sessions
    /// and converts them into `Message::PtyOutput` for the application's update loop.
    ///
    /// # Returns
    ///
    /// An `iced::Subscription` for PTY events.
    fn pty_manager_subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::unfold(
            "pty_manager_events",
            self.pty_rx.clone(),
            |mut receiver| async move {
                let msg = receiver.recv().await.expect("PTY receiver closed unexpectedly");
                (Message::PtyOutput(msg), receiver)
            },
        )
    }

    /// Creates a subscription for workflow executor events.
    ///
    /// This subscription listens for events related to workflow execution
    /// and converts them into `Message::WorkflowExecutionEvent` for the application's update loop.
    ///
    /// # Returns
    ///
    /// An `iced::Subscription` for workflow events.
    fn workflow_executor_subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::unfold(
            "workflow_executor_events",
            self.workflow_event_rx.clone(),
            |mut receiver| async move {
                let msg = receiver.recv().await.expect("Workflow event receiver closed unexpectedly");
                (Message::WorkflowExecutionEvent(msg), receiver)
            },
        )
    }
}

impl PtyMessage {
    /// Returns the block ID associated with the PTY message.
    fn get_block_id(&self) -> &str {
        match self {
            PtyMessage::OutputChunk { block_id, .. } => block_id,
            PtyMessage::Completed { block_id, .. } => block_id,
            PtyMessage::Failed { block_id, .. } => block_id,
            PtyMessage::Killed { block_id, .. } => block_id,
        }
    }
}

/// The main entry point for the NeoTerm application.
///
/// This function initializes logging, Sentry for error reporting,
/// parses command-line arguments, initializes all core modules,
/// and then runs either the Iced GUI or a headless CLI command based on arguments.
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    let _guard = sentry::init((
        "https://example.com/sentry/42", // Replace with your DSN
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    log::info!("Starting NeoTerm...");

    let cli = Cli::parse();

    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
        log::info!("Verbose logging enabled.");
    }

    // Initialize core modules
    config::init();
    asset_macro::init();
    block::init();
    cli::init();
    command::pty::init();
    fuzzy_match::init();
    graphql::init();
    input::init();
    languages::init();
    lpc::init();
    markdown_parser::init();
    string_offset::init();
    sum_tree::init();
    syntax_tree::init();
    ui::init();
    virtual_fs::init();
    watcher::init();
    websocket::init();
    workflows::init();
    workflows::debugger::init();
    workflows::executor::init();
    workflows::manager::init();
    workflows::ui::init();
    plugins::init();
    plugins::lua_engine::init();
    plugins::plugin_manager::init();
    plugins::wasm_runtime::init();
    resources::init();
    serve_wasm::init();
    settings::init();
    settings::keybinding_editor::init();
    settings::theme_editor::init();
    settings::yaml_theme_ui::init();
    settings::appearance_settings::init();
    shell::init();
    ai::init();
    agent_mode_eval::init(); // Initialize agent_mode_eval

    // Run the Iced GUI application
    match cli.command {
        Some(cli::Commands::Gui { path }) => {
            if let Some(p) = path {
                log::info!("Starting GUI with initial path: {}", p.display());
                // TODO: Pass initial path to shell manager or virtual FS
            }
            NeoTerm::run(Settings::default()).await?;
        }
        Some(cli::Commands::Run { command, args }) => {
            log::info!("Running command in headless mode: {} {:?}", command, args);
            let (command_event_tx, mut command_event_rx) = mpsc::channel(100);
            let command_manager = CommandManager::new(command_event_tx);

            let cmd_id = uuid::Uuid::new_v4().to_string();
            let cmd = command::Command {
                id: cmd_id.clone(),
                name: "cli_run".to_string(),
                description: format!("CLI run: {} {}", command, args.join(" ")),
                executable: command,
                args,
                env: std::collections::HashMap::new(),
                working_dir: None,
                output_format: command::CommandOutputFormat::PlainText,
            };
            command_manager.execute_command(cmd).await?;

            while let Some(event) = command_event_rx.recv().await {
                match event {
                    CommandEvent::Output { data, .. } => {
                        print!("{}", String::from_utf8_lossy(&data));
                    }
                    CommandEvent::Completed { id, exit_code } => {
                        if id == cmd_id {
                            log::info!("Headless command completed with exit code {:?}", exit_code);
                            break;
                        }
                    }
                    CommandEvent::Error { id, message } => {
                        if id == cmd_id {
                            log::error!("Headless command error: {}", message);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        Some(cli::Commands::Config { action }) => {
            let config_manager = Arc::new(ConfigManager::new().await?);
            match action {
                cli::ConfigCommands::Show => {
                    let prefs = config_manager.get_preferences().read().await.clone();
                    println!("Current Preferences:\n{:#?}", prefs);
                    let current_theme = config_manager.get_current_theme().await?;
                    println!("\nCurrent Theme: {}", current_theme.name);
                }
                cli::ConfigCommands::Set { key, value } => {
                    let mut prefs = config_manager.get_preferences().write().await;
                    // This is a simplified example; a real implementation would use reflection or a match statement
                    // to update specific fields based on `key`.
                    println!("Attempting to set {} = {} (Not fully implemented)", key, value);
                    // Example: if key == "terminal_font_size" { prefs.terminal_font_size = value.parse()?; }
                    config_manager.update_preferences(prefs.clone()).await?;
                    println!("Configuration updated (simulated).");
                }
                cli::ConfigCommands::Edit => {
                    println!("Opening configuration file in default editor (Not implemented).");
                    // In a real app, you'd use `edit::edit_file(Preferences::path())`
                }
            }
        }
        Some(cli::Commands::Ai { action }) => {
            // Dummy managers for CLI AI commands
            let command_manager_dummy = Arc::new(CommandManager::new(mpsc::channel(1).0));
            let virtual_file_system_dummy = Arc::new(VirtualFileSystem::new());
            let watcher_dummy = Arc::new(Watcher::new(mpsc::channel(1).0));
            let preferences = UserPreferences::load().await?;

            let ai_context_dummy = Arc::new(RwLock::new(AIContext::new(
                virtual_file_system_dummy.clone(),
                command_manager_dummy.clone(),
                watcher_dummy.clone(),
                Arc::new(ResourceManager::new()),
                Arc::new(PluginManager::new(mpsc::unbounded_channel().0)),
                Arc::new(ShellManager::new()),
                Arc::new(DriveManager::new(Default::default(), mpsc::channel(1).0)),
                Arc::new(WebSocketServer::new()),
                Arc::new(LpcEngine::new(mpsc::channel(1).0)),
                Arc::new(McqManager::new()),
                Arc::new(NaturalLanguageDetector::new()),
                Arc::new(SyntaxTreeManager::new()),
                Arc::new(StringOffsetManager::new()),
                Arc::new(SumTreeManager::new()),
                Arc::new(FuzzyMatchManager::new()),
                Arc::new(MarkdownParser::new()),
                Arc::new(LanguageManager::new()),
                Arc::new(SettingsManager::new(Arc::new(ConfigManager::new().await.unwrap()))), // Dummy ConfigManager for SettingsManager
                Arc::new(SessionSharingManager::new(mpsc::channel(1).0)),
                Arc::new(SyncManager::new(Default::default(), mpsc::channel(1).0)),
                Arc::new(WasmServer::new()),
            )));

            let assistant = Arc::new(RwLock::new(Assistant::new(
                command_manager_dummy.clone(),
                virtual_file_system_dummy.clone(),
                watcher_dummy.clone(),
                ai_context_dummy.clone(), // Pass AIContext
                &preferences.ai_provider_type,
                preferences.ai_api_key.clone(),
                preferences.ai_model.clone(),
                preferences.fallback_ai_provider_type.clone(),
                None,
                preferences.fallback_ai_model.clone(),
                preferences.redact_sensitive_info,
                preferences.local_only_ai_mode,
            )?));

            match action {
                cli::AiCommands::Chat { message } => {
                    println!("Sending message to AI: {}", message);
                    let mut assistant_lock = assistant.write().await;
                    match assistant_lock.stream_chat(&message).await {
                        Ok(mut rx) => {
                            while let Some(msg) = rx.recv().await {
                                match msg.role.as_str() {
                                    "assistant" => print!("{}", msg.content.unwrap_or_default()),
                                    "tool_calls" => println!("\nAI Tool Call: {:?}", msg.tool_calls),
                                    _ => {}
                                }
                            }
                            println!("\n");
                        }
                        Err(e) => eprintln!("Error from AI: {}", e),
                    }
                }
                cli::AiCommands::History => {
                    println!("AI Conversation History:");
                    let assistant_lock = assistant.read().await;
                    for msg in assistant_lock.get_history() {
                        println!("{}: {}", msg.role, msg.content.clone().unwrap_or_default());
                    }
                }
                cli::AiCommands::Reset => {
                    let mut assistant_lock = assistant.write().await;
                    assistant_lock.clear_history();
                    println!("AI conversation reset.");
                }
            }
        }
        Some(cli::Commands::Benchmark) => {
            println!("Running performance benchmarks...");
            let workflow_manager = Arc::new(WorkflowManager::new());
            let mut benchmarks = PerformanceBenchmarks::new(workflow_manager);
            let results = benchmarks.run_all_benchmarks().await;
            println!("\nBenchmark Results:");
            for res in results {
                println!("  {}: {:.2?} ({} iterations)", res.name, res.duration, res.iterations);
            }
        }
        Some(cli::Commands::Sync { force }) => {
            println!("Triggering cloud sync (force: {})...", force);
            let (sync_event_tx, mut sync_event_rx) = mpsc::channel(100);
            let sync_manager = SyncManager::new(Default::default(), sync_event_tx);
            tokio::spawn(async move {
                if let Err(e) = sync_manager.trigger_manual_sync(force).await {
                    eprintln!("Sync failed: {}", e);
                }
            });
            println!("Sync initiated. Check logs for progress.");
            // Wait for sync events in CLI mode
            while let Some(event) = sync_event_rx.recv().await {
                log::info!("Sync Event: {:?}", event);
                match event {
                    SyncEvent::Complete | SyncEvent::Error(_) => break,
                    _ => {}
                }
            }
        }
        Some(cli::Commands::Plugin { action }) => {
            let (plugin_event_tx, _plugin_event_rx) = mpsc::unbounded_channel();
            let plugin_manager = PluginManager::new(plugin_event_tx);
            match action {
                cli::PluginCommands::List => {
                    println!("Installed Plugins:");
                    for plugin in plugin_manager.list_plugins().await {
                        println!("- {} (Version: {})", plugin.name, plugin.version);
                    }
                }
                cli::PluginCommands::Install { source } => {
                    println!("Installing plugin from: {}", source);
                    match plugin_manager.install_plugin(&source).await {
                        Ok(_) => println!("Plugin installed successfully."),
                        Err(e) => eprintln!("Failed to install plugin: {}", e),
                    }
                }
                cli::PluginCommands::Uninstall { name } => {
                    println!("Uninstalling plugin: {}", name);
                    match plugin_manager.uninstall_plugin(&name).await {
                        Ok(_) => println!("Plugin uninstalled successfully."),
                        Err(e) => eprintln!("Failed to uninstall plugin: {}", e),
                    }
                }
                cli::PluginCommands::Update => {
                    println!("Updating all plugins (Not fully implemented).");
                }
            }
        }
        Some(cli::Commands::Workflow { action }) => {
            let mut workflow_manager = WorkflowManager::new();
            workflow_manager.init().await?; // Ensure workflows are loaded

            let command_manager_dummy = Arc::new(CommandManager::new(mpsc::channel(1).0));
            let virtual_file_system_dummy = Arc::new(VirtualFileSystem::new());
            let watcher_dummy = Arc::new(Watcher::new(mpsc::channel(1).0));
            let resource_manager_dummy = Arc::new(ResourceManager::new());
            let plugin_manager_dummy = Arc::new(PluginManager::new(mpsc::unbounded_channel().0));
            let shell_manager_dummy = Arc::new(ShellManager::new());
            let drive_manager_dummy = Arc::new(DriveManager::new(Default::default(), mpsc::channel(1).0));
            let websocket_server_dummy = Arc::new(WebSocketServer::new());
            let lpc_engine_dummy = Arc::new(LpcEngine::new(mpsc::channel(1).0));
            let mcq_manager_dummy = Arc::new(McqManager::new());
            let natural_language_detector_dummy = Arc::new(NaturalLanguageDetector::new());
            let syntax_tree_manager_dummy = Arc::new(SyntaxTreeManager::new());
            let string_offset_manager_dummy = Arc::new(StringOffsetManager::new());
            let sum_tree_manager_dummy = Arc::new(SumTreeManager::new());
            let fuzzy_match_manager_dummy = Arc::new(FuzzyMatchManager::new());
            let markdown_parser_dummy = Arc::new(MarkdownParser::new());
            let language_manager_dummy = Arc::new(LanguageManager::new());
            let config_manager_dummy = Arc::new(ConfigManager::new().await?);
            let settings_manager_dummy = Arc::new(SettingsManager::new(config_manager_dummy.clone()));
            let collaboration_manager_dummy = Arc::new(SessionSharingManager::new(mpsc::channel(1).0));
            let sync_manager_dummy = Arc::new(SyncManager::new(Default::default(), mpsc::channel(1).0));
            let wasm_server_dummy = Arc::new(WasmServer::new());

            let ai_context_dummy = Arc::new(RwLock::new(AIContext::new(
                virtual_file_system_dummy.clone(),
                command_manager_dummy.clone(),
                watcher_dummy.clone(),
                resource_manager_dummy.clone(),
                plugin_manager_dummy.clone(),
                shell_manager_dummy.clone(),
                drive_manager_dummy.clone(),
                websocket_server_dummy.clone(),
                lpc_engine_dummy.clone(),
                mcq_manager_dummy.clone(),
                natural_language_detector_dummy.clone(),
                syntax_tree_manager_dummy.clone(),
                string_offset_manager_dummy.clone(),
                sum_tree_manager_dummy.clone(),
                fuzzy_match_manager_dummy.clone(),
                markdown_parser_dummy.clone(),
                language_manager_dummy.clone(),
                settings_manager_dummy.clone(),
                collaboration_manager_dummy.clone(),
                sync_manager_dummy.clone(),
                wasm_server_dummy.clone(),
            )));

            let preferences = UserPreferences::load().await?;
            let ai_assistant_dummy = Arc::new(RwLock::new(Assistant::new(
                command_manager_dummy.clone(),
                virtual_file_system_dummy.clone(),
                watcher_dummy.clone(),
                ai_context_dummy.clone(), // Pass AIContext
                &preferences.ai_provider_type,
                preferences.ai_api_key.clone(),
                preferences.ai_model.clone(),
                preferences.fallback_ai_provider_type.clone(),
                None,
                preferences.fallback_ai_model.clone(),
                preferences.redact_sensitive_info,
                preferences.local_only_ai_mode,
            )?));
            let agent_config = {
                let mut cfg = AgentConfig::default();
                if let Some(api_key) = std::env::var("OPENAI_API_KEY").ok() {
                    cfg.api_key = Some(api_key);
                }
                cfg
            };
            let agent_mode_dummy = Arc::new(RwLock::new(AgentMode::new(agent_config, ai_assistant_dummy.clone(), ai_context_dummy.clone())?));

            let executor = WorkflowExecutor::new(
                command_manager_dummy,
                virtual_file_system_dummy,
                agent_mode_dummy,
                resource_manager_dummy,
                plugin_manager_dummy,
                shell_manager_dummy,
                drive_manager_dummy,
                watcher_dummy,
                websocket_server_dummy,
                lpc_engine_dummy,
                mcq_manager_dummy,
                natural_language_detector_dummy,
                syntax_tree_manager_dummy,
                string_offset_manager_dummy,
                sum_tree_manager_dummy,
                fuzzy_match_manager_dummy,
                markdown_parser_dummy,
                language_manager_dummy,
                settings_manager_dummy,
                collaboration_manager_dummy,
                sync_manager_dummy,
                wasm_server_dummy,
            );

            match action {
                cli::WorkflowCommands::List => {
                    println!("Available Workflows:");
                    for workflow in workflow_manager.list_workflows().await {
                        println!("- {} (Description: {})", workflow.name, workflow.description.as_deref().unwrap_or("No description"));
                    }
                }
                cli::WorkflowCommands::Run { name, args } => {
                    println!("Running workflow: {} with args: {:?}", name, args);
                    match workflow_manager.get_workflow(&name).await {
                        Ok(workflow) => {
                            let (workflow_event_tx, mut workflow_event_rx) = mpsc::channel(100);
                            let mut executor_clone = executor.clone();
                            executor_clone.set_event_sender(workflow_event_tx);

                            tokio::spawn(async move {
                                if let Err(e) = executor_clone.execute_workflow(workflow, args).await {
                                    eprintln!("Workflow execution failed: {}", e);
                                }
                            });
                            println!("Workflow '{}' started in background. Waiting for completion...", name);
                            while let Some(event) = workflow_event_rx.recv().await {
                                log::info!("Workflow Event: {:?}", event);
                                match event {
                                    WorkflowExecutionEvent::Completed { .. } | WorkflowExecutionEvent::Error { .. } => break,
                                    _ => {}
                                }
                            }
                            println!("Workflow '{}' finished.", name);
                        }
                        Err(e) => eprintln!("Workflow '{}' not found: {}", name, e),
                    }
                }
                cli::WorkflowCommands::Edit { name } => {
                    println!("Opening workflow '{}' for editing (Not implemented).", name);
                }
                cli::WorkflowCommands::Import { source } => {
                    println!("Importing workflow from: {}", source);
                    match workflow_manager.import_workflow(&source).await {
                        Ok(wf_name) => println!("Workflow '{}' imported successfully.", wf_name),
                        Err(e) => eprintln!("Failed to import workflow: {}", e),
                    }
                }
            }
        }
        None => {
            // No subcommand, run GUI by default
            NeoTerm::run(Settings::default()).await?;
        }
    }

    Ok(())
}
