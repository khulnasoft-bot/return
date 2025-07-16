use serde::{Deserialize, Serialize};
use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use async_trait::async_trait;
use chrono::Utc;
use crate::agent_mode_eval::ai_client::{AIClient, OpenAIClient, AIClientError, AIStreamChunk};
use crate::block::{Block as UIBlock, Block, BlockContent, BlockId, BlockType}; // Alias to avoid conflict with Message
use crate::command::CommandExecutor;
use crate::config::Config;
use crate::agent_mode_eval::conversation::{Conversation, Message, MessageRole};
use crate::agent_mode_eval::tools::{Tool, ToolRegistry, ToolCall, ToolResult};
use ai_client::{AIClient, ChatMessage, AiConfig, OpenAIClient};
use conversation::Conversation;
use tools::ToolManager;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::workflows::Workflow; // Import Workflow
use log::{info, warn, error};

pub mod ai_client;
pub mod conversation;
pub mod tools;

use crate::ai::assistant::Assistant;
use crate::ai::providers::ChatMessage as ProviderChatMessage;
use crate::block::{Block, BlockContent};
use crate::agent_mode_eval::tools::{ToolCall as AgentToolCall, ToolResult};

pub use conversation::{Conversation, Message, MessageRole};
pub use tools::{Tool, ToolCall, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    UserMessage(String),
    AgentResponse(String),
    ToolCall(ToolCall),
    ToolResult(String),
    SystemMessage(String),
    Done,
    Error(String),
    WorkflowSuggested(crate::workflows::Workflow),
    AgentPromptRequest { prompt_id: String, message: String },
    AgentPromptResponse { prompt_id: String, response: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub api_key: Option<String>,
    // Add other agent-specific configuration options here
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            api_key: None,
        }
    }
}

pub struct AgentMode {
    config: AgentConfig,
    enabled: bool,
    ai_assistant: Arc<RwLock<Assistant>>,
    message_sender: mpsc::Sender<AgentMessage>,
    message_receiver: mpsc::Receiver<AgentMessage>,
    active_workflow_prompt_tx: HashMap<String, mpsc::Sender<String>>, // prompt_id -> sender for user response
}

impl AgentMode {
    pub fn new(config: AgentConfig, ai_assistant: Arc<RwLock<Assistant>>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100); // Channel for agent messages to UI
        Ok(Self {
            config,
            enabled: false,
            ai_assistant,
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
        // Initialize conversation state, load initial context, etc.
        Ok(())
    }

    pub async fn send_message(&mut self, prompt: String, context_blocks: Vec<Block>) -> Result<mpsc::Receiver<AgentMessage>> {
        let sender_clone = self.message_sender.clone();
        let ai_assistant_clone = self.ai_assistant.clone(); // Clone the Arc for the spawned task

        // Add user message to history
        let mut ai_assistant = ai_assistant_clone.write().await;
        ai_assistant.conversation_history.push(crate::ai::providers::ChatMessage {
            role: "user".to_string(),
            content: Some(prompt.clone()),
            tool_calls: None,
            tool_call_id: None,
        });

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
                                                name: tool_call.function.name,
                                                arguments: tool_call.function.arguments,
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
                        ai_assistant.conversation_history.push(crate::ai::providers::ChatMessage {
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
