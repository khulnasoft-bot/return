use crate::ai::providers::{AIProvider, OpenAIProvider, OllamaProvider, AnthropicProvider};
use crate::ai::prompts::PromptBuilder;
use crate::ai::context::AIContext;
use crate::ai::{ChatMessage, ToolCall, ToolFunction}; // Import from parent module
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex; // Use tokio's Mutex
use tokio::sync::mpsc;
use serde_json::{json, Value};
use crate::workflows::Workflow;
use regex::Regex;
use crate::command::CommandManager;
use crate::virtual_fs::VirtualFileSystem;
use crate::watcher::Watcher;
use crate::config::preferences::AiPreferences;
use crate::plugins::plugin_manager::PluginManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::block::{Block, BlockContent};
use log::{info, error}; // Import error macro

/// Trait defining the interface for an AI tool.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
   fn name(&self) -> String;
   fn description(&self) -> String;
   async fn execute(&self, arguments: String) -> Result<String>;
}

/// Manages the registration and retrieval of AI tools.
pub struct ToolManager {
   tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
}

impl ToolManager {
   /// Creates a new `ToolManager`.
   pub fn new() -> Self {
       Self {
           tools: HashMap::new(),
       }
   }

   /// Registers a new tool with the manager.
   ///
   /// # Arguments
   ///
   /// * `tool` - The tool to register.
   pub fn register_tool(&mut self, tool: Box<dyn Tool + Send + Sync>) {
       self.tools.insert(tool.name(), tool);
   }

   /// Retrieves a tool by its name.
   ///
   /// # Arguments
   ///
   /// * `name` - The name of the tool to retrieve.
   pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
       self.tools.get(name).map(|tool| tool.as_ref())
   }

   /// Lists the names of all registered tools.
   pub fn list_tools(&self) -> Vec<String> {
       self.tools.keys().cloned().collect()
   }
}

/// The main AI assistant responsible for interacting with AI providers and managing conversation history.
pub struct Assistant {
   command_manager: Arc<CommandManager>,
   virtual_file_system: Arc<VirtualFileSystem>,
   watcher: Arc<Watcher>,
   ai_provider: Box<dyn AIProvider + Send + Sync>,
   fallback_ai_provider: Option<Box<dyn AIProvider + Send + Sync>>,
   pub conversation_history: Vec<ChatMessage>,
   redact_sensitive_info: bool,
   local_only_ai_mode: bool,
   pub tool_manager: Arc<Mutex<ToolManager>>, // Corrected to tokio::sync::Mutex
   ai_context: Arc<tokio::sync::RwLock<AIContext>>,
}

impl Assistant {
   /// Creates a new `Assistant` instance.
   ///
   /// # Arguments
   ///
   /// * `command_manager` - Shared reference to the `CommandManager`.
   /// * `virtual_file_system` - Shared reference to the `VirtualFileSystem`.
   /// * `watcher` - Shared reference to the `Watcher`.
   /// * `ai_context` - Shared reference to the `AIContext` for providing contextual information.
   /// * `ai_provider_type` - The type of the primary AI provider (e.g., "openai", "anthropic").
   /// * `ai_api_key` - The API key for the primary AI provider.
   /// * `ai_model` - The model name for the primary AI provider.
   /// * `fallback_ai_provider_type` - Optional type for a fallback AI provider.
   /// * `fallback_ai_api_key` - Optional API key for the fallback AI provider.
   /// * `fallback_ai_model` - Optional model name for the fallback AI provider.
   /// * `redact_sensitive_info` - Whether to redact sensitive information in prompts.
   /// * `local_only_ai_mode` - Whether to only use local AI providers.
   pub fn new(
       command_manager: Arc<CommandManager>,
       virtual_file_system: Arc<VirtualFileSystem>,
       watcher: Arc<Watcher>,
       ai_context: Arc<tokio::sync::RwLock<AIContext>>,
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
               let api_key = fallback_ai_api_key.or(ai_api_key.clone());
               match provider_type.as_str() {
                   "openai" => Some(Box::new(OpenAIProvider::new(api_key, fallback_ai_model.unwrap_or("gpt-3.5-turbo".to_string()))?)),
                   "anthropic" => Some(Box::new(AnthropicProvider::new(api_key, fallback_ai_model.unwrap_or("claude-3-opus-20240229".to_string()))?)),
                   "ollama" => Some(Box::new(OllamaProvider::new(api_key, fallback_ai_model.unwrap_or("llama2".to_string()))?)),
                   _ => return Err(anyhow!("Unsupported fallback AI provider: {}", provider_type)),
               }
           }
           None => None,
       };

       let mut tool_manager = ToolManager::new();
       // Register concrete tools
       tool_manager.register_tool(Box::new(crate::agent_mode_eval::tools::ListFilesTool::new(virtual_file_system.clone())));
       tool_manager.register_tool(Box::new(crate::agent_mode_eval::tools::ReadFileTool::new(virtual_file_system.clone())));
       tool_manager.register_tool(Box::new(crate::agent_mode_eval::tools::WriteFileTool::new(virtual_file_system.clone())));
       tool_manager.register_tool(Box::new(crate::agent_mode_eval::tools::ExecuteCommandTool::new(command_manager.clone())));
       tool_manager.register_tool(Box::new(crate::agent_mode_eval::tools::ChangeDirectoryTool::new(virtual_file_system.clone())));

       Ok(Self {
           command_manager,
           virtual_file_system,
           watcher,
           ai_provider,
           fallback_ai_provider,
           conversation_history: Vec::new(),
           redact_sensitive_info,
           local_only_ai_mode,
           tool_manager: Arc::new(Mutex::new(tool_manager)), // Wrap in tokio::sync::Mutex
           ai_context,
       })
   }

   /// Streams a chat conversation with the AI. This is for general chat, not the agent loop.
   ///
   /// # Arguments
   ///
   /// * `prompt` - The user's message.
   ///
   /// Returns a receiver for streaming `ChatMessage` chunks.
   pub async fn stream_chat(&mut self, prompt: &str) -> Result<mpsc::Receiver<ChatMessage>> {
       let (tx, rx) = mpsc::channel(100);

       let system_prompt = PromptBuilder::new().build_general_chat_prompt();
       let context = self.ai_context.read().await.get_full_context().await;

       let mut messages = vec![
           ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None, tool_call_id: None },
       ];
       messages.extend(self.conversation_history.iter().cloned());
       messages.push(ChatMessage { role: "user".to_string(), content: Some(format!("{}\n\nContext:\n{}", prompt, context)), tool_calls: None, tool_call_id: None });

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

   /// Sends a list of messages to the configured AI provider and returns a stream of ChatMessages.
   /// This is an internal helper for the agent loop in `AgentMode`.
   pub(crate) async fn send_message_to_provider(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<ChatMessage>> {
       let ai_provider = if self.local_only_ai_mode {
           self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
       } else {
           &self.ai_provider
       };
       // Pass tools schema to the AI provider if tool use is enabled
       let tool_manager_lock = self.tool_manager.lock().await;
       let tools_schema: Option<Value> = if !tool_manager_lock.list_tools().is_empty() {
           let mut functions = Vec::new();
           for tool_name in tool_manager_lock.list_tools() {
               if let Some(tool) = tool_manager_lock.get_tool(&tool_name) {
                   // This is a simplified schema. A real implementation would generate proper JSON schema.
                   functions.push(json!({
                       "type": "function",
                       "function": {
                           "name": tool.name(),
                           "description": tool.description(),
                           "parameters": {
                               "type": "object",
                               "properties": {}, // Placeholder, actual schema would be here
                           },
                       },
                   }));
               }
           }
           Some(Value::Array(functions))
       } else {
           None
       };

       ai_provider.stream_chat_completion(messages, tools_schema).await
   }

   /// Executes a given tool call using the registered tools.
   pub async fn execute_tool_call(&self, tool_call: crate::ai::ToolCall) -> Result<String> {
       let tool_manager_lock = self.tool_manager.lock().await;
       if let Some(tool) = tool_manager_lock.get_tool(&tool_call.function.name) {
           info!("Executing tool: {} with arguments: {}", tool_call.function.name, tool_call.function.arguments);
           tool.execute(tool_call.function.arguments.to_string()).await
       } else {
           Err(anyhow!("Tool not found: {}", tool_call.function.name))
       }
   }

   /// Generates a shell command from a natural language query.
   ///
   /// # Arguments
   ///
   /// * `natural_language_query` - The natural language query.
   pub async fn generate_command(&mut self, natural_language_query: &str) -> Result<String> {
       let system_prompt = PromptBuilder::new().build_command_generation_prompt();
       let context = self.ai_context.read().await.get_full_context().await;
       let messages = vec![
           ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None, tool_call_id: None },
           ChatMessage { role: "user".to_string(), content: Some(format!("{}\n\nContext:\n{}", natural_language_query, context)), tool_calls: None, tool_call_id: None },
       ];

       let ai_provider = if self.local_only_ai_mode {
           self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
       } else {
           &self.ai_provider
       };

       let response = ai_provider.chat(messages).await?;
       Ok(response)
   }

   /// Suggests a fix for a failed command.
   ///
   /// # Arguments
   ///
   /// * `original_command` - The command that failed.
   /// * `error_message` - The error message received.
   pub async fn fix(&mut self, original_command: &str, error_message: &str) -> Result<String> {
       let system_prompt = PromptBuilder::new().build_fix_suggestion_prompt();
       let context = self.ai_context.read().await.get_full_context().await;
       let messages = vec![
           ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None, tool_call_id: None },
           ChatMessage { role: "user".to_string(), content: Some(format!("Original command: {}\nError: {}\n\nContext:\n{}", original_command, error_message, context)), tool_calls: None, tool_call_id: None },
       ];

       let ai_provider = if self.local_only_ai_mode {
           self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
       } else {
           &self.ai_provider
       };

       let response = ai_provider.chat(messages).await?;
       Ok(response)
   }

   /// Explains the output of a command.
   ///
   /// # Arguments
   ///
   /// * `command_input` - The command that was executed.
   /// * `output_content` - The output received from the command.
   /// * `error_message` - Optional error message if the command failed.
   pub async fn explain_output(&mut self, command_input: &str, output_content: &str, error_message: Option<&str>) -> Result<String> {
       let system_prompt = PromptBuilder::new().build_explanation_prompt();
       let context = self.ai_context.read().await.get_full_context().await;
       let mut user_prompt = format!("Command: `{}`\nOutput:\n```\n{}\n```", command_input, output_content);
       if let Some(err) = error_message {
           user_prompt.push_str(&format!("\nError: {}", err));
       }
       user_prompt.push_str(&format!("\n\nContext:\n{}", context));

       let messages = vec![
           ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None, tool_call_id: None },
           ChatMessage { role: "user".to_string(), content: Some(user_prompt), tool_calls: None, tool_call_id: None },
       ];

       let ai_provider = if self.local_only_ai_mode {
           self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
       } else {
           &self.ai_provider
       };

       let response = ai_provider.chat(messages).await?;
       Ok(response)
   }

   /// Infers a workflow from a natural language query.
   ///
   /// # Arguments
   ///
   /// * `natural_language_query` - The natural language query describing the desired workflow.
   pub async fn infer_workflow(&mut self, natural_language_query: &str) -> Result<Workflow> {
       let system_prompt = PromptBuilder::new().build_workflow_inference_prompt();
       let context = self.ai_context.read().await.get_full_context().await;
       let messages = vec![
           ChatMessage { role: "system".to_string(), content: Some(system_prompt), tool_calls: None, tool_call_id: None },
           ChatMessage { role: "user".to_string(), content: Some(format!("{}\n\nContext:\n{}", natural_language_query, context)), tool_calls: None, tool_call_id: None },
       ];

       let ai_provider = if self.local_only_ai_mode {
           self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
       } else {
           &self.ai_provider
       };

       let response_json_str = ai_provider.chat(messages).await?;
       serde_json::from_str(&response_json_str)
           .map_err(|e| anyhow!("Failed to parse workflow from AI response: {}. Response: {}", e, response_json_str))
   }

   /// Clears the conversation history.
   pub fn clear_history(&mut self) {
       self.conversation_history.clear();
   }

   /// Returns a clone of the conversation history.
   pub fn get_history(&self) -> Vec<ChatMessage> {
       self.conversation_history.clone()
   }

   /// Retrieves the AI usage quota from the configured AI provider.
   pub async fn get_usage_quota(&self) -> Result<String> {
       let ai_provider = if self.local_only_ai_mode {
           self.fallback_ai_provider.as_ref().ok_or(anyhow!("Local-only mode enabled, but no local AI provider configured."))?
       } else {
           &self.ai_provider
       };
       ai_provider.get_usage_quota().await
   }
}

/// Initializes the `ai/assistant` module.
pub fn init() {
   log::info!("ai/assistant module loaded");
}
