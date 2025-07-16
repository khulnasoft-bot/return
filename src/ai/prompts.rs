use crate::ai::context::AIContext;
use crate::ai::providers::ChatMessage;
use std::collections::HashMap; // Import HashMap
use log::info;

pub struct PromptBuilder {
    // Add any internal state if needed for prompt building
    // Add fields for prompt templates, context variables, etc.
    templates: HashMap<String, String>,
    context_variables: HashMap<String, String>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert("system", "You are a helpful AI assistant. Respond concisely.");
        templates.insert("command_generation", "You are a command generation assistant. Generate shell commands based on user requests.");
        templates.insert("fix_suggestion", "You are a fix suggestion assistant. Suggest fixes for failed commands.");
        templates.insert("explanation", "You are an explanation assistant. Explain command outputs.");

        let context_variables = HashMap::new();

        Self {
            templates,
            context_variables,
        }
    }

    /// Builds a system prompt for general chat (mock).
    pub fn build_system_prompt(&self) -> String {
        self.templates.get("system").unwrap_or(&"Default system prompt".to_string()).to_string()
    }

    /// Builds a prompt for command generation (mock).
    pub fn build_command_generation_prompt(&self) -> String {
        self.templates.get("command_generation").unwrap_or(&"Default command generation prompt".to_string()).to_string()
    }

    /// Builds a prompt for fix suggestions (mock).
    pub fn build_fix_suggestion_prompt(&self) -> String {
        self.templates.get("fix_suggestion").unwrap_or(&"Default fix suggestion prompt".to_string()).to_string()
    }

    /// Builds a prompt for explanation (mock).
    pub fn build_explanation_prompt(&self) -> String {
        self.templates.get("explanation").unwrap_or(&"Default explanation prompt".to_string()).to_string()
    }

    pub fn build_suggestion_prompt(&self, context: &AIContext) -> ChatMessage {
        // Implementation for building suggestion prompt based on context
        ChatMessage {
            role: "assistant".to_string(),
            content: "Suggestion prompt based on context".to_string(),
        }
    }
}
