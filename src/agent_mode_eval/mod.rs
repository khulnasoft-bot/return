//! This module provides the core logic for the AI Agent Mode, enabling
//! autonomous and interactive AI-driven operations within the terminal.

pub mod ai_client;
pub mod conversation;
pub mod tools;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use log::{info, warn, error};
use serde::{Serialize, Deserialize};
use crate::ai::assistant::{Assistant, AgentMessage as ProviderAgentMessage, Tool as AiTool, ToolManager};
use crate::ai::context::AIContext;
use crate::block::Block;
use crate::workflows::Workflow;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents messages exchanged within the agent mode, including UI interactions.
#[derive(Debug, Clone)]
pub enum AgentMessage {
   UserMessage(String),
   AgentResponse(String),
   ToolCall(AiToolCall), // Use the consolidated AiToolCall
   ToolResult(String), // For tool output
   SystemMessage(String),
   Done,
   Error(String),
   WorkflowSuggested(Workflow),
   AgentPromptRequest { prompt_id: String, message: String },
   AgentPromptResponse { prompt_id: String, response: String },
}

/// Configuration for the AI Agent Mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
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

impl Default for AgentConfig {
   fn default() -> Self {
       // Default values, but these should ideally be loaded from UserPreferences
       Self {
           provider_type: "openai".to_string(),
           api_key: None,
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

/// The main struct for managing the AI Agent Mode.
pub struct AgentMode {
    config: AgentConfig,
    assistant: Arc<RwLock<Assistant>>,
    ai_context: Arc<RwLock<AIContext>>,
    is_enabled: bool,
    pending_agent_prompts: Mutex<HashMap<String, mpsc::Sender<String>>>, // Use tokio::sync::Mutex
}

impl AgentMode {
    /// Creates a new `AgentMode` instance.
    pub fn new(
        config: AgentConfig,
        assistant: Arc<RwLock<Assistant>>,
        ai_context: Arc<RwLock<AIContext>>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            assistant,
            ai_context,
            is_enabled: false,
            pending_agent_prompts: Mutex::new(HashMap::new()), // Initialize with tokio::sync::Mutex
        })
    }

    /// Toggles the agent mode on or off.
    /// Returns the new state of `is_enabled`.
    pub fn toggle(&mut self) -> bool {
        self.is_enabled = !self.is_enabled;
        info!("Agent mode toggled to: {}", self.is_enabled);
        self.is_enabled
    }

    /// Checks if agent mode is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Starts a new conversation with the AI agent.
    pub async fn start_conversation(&mut self) -> Result<()> {
        let mut assistant_lock = self.assistant.write().await;
        assistant_lock.clear_history();
        info!("Agent conversation started.");
        Ok(())
    }

    /// Sends a message to the AI and manages the agent's interaction loop,
    /// including tool execution and sending results back to the AI.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user's message.
    /// * `context_blocks` - UI blocks providing additional context to the AI.
    ///
    /// Returns a receiver for streaming `AgentMessage`s to the UI.
    pub async fn send_message(&self, prompt: String, context_blocks: Vec<Block>) -> Result<mpsc::Receiver<AgentMessage>> {
        let (tx, rx) = mpsc::channel(100);
        let sender_clone = tx.clone();
        let ai_assistant_clone = self.assistant.clone();
        let ai_context_clone = self.ai_context.clone(); // Clone AIContext for the spawned task

        tokio::spawn(async move {
            let mut ai_assistant = ai_assistant_clone.write().await;

            let system_prompt = crate::ai::prompts::PromptBuilder::new().build_general_chat_prompt();
            let context = ai_context_clone.read().await.get_full_context().await;

            let mut current_messages = Vec::new();
            current_messages.push(ProviderChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None, tool_call_id: None });

            // Add context blocks to the messages
            for block in context_blocks {
                let block_content = match block.content {
                    crate::block::BlockContent::Command { input, output, status, error, .. } => {
                        format!("Command: `{}`\nOutput:\n\`\`\`\n{}\n\`\`\`\nStatus: {}\nError: {}", input, output.iter().map(|(s, _)| s.clone()).collect::<Vec<String>>().join("\n"), status, error)
                    },
                    crate::block::BlockContent::AgentMessage { content, is_user, .. } => {
                        format!("{}: {}", if is_user { "User" } else { "Agent" }, content)
                    },
                    crate::block::BlockContent::Info { title, message, .. } => {
                        format!("Info ({}): {}", title, message)
                    },
                    crate::block::BlockContent::Error { message, .. } => {
                        format!("Error: {}", message)
                    },
                    crate::block::BlockContent::WorkflowSuggestion { workflow } => {
                        format!("Workflow Suggestion: {}\nDescription: {}\nSteps: {:#?}", workflow.name, workflow.description.as_deref().unwrap_or(""), workflow.steps)
                    },
                    crate::block::BlockContent::AgentPrompt { message, .. } => {
                        format!("Agent Prompt: {}", message)
                    },
                    crate::block::BlockContent::StreamingToolCall { id, name, arguments } => {
                        format!("Streaming Tool Call (ID: {}): {}\nArguments: {}", id, name, arguments)
                    }
                };
                current_messages.push(ProviderChatMessage { role: "system".to_string(), content: Some(block_content), tool_calls: None, tool_call_id: None });
            }

            // Add existing conversation history
            current_messages.extend(ai_assistant.conversation_history.iter().cloned());
            // Add the current user prompt
            current_messages.push(ProviderChatMessage { role: "user".to_string(), content: Some(format!("{}\n\nContext:\n{}", prompt, context)), tool_calls: None, tool_call_id: None });

            let mut iteration_count = 0;
            let max_iterations = 10; // Limit iterations to prevent infinite loops

            let mut final_assistant_response_content = String::new(); // Accumulate final text response

            loop {
                if iteration_count >= max_iterations {
                    let _ = sender_clone.send(AgentMessage::Error("Agent reached max iterations without completing.".to_string())).await;
                    break;
                }
                iteration_count += 1;

                let stream_result = ai_assistant.send_message_to_provider(current_messages.clone()).await;
                match stream_result {
                    Ok(mut stream_rx) => {
                        let mut current_turn_text_response = String::new();
                        let mut tool_calls_to_execute: Vec<AiToolCall> = Vec::new();
                        let mut stream_finished_this_turn = false;

                        while let Some(msg) = stream_rx.recv().await {
                            match msg.role.as_str() {
                                "assistant" => {
                                    if let Some(content) = msg.content {
                                        current_turn_text_response.push_str(&content);
                                        final_assistant_response_content.push_str(&content); // Accumulate for final history
                                        if sender_clone.send(AgentMessage::AgentResponse(content)).await.is_err() {
                                            warn!("Agent message receiver dropped during streaming.");
                                            stream_finished_this_turn = true;
                                            break;
                                        }
                                    }
                                },
                                "tool_calls" => {
                                    if let Some(tool_calls) = msg.tool_calls {
                                        for tool_call in tool_calls {
                                            // Send tool call to UI for display (streaming updates handled in main.rs)
                                            if sender_clone.send(AgentMessage::ToolCall(tool_call.clone())).await.is_err() {
                                                warn!("Agent message receiver dropped during tool call.");
                                                stream_finished_this_turn = true;
                                                break;
                                            }
                                            tool_calls_to_execute.push(tool_call);
                                        }
                                    }
                                },
                                _ => {
                                    if sender_clone.send(AgentMessage::SystemMessage(format!("Unknown role from AI: {}", msg.role))).await.is_err() {
                                        warn!("Agent message receiver dropped for unknown role.");
                                        stream_finished_this_turn = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if stream_finished_this_turn {
                            break; // UI receiver dropped, exit agent loop
                        }

                        // Add the AI's response (text and tool calls) from this turn to history
                        if !current_turn_text_response.is_empty() || !tool_calls_to_execute.is_empty() {
                            current_messages.push(ProviderChatMessage {
                                role: "assistant".to_string(),
                                content: if current_turn_text_response.is_empty() { None } else { Some(current_turn_text_response) },
                                tool_calls: if tool_calls_to_execute.is_empty() { None } else { Some(tool_calls_to_execute.clone()) },
                                tool_call_id: None,
                            });
                        }

                        if tool_calls_to_execute.is_empty() {
                            // No tool calls, AI is done with this turn. Exit the agent loop.
                            break;
                        } else {
                            // Execute tool calls and add results to history for the next AI turn
                            for tool_call in tool_calls_to_execute {
                                let tool_result = match ai_assistant.execute_tool_call(tool_call.clone()).await {
                                    Ok(res) => res,
                                    Err(e) => {
                                        error!("Failed to execute tool {}: {}", tool_call.function.name, e);
                                        format!("Error executing tool {}: {}", tool_call.function.name, e)
                                    }
                                };
                                // Send tool result to UI
                                if sender_clone.send(AgentMessage::ToolResult(tool_result.clone())).await.is_err() {
                                    warn!("Agent message receiver dropped during tool result.");
                                    stream_finished_this_turn = true;
                                    break;
                                }
                                // Add tool result to conversation history for the next AI turn
                                current_messages.push(ProviderChatMessage {
                                    role: "tool".to_string(),
                                    content: Some(tool_result),
                                    tool_calls: None,
                                    tool_call_id: Some(tool_call.id), // Link result to original tool call
                                });
                            }
                            if stream_finished_this_turn {
                                break;
                            }
                            // Loop again with updated history
                        }
                    },
                    Err(e) => {
                        let _ = sender_clone.send(AgentMessage::Error(format!("AI stream error: {}", e))).await;
                        break;
                    }
                }
            }
            // Update the assistant's history with the final state of this turn
            ai_assistant.conversation_history = current_messages;
            let _ = sender_clone.send(AgentMessage::Done).await;
        });

        Ok(rx)
    }

    /// Generates a shell command using the AI assistant.
    pub async fn generate_command(&self, natural_language_query: &str) -> Result<String> {
        info!("AgentMode requesting command generation for: {}", natural_language_query);
        let mut assistant_lock = self.assistant.write().await;
        assistant_lock.generate_command(natural_language_query).await
    }

    /// Requests the AI assistant to suggest a fix for a failed command.
    pub async fn fix(&self, original_command: &str, error_message: &str) -> Result<String> {
        info!("AgentMode requesting fix for command: '{}' with error: '{}'", original_command, error_message);
        let mut assistant_lock = self.assistant.write().await;
        assistant_lock.fix(original_command, error_message).await
    }

    /// Requests the AI assistant to explain command output.
    pub async fn explain_output(&self, command_input: &str, output_content: &str, error_message: Option<&str>) -> Result<String> {
        info!("AgentMode requesting explanation for command: '{}'", command_input);
        let mut assistant_lock = self.assistant.write().await;
        assistant_lock.explain_output(command_input, output_content, error_message).await
    }

    /// Retrieves the conversation history from the AI assistant.
    pub async fn get_conversation_history(&self) -> Vec<crate::ai::ChatMessage> {
        let assistant_lock = self.assistant.read().await;
        assistant_lock.get_history()
    }

    /// Handles a user's response to an agent prompt.
    pub async fn handle_agent_prompt_response(&self, prompt_id: String, response: String) -> Result<()> {
        let mut pending_prompts = self.pending_agent_prompts.lock().await;
        if let Some(tx) = pending_prompts.remove(&prompt_id) {
            info!("Received response for agent prompt {}: {}", prompt_id, response);
            tx.send(response).await.map_err(|e| anyhow!("Failed to send response to agent prompt channel: {}", e))?;
            Ok(())
        } else {
            error!("No pending prompt found for ID: {}", prompt_id);
            Err(anyhow!("No pending prompt found for ID: {}", prompt_id))
        }
    }

    /// Sends an interactive prompt to the user and waits for a response.
    /// This is called by workflow executor or other agent components.
    pub async fn send_interactive_prompt(&self, message: String) -> Result<String> {
        let prompt_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel(1); // Channel to receive user's response

        {
            let mut pending_prompts = self.pending_agent_prompts.lock().await;
            pending_prompts.insert(prompt_id.clone(), tx);
        }

        // Send a message to the main application to display the prompt UI
        // This requires a way for AgentMode to send messages back to the main Iced loop.
        // For now, we'll simulate this by assuming the main loop will pick up the prompt
        // via a BlockContent::AgentPrompt and then call handle_agent_prompt_response.
        info!("Agent requesting user input (ID: {}): {}", prompt_id, message);

        // Wait for the user's response
        let response = rx.recv().await.ok_or(anyhow!("Agent prompt channel closed unexpectedly"))?;
        Ok(response)
    }
}

/// Initializes the `agent_mode_eval` module.
pub fn init() {
    info!("agent_mode_eval module loaded");
}

// Alias ChatMessage and ToolCall from crate::ai to avoid conflicts
use crate::ai::{ChatMessage as ProviderChatMessage, ToolCall as AiToolCall};
