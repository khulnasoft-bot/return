use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use log::info;
use serde::{Serialize, Deserialize};
use crate::ai::assistant::ChatMessage;

pub struct OllamaProvider {
    api_key: Option<String>,
    model: String,
}

impl OllamaProvider {
    pub fn new(api_key: Option<String>, model: String) -> Result<Self> {
        Ok(Self {
            api_key,
            model,
        })
    }
}

#[async_trait::async_trait]
impl super::AIProvider for OllamaProvider {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        info!("Calling Ollama API (mock) with model: {}", self.model);
        // Mock implementation
        Ok("Ollama response (mock)".to_string())
    }

    async fn stream_chat(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<ChatMessage>> {
        let (tx, rx) = mpsc::channel(100);

        // Mock streaming implementation
        tokio::spawn(async move {
            tx.send(ChatMessage { role: "assistant".to_string(), content: Some("Ollama streaming response (mock) - part 1".to_string()), tool_calls: None }).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(ChatMessage { role: "assistant".to_string(), content: Some("Ollama streaming response (mock) - part 2".to_string()), tool_calls: None }).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(ChatMessage { role: "assistant".to_string(), content: Some("Ollama streaming response (mock) - part 3".to_string()), tool_calls: None }).await.unwrap();
        });

        Ok(rx)
    }

    async fn get_usage_quota(&self) -> Result<String> {
        // Mock implementation
        Ok("Ollama usage quota (mock)".to_string())
    }
}

pub fn init() {
    info!("ai/providers/ollama module loaded");
}
