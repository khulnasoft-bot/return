use async_trait::async_trait;
use tokio::sync::mpsc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ai::assistant::ChatMessage;

pub mod openai;
pub mod ollama;
pub mod anthropic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String, // "function"
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: Value, // JSON object as a string
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;
    async fn stream_chat(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<ChatMessage>>;
    async fn get_usage_quota(&self) -> Result<String>;
}

pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;

pub fn init() {
    println!("ai/providers module loaded");
}
