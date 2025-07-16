use serde::{Deserialize, Serialize};
use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use async_trait::async_trait;
use chrono::Utc;
use crate::block::{Block as UIBlock, Block, BlockContent, BlockId, BlockType}; // Alias to avoid conflict with Message
use crate::command::CommandManager;
use crate::config::ConfigManager; // Use ConfigManager to get AI preferences
use crate::agent_mode_eval::conversation::{Conversation, Message, MessageRole};
use crate::agent_mode_eval::tools::{Tool, ToolRegistry, ToolCall, ToolResult};
use crate::ai::assistant::Assistant;
use crate::ai::providers::ChatMessage as ProviderChatMessage;
use crate::ai::context::AIContext; // Import AIContext
use anyhow::{Result, anyhow};
use log::{info, warn, error};

pub mod ai_client;
pub mod conversation;
pub mod tools;

// Re-export necessary types from sub-modules
pub use conversation::Conversation;
pub use tools::{Tool, ToolCall, ToolResult}; // ToolCall here is the one used by AgentMode

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    UserMessage(String),
    AgentResponse(String),
    ToolCall(ToolCall), // Use the ToolCall from tools.rs
    ToolResult(String),
    SystemMessage(String),
    Done,
    Error(String),
    WorkflowSuggested(crate::workflows::Workflow),
    AgentPromptRequest { prompt_id: String, message: String },
    AgentPromptResponse { prompt_id: String, response: String },
}

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

pub struct AgentMode {
    config: AgentConfig,
    enabled: bool,
    pub ai_assistant: Arc<RwLock<Assistant>>,
    ai_context: Arc<AIContext>, // Added AIContext
    message_sender: mpsc::Sender<AgentMessage>,
    message_receiver: mpsc::Receiver<AgentMessage>,
    active_workflow_prompt_tx: HashMap<String, mpsc::Sender<String>>, // prompt_id -> sender for user response
}

impl AgentMode {
    pub fn new(config: AgentConfig, ai_assistant: Arc<RwLock<Assistant>>, ai_context: Arc<AIContext>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100); // Channel for agent messages to UI
        Ok(Self {
            config,
            enabled: false,
            ai_assistant,
            ai_context, // Initialize AIContext
            message_sender: tx,
            message_receiver: rx,
            active_workflow_prompt_tx: HashMap::new(),
        })
    }

    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    pub async fn start_conversation(&mut self) -> Result<()> {
        info!("Starting new agent conversation.");
        let mut assistant_lock = self.ai_assistant.write().await;
        assistant_lock.clear_history();
        // Optionally send an initial system message or greeting
        let _ = self.message_sender.send(AgentMessage::SystemMessage("Agent mode activated. How can I help you?".to_string())).await;
        Ok(())
    }

    pub async fn send_message(&mut self, prompt: String, context_blocks: Vec<Block>) -> Result<mpsc::Receiver<AgentMessage>> {
        let sender_clone = self.message_sender.clone();
        let ai_assistant_clone = self.ai_assistant.clone(); // Clone the Arc for the spawned task

        // Add user message to assistant's history
        let mut ai_assistant_write_guard = ai_assistant_clone.write().await;
        ai_assistant_write_guard.conversation_history.push(ProviderChatMessage {
            role: "user".to_string(),
            content: Some(prompt.clone()),
            tool_calls: None,
            tool_call_id: None, // This field is now part of the ChatMessage struct in ai/assistant.rs
        });
        drop(ai_assistant_write_guard); // Drop the write guard before spawning the task

        tokio::spawn(async move {
            let mut ai_assistant = ai_assistant_clone.write().await; // Get write lock inside the task

            // Check for workflow inference intent
            let lower_input = prompt.to_lowercase();
            let is_workflow_request = lower_input.contains("workflow") ||
                                      lower_input.contains("automate") ||
                                      lower_input.contains("sequence of steps") ||
                                      lower_input.contains("multi-step task");

            if is_workflow_request {
                match ai_assistant.infer_workflow(&prompt).await {
                    Ok(workflow) => {
                        info!("AI inferred workflow: {}", workflow.name);
                        let _ = sender_clone.send(AgentMessage::WorkflowSuggested(workflow)).await;
                    },
                    Err(e) => {
                        error!("Failed to infer workflow: {}", e);
                        let _ = sender_clone.send(AgentMessage::Error(format!("Failed to infer workflow: {}", e))).await;
                    }
                }
            } else {
                // Existing general chat logic
                let stream_result = ai_assistant.stream_chat(&prompt).await;
                match stream_result {
                    Ok(mut rx) => {
                        let mut full_response_content = String::new();
                        while let Some(msg) = rx.recv().await {
                            match msg.role.as_str() {
                                "assistant" => {
                                    if let Some(content) = msg.content {
                                        full_response_content.push_str(&content);
                                        if sender_clone.send(AgentMessage::AgentResponse(content)).await.is_err() {
                                            warn!("Agent message receiver dropped during streaming.");
                                            break;
                                        }
                                    }
                                },
                                "tool_calls" => {
                                    if let Some(tool_calls) = msg.tool_calls {
                                        for tool_call in tool_calls {
                                            let agent_tool_call = ToolCall {
                                                name: tool_call.name,
                                                arguments: tool_call.arguments,
                                            };
                                            if sender_clone.send(AgentMessage::ToolCall(agent_tool_call)).await.is_err() {
                                                warn!("Agent message receiver dropped during tool call.");
                                                break;
                                            }
                                            // TODO: Execute tool and send result back to assistant
                                        }
                                    }
                                },
                                _ => {} // Ignore other roles for now
                            }
                        }
                        // Add the full response to the assistant's history
                        ai_assistant.conversation_history.push(ProviderChatMessage {
                            role: "assistant".to_string(),
                            content: Some(full_response_content),
                            tool_calls: None,
                            tool_call_id: None,
                        });
                    },
                    Err(e) => {
                        let _ = sender_clone.send(AgentMessage::Error(format!("AI stream error: {}", e))).await;
                    }
                }
            }
            let _ = sender_clone.send(AgentMessage::Done).await;
        });

        Ok(self.message_receiver.clone()) // Return a clone of the receiver for the UI to subscribe
    }

    pub async fn generate_command(&mut self, natural_language_query: &str) -> Result<String> {
        let ai_assistant_clone = self.ai_assistant.clone();
        let mut ai_assistant = ai_assistant_clone.write().await;
        ai_assistant.generate_command(natural_language_query).await
    }

    pub async fn fix(&mut self, original_command: &str, error_message: &str) -> Result<String> {
        let ai_assistant_clone = self.ai_assistant.clone();
        let mut ai_assistant = ai_assistant_clone.write().await;
        ai_assistant.fix(original_command, error_message).await
    }

    pub async fn explain_output(&mut self, command_input: &str, output_content: &str, error_message: Option<&str>) -> Result<String> {
        let ai_assistant_clone = self.ai_assistant.clone();
        let mut ai_assistant = ai_assistant_clone.write().await;
        ai_assistant.explain_output(command_input, output_content, error_message).await
    }

    pub async fn handle_agent_prompt_response(&mut self, prompt_id: String, response: String) -> Result<()> {
        info!("Handling agent prompt response for prompt ID: {}", prompt_id);
        if let Some(tx) = self.active_workflow_prompt_tx.remove(&prompt_id) {
            tx.send(response).await
                .map_err(|e| anyhow!("Failed to send response to workflow executor: {}", e))?;
            Ok(())
        } else {
            Err(anyhow!("No active prompt found for ID: {}", prompt_id))
        }
    }

    /// Requests user input for an agent prompt step in a workflow.
    /// Returns a receiver that will get the user's response.
    pub async fn request_agent_prompt_input(&mut self, prompt_id: String, message: String) -> Result<mpsc::Receiver<String>> {
        let (tx, rx) = mpsc::channel(1); // Channel for this specific prompt response
        self.active_workflow_prompt_tx.insert(prompt_id.clone(), tx);
        
        // Send the prompt request to the UI
        self.message_sender.send(AgentMessage::AgentPromptRequest {
            prompt_id,
            message,
        }).await?;

        Ok(rx)
    }
}

pub fn init() {
    info!("agent_mode_eval module loaded");
}
