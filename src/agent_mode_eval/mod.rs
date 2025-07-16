//! This module provides the core logic for the AI Agent Mode, enabling
//! autonomous and interactive AI-driven operations within the terminal.

pub mod ai_client;
pub mod conversation;
pub mod tools;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use crate::ai::assistant::{Assistant, AgentMessage, Tool, ToolManager};
use crate::ai::context::AIContext;
use crate::block::Block;
use crate::workflows::Workflow;

/// Configuration for the AI Agent Mode.
#[derive(Debug, Clone, Default)]
pub struct AgentConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub enable_tools: bool,
    pub max_iterations: usize,
}

/// The main struct for managing the AI Agent Mode.
pub struct AgentMode {
    config: AgentConfig,
    assistant: Arc<tokio::sync::RwLock<Assistant>>,
    ai_context: Arc<tokio::sync::RwLock<AIContext>>,
    is_enabled: bool,
    // In-memory storage for agent prompts awaiting user response
    pending_agent_prompts: Mutex<HashMap<String, mpsc::Sender<String>>>,
}

impl AgentMode {
    /// Creates a new `AgentMode` instance.
    pub fn new(
        config: AgentConfig,
        assistant: Arc<tokio::sync::RwLock<Assistant>>,
        ai_context: Arc<tokio::sync::RwLock<AIContext>>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            assistant,
            ai_context,
            is_enabled: false,
            pending_agent_prompts: Mutex::new(HashMap::new()),
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

    /// Handles user input in agent mode, sending it to the AI assistant.
    /// Returns a vector of `AgentMessage` responses from the AI.
    pub async fn handle_user_input(&self, input: String) -> Result<mpsc::Receiver<AgentMessage>> {
        info!("AgentMode received user input: {}", input);
        let mut assistant_lock = self.assistant.write().await;
        assistant_lock.send_message(input, Vec::new()).await
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
        let prompt_id = uuid::Uuid::new_v4().to_string();
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
