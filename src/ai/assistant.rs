use crate::ai::providers::{AIProvider, ChatMessage, OpenAIProvider, OllamaProvider, AnthropicProvider};
use crate::ai::prompts::PromptBuilder;
use crate::ai::context::AIContext;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde_json::Value;
use crate::workflows::Workflow; // Import Workflow struct
use regex::Regex; // For redaction
use crate::command::CommandManager;
use crate::virtual_fs::VirtualFileSystem;
use crate::watcher::Watcher;
use crate::config::preferences::AiPreferences;
use crate::plugins::plugin_manager::PluginManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: String,
}

pub enum AgentMessage {
    AgentResponse(String),
    ToolCall(ToolCall),
    SystemMessage(String),
    Done,
}

pub struct Assistant {
    command_manager: Arc<CommandManager>,
    virtual_file_system: Arc<VirtualFileSystem>,
    watcher: Arc<Watcher>,
    ai_provider: Box<dyn AIProvider + Send + Sync>,
    fallback_ai_provider: Option<Box<dyn AIProvider + Send + Sync>>,
    history: Vec<ChatMessage>,
    redact_sensitive_info: bool,
    local_only_ai_mode: bool,
    tool_manager: Arc<Mutex<ToolManager>>,
}

impl Assistant {
    pub fn new(
        command_manager: Arc<CommandManager>,
        virtual_file_system: Arc<VirtualFileSystem>,
        watcher: Arc<Watcher>,
        ai_provider_type: &str,
        ai_api_key: Option<String>,
        ai_model: String,
        fallback_ai_provider_type: Option<String>,
        fallback_ai_api_key: Option<String>,
        fallback_ai_model: Option<String>,
        redact_sensitive_info: bool,
        local_only_ai_mode: bool,
    ) -> Result<Self> {
        let ai_provider: Box<dyn AIProvider + Send + Sync> = match ai_provider_type {
            "openai" => Box::new(OpenAIProvider::new(ai_api_key, ai_model)?),
            "anthropic" => Box::new(AnthropicProvider::new(ai_api_key, ai_model)?),
            "ollama" => Box::new(OllamaProvider::new(ai_api_key, ai_model)?),
            _ => return Err(anyhow!("Unsupported AI provider: {}", ai_provider_type)),
        };

        let fallback_ai_provider: Option<Box<dyn AIProvider + Send + Sync>> = match fallback_ai_provider_type {
            Some(provider_type) => {
                let api_key = fallback_ai_api_key.or(ai_api_key.clone()); // Fallback to primary if not provided
                match provider_type.as_str() {
                    "openai" => Some(Box::new(OpenAIProvider::new(api_key, fallback_ai_model.unwrap_or("gpt-3.5-turbo".to_string()))?)),
                    "anthropic" => Some(Box::new(AnthropicProvider::new(api_key, fallback_ai_model.unwrap_or("claude-3-opus-20240229".to_string()))?)),
                    "ollama" => Some(Box::new(OllamaProvider::new(api_key, fallback_ai_model.unwrap_or("llama2".to_string()))?)),
                    _ => return Err(anyhow!("Unsupported fallback AI provider: {}", provider_type)),
                }
            }
            None => None,
        };

        Ok(Self {
            command_manager,
            virtual_file_system,
            watcher,
            ai_provider,
            fallback_ai_provider,
            history: Vec::new(),
            redact_sensitive_info,
            local_only_ai_mode,
            tool_manager: Arc::new(Mutex::new(ToolManager::new())),
        })
    }

    pub async fn stream_chat(&mut self, prompt: &str) -> Result<mpsc::Receiver<ChatMessage>> {
        let (tx, rx) = mpsc::channel(100);

        let system_prompt = PromptBuilder::new().build_general_chat_prompt();
        let context = AIContext::new(
            self.command_manager.clone(),
            self.virtual_file_system.clone(),
            self.watcher.clone(),
            self.redact_sensitive_info,
        ).get_context().await?;

        let messages = vec![
            ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None },
            ChatMessage { role: "user".to_string(), content: Some(format!("{}\n{}", prompt, context)), tool_calls: None },
        ];

        let mut stream = self.ai_provider.stream_chat(messages).await?;

        tokio::spawn(async move {
            while let Some(chunk) = stream.recv().await {
                if tx.send(chunk).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }

    pub async fn send_message(&mut self, prompt: String, context_blocks: Vec<Block>) -> Result<mpsc::Receiver<AgentMessage>> {
        let (tx, rx) = mpsc::channel(100);

        let system_prompt = PromptBuilder::new().build_general_chat_prompt();
        let context = AIContext::new(
            self.command_manager.clone(),
            self.virtual_file_system.clone(),
            self.watcher.clone(),
            self.redact_sensitive_info,
        ).get_context().await?;

        let mut messages = vec![
            ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None },
        ];

        // Add context blocks to the messages
        for block in context_blocks {
            let block_content = match block.content {
                BlockContent::Command { input, output, status, error, .. } => {
                    format!("Command: `{}`\nOutput:\n\`\`\`\n{}\n\`\`\`\nStatus: {}\nError: {}", input, output.iter().map(|(s, _)| s.clone()).collect::<Vec<String>>().join("\n"), status, error)
                },
                BlockContent::AgentMessage { content, is_user, .. } => {
                    format!("{}: {}", if is_user { "User" } else { "Agent" }, content)
                },
                BlockContent::Info { title, message, .. } => {
                    format!("Info ({}): {}", title, message)
                },
                BlockContent::Error { message, .. } => {
                    format!("Error: {}", message)
                },
                BlockContent::WorkflowSuggestion { workflow } => {
                    format!("Workflow Suggestion: {}\nDescription: {}\nSteps: {:#?}", workflow.name, workflow.description.as_deref().unwrap_or(""), workflow.steps)
                },
                BlockContent::AgentPrompt { message, .. } => {
                    format!("Agent Prompt: {}", message)
                },
            };
            messages.push(ChatMessage { role: "system".to_string(), content: Some(block_content), tool_calls: None });
        }

        messages.push(ChatMessage { role: "user".to_string(), content: Some(format!("{}\n{}", prompt, context)), tool_calls: None });

        let ai_provider = if self.local_only_ai_mode {
            self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
        } else {
            &self.ai_provider
        };

        let mut stream = ai_provider.stream_chat(messages).await?;

        tokio::spawn(async move {
            while let Some(chunk) = stream.recv().await {
                // Convert generic ChatMessage to AgentMessage
                let agent_message = match chunk.role.as_str() {
                    "assistant" => AgentMessage::AgentResponse(chunk.content.unwrap_or_default()),
                    "tool_calls" => {
                        if let Some(tool_calls) = chunk.tool_calls {
                            for tool_call in tool_calls {
                                AgentMessage::ToolCall(ToolCall { name: tool_call.name, arguments: tool_call.arguments })
                            }
                            continue;
                        } else {
                            AgentMessage::Error("Tool call with no arguments".to_string())
                        }
                    }
                    _ => AgentMessage::SystemMessage(format!("Unknown role: {}", chunk.role)),
                };

                if tx.send(agent_message).await.is_err() {
                    break;
                }
            }
            // Signal completion
            let _ = tx.send(AgentMessage::Done).await;
        });

        Ok(rx)
    }

    pub async fn generate_command(&mut self, natural_language_query: &str) -> Result<String> {
        let system_prompt = PromptBuilder::new().build_command_generation_prompt();
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None },
            ChatMessage { role: "user".to_string(), content: Some(natural_language_query.to_string()), tool_calls: None },
        ];

        let ai_provider = if self.local_only_ai_mode {
            self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
        } else {
            &self.ai_provider
        };

        let response = ai_provider.chat(messages).await?;
        Ok(response)
    }

    pub async fn fix(&mut self, original_command: &str, error_message: &str) -> Result<String> {
        let system_prompt = PromptBuilder::new().build_fix_suggestion_prompt();
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None },
            ChatMessage { role: "user".to_string(), content: Some(format!("Original command: {}\nError: {}", original_command, error_message)), tool_calls: None },
        ];

        let ai_provider = if self.local_only_ai_mode {
            self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
        } else {
            &self.ai_provider
        };

        let response = ai_provider.chat(messages).await?;
        Ok(response)
    }

    pub async fn explain_output(&mut self, command_input: &str, output_content: &str, error_message: Option<&str>) -> Result<String> {
        let system_prompt = PromptBuilder::new().build_explanation_prompt();
        let mut user_prompt = format!("Command: `{}`\nOutput:\n\`\`\`\n{}\n\`\`\`", command_input, output_content);
        if let Some(err) = error_message {
            user_prompt.push_str(&format!("\nError: {}", err));
        }

        let messages = vec![
            ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None },
            ChatMessage { role: "user".to_string(), content: Some(user_prompt), tool_calls: None },
        ];

        let ai_provider = if self.local_only_ai_mode {
            self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
        } else {
            &self.ai_provider
        };

        let response = ai_provider.chat(messages).await?;
        Ok(response)
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn get_history(&self) -> Vec<ChatMessage> {
        self.history.clone()
    }

    pub async fn get_usage_quota(&self) -> Result<String> {
        let ai_provider = if self.local_only_ai_mode {
            self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
        } else {
            &self.ai_provider
        };
        ai_provider.get_usage_quota().await
    }
}

pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn execute(&self, arguments: String) -> Result<String>;
}

pub struct ToolManager {
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|tool| tool.as_ref())
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

pub fn init() {
    log::info!("ai/assistant module loaded");
}
