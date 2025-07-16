use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use log::info;
use serde::{Serialize, Deserialize};
use crate::ai::{ChatMessage, ToolCall}; // Import from crate::ai
use crate::config::preferences::AiConfig; // Assuming AiConfig is defined here or accessible

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIStreamChunk {
    Text(String),
    ToolCall { name: String, arguments: String },
    Done,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum AIClientError {
    RequestFailed(String),
    ResponseError(String),
    StreamError(String),
}

#[async_trait::async_trait]
pub trait AIClient: Send + Sync {
    async fn stream_chat(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<AIStreamChunk>>;
    async fn get_conversation_history(&self) -> Vec<ChatMessage>;
}

pub struct OpenAIClient {
    config: AiConfig,
}

impl OpenAIClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
        }
    }
}

#[async_trait::async_trait]
impl AIClient for OpenAIClient {
    async fn stream_chat(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<AIStreamChunk>> {
        let (tx, rx) = mpsc::channel(100);

        // Mock streaming implementation
        tokio::spawn(async move {
            tx.send(AIStreamChunk::Text("OpenAI streaming response (mock) - part 1".to_string())).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(AIStreamChunk::Text("OpenAI streaming response (mock) - part 2".to_string())).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(AIStreamChunk::Text("OpenAI streaming response (mock) - part 3".to_string())).await.unwrap();
            // Mock tool call streaming
            tx.send(AIStreamChunk::ToolCall { name: "mock_tool".to_string(), arguments: r#"{"param":"value"}"#.to_string() }).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(AIStreamChunk::Done).await.unwrap();
        });

        Ok(rx)
    }

    async fn get_conversation_history(&self) -> Vec<ChatMessage> {
        // Mock implementation
        vec![
            ChatMessage { role: "user".to_string(), content: Some("Hello, AI!".to_string()), tool_calls: None, tool_call_id: None },
            ChatMessage { role: "assistant".to_string(), content: Some("Hello, User!".to_string()), tool_calls: None, tool_call_id: None },
        ]
    }
}

pub fn init() {
    info!("agent_mode_eval/ai_client module loaded");
}
