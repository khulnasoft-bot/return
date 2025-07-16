//! This module provides the core AI functionalities, including the main Assistant,
//! AI context management, prompt building, and integration with various AI providers.

pub mod assistant;
pub mod context;
pub mod prompts;
pub mod providers;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use log::info;

/// Represents a chat message, including its role, content, and any associated tool calls.
/// This structure is designed to be compatible with common AI model APIs like OpenAI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>, // For tool_message role, linking to a specific tool_call
}

/// Represents a tool call made by the AI, matching OpenAI's structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String, // e.g., "function"
    pub function: ToolFunction,
}

/// Represents the function details within a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: Value, // Arguments are streamed as JSON chunks, so use Value
}

pub fn init() {
    info!("AI module loaded");
    assistant::init();
    context::init();
    prompts::init();
    providers::init();
}
