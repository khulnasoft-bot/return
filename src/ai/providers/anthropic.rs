use super::{AIProvider, ChatMessage};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use futures_util::StreamExt;
use std::collections::HashMap;
use log::info;

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    api_key: Option<String>,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: Option<String>, model: String) -> Result<Self> {
        Ok(Self {
            api_key,
            model,
        })
    }
}

#[async_trait]
impl super::AIProvider for AnthropicProvider {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        info!("Calling Anthropic API (mock) with model: {}", self.model);
        // Mock implementation
        Ok("Anthropic response (mock)".to_string())
    }

    async fn stream_chat(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<ChatMessage>> {
        let (tx, rx) = mpsc::channel(100);

        // Mock streaming implementation
        tokio::spawn(async move {
            tx.send(ChatMessage { role: "assistant".to_string(), content: Some("Anthropic streaming response (mock) - part 1".to_string()), tool_calls: None }).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(ChatMessage { role: "assistant".to_string(), content: Some("Anthropic streaming response (mock) - part 2".to_string()), tool_calls: None }).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            tx.send(ChatMessage { role: "assistant".to_string(), content: Some("Anthropic streaming response (mock) - part 3".to_string()), tool_calls: None }).await.unwrap();
        });

        Ok(rx)
    }

    async fn get_usage_quota(&self) -> Result<String> {
        // Mock implementation
        Ok("Anthropic usage quota (mock)".to_string())
    }
}

pub fn init() {
    info!("ai/providers/anthropic module loaded");
}
