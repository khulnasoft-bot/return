use crate::ai::context::AIContext;
use crate::ai::providers::ChatMessage;
use std::collections::HashMap; // Import HashMap
use log::info;

pub struct PromptBuilder {
    // Add any internal state if needed for prompt building
    // Add fields for prompt templates, context variables, etc.
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {}
    }

    /// Builds a system prompt for general chat (mock).
    pub fn build_system_prompt(&self) -> String {
        "You are a helpful AI assistant. Respond concisely.".to_string()
    }

    /// Builds a prompt for command generation (mock).
    pub fn build_command_generation_prompt(&self) -> String {
        "You are a command generation assistant. Generate shell commands based on user requests.".to_string()
    }

    /// Builds a prompt for fix suggestions (mock).
    pub fn build_fix_suggestion_prompt(&self) -> String {
        "You are a fix suggestion assistant. Suggest fixes for failed commands.".to_string()
    }

    /// Builds a prompt for explanation (mock).
    pub fn build_explanation_prompt(&self) -> String {
        "You are an explanation assistant. Explain command outputs.".to_string()
    }

    pub fn build_suggestion_prompt(&self, context: &AIContext) -> ChatMessage {
