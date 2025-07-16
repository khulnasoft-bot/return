use anyhow::Result;
use log::info;

/// Represents the aggregated context for the AI assistant.
/// This struct gathers relevant information from various parts of the application
/// to provide a comprehensive context for AI queries.
#[derive(Debug, Clone)] // Derive Debug for easier logging/debugging
pub struct AIContext {
    pub current_shell_state: String,
    pub active_file_content: Option<String>,
    pub selected_text: Option<String>,
    pub recent_commands: Vec<String>,
    pub project_structure_summary: String,
    pub relevant_documentation_summary: String,
    pub user_preferences_summary: String,
    pub system_info_summary: String,
    pub workflow_status_summary: String,
    pub plugin_status_summary: String,
    pub collaboration_status_summary: String,
    pub drive_status_summary: String,
    pub recent_ai_interactions_summary: String,
}

impl AIContext {
    pub fn new() -> Self {
        Self {
            current_shell_state: String::new(),
            active_file_content: None,
            selected_text: None,
            recent_commands: Vec::new(),
            project_structure_summary: String::new(),
            relevant_documentation_summary: String::new(),
            user_preferences_summary: String::new(),
            system_info_summary: String::new(),
            workflow_status_summary: String::new(),
            plugin_status_summary: String::new(),
            collaboration_status_summary: String::new(),
            drive_status_summary: String::new(),
            recent_ai_interactions_summary: String::new(),
        }
    }

    /// Gathers all available context information.
    pub async fn get_full_context(&self) -> String {
        let mut context_parts = Vec::new();

        context_parts.push(format!("Current Shell State: {}", self.get_shell_state_summary().await));
        if let Some(content) = self.get_active_file_content_summary().await {
            context_parts.push(format!("Active File Content: {}", content));
        }
        if let Some(text) = self.get_selected_text_summary().await {
            context_parts.push(format!("Selected Text: {}", text));
        }
        context_parts.push(format!("Recent Commands: {}", self.get_recent_commands_summary().await));
        context_parts.push(format!("Project Structure: {}", self.get_project_structure_summary().await));
        context_parts.push(format!("Relevant Documentation: {}", self.get_relevant_documentation_summary().await));
        context_parts.push(format!("User Preferences: {}", self.get_user_preferences_summary().await));
        context_parts.push(format!("System Info: {}", self.get_system_info_summary().await));
        context_parts.push(format!("Workflow Status: {}", self.get_workflow_status_summary().await));
        context_parts.push(format!("Plugin Status: {}", self.get_plugin_status_summary().await));
        context_parts.push(format!("Collaboration Status: {}", self.get_collaboration_status_summary().await));
        context_parts.push(format!("Drive Status: {}", self.get_drive_status_summary().await));
        context_parts.push(format!("Recent AI Interactions: {}", self.get_recent_ai_interactions_summary().await));

        context_parts.join("\n\n")
    }

    /// Placeholder for getting current shell state summary.
    pub async fn get_shell_state_summary(&self) -> String {
        // In a real application, this would query the active shell for its state (e.g., current directory, last command output).
        // For now, it returns a mock value or the stored value.
        if self.current_shell_state.is_empty() {
            "Current working directory: /home/user/project, Last command: ls -l".to_string()
        } else {
            self.current_shell_state.clone()
        }
    }

    /// Placeholder for getting active file content summary.
    pub async fn get_active_file_content_summary(&self) -> Option<String> {
        // In a real application, this would read the content of the currently active file in the editor.
        // For now, it returns a mock value or the stored value.
        self.active_file_content.clone().or_else(|| Some("fn main() { println!(\"Hello, world!\"); }".to_string()))
    }

    /// Placeholder for getting selected text summary.
    pub async fn get_selected_text_summary(&self) -> Option<String> {
        // In a real application, this would retrieve the text currently selected by the user.
        // For now, it returns a mock value or the stored value.
        self.selected_text.clone().or_else(|| Some("println!(\"Hello, world!\");".to_string()))
    }

    /// Placeholder for getting recent commands summary.
    pub async fn get_recent_commands_summary(&self) -> String {
        // In a real application, this would fetch recent commands from the shell history.
        // For now, it returns a mock value or the stored value.
        if self.recent_commands.is_empty() {
            "git status, npm install, cargo build".to_string()
        } else {
            self.recent_commands.join(", ")
        }
    }

    /// Placeholder for getting project structure summary.
    pub async fn get_project_structure_summary(&self) -> String {
        // In a real application, this would analyze the file system to provide a summary of the project structure.
        "Project root: /home/user/project, Contains: src/, tests/, Cargo.toml, README.md".to_string()
    }

    /// Placeholder for getting relevant documentation summary.
    pub async fn get_relevant_documentation_summary(&self) -> String {
        // In a real application, this would fetch relevant documentation based on the current context (e.g., language, libraries).
        "Rust documentation for `std::fs`, `tokio::fs`".to_string()
    }

    /// Placeholder for getting user preferences summary.
    pub async fn get_user_preferences_summary(&self) -> String {
        // In a real application, this would summarize key user preferences.
        "Theme: Nord, Font Size: 14, AI Assistant: Enabled".to_string()
    }

    /// Placeholder for getting system information summary.
    pub async fn get_system_info_summary(&self) -> String {
        // In a real application, this would gather system information (OS, CPU, RAM).
        "OS: Linux, CPU: Intel i7, RAM: 16GB".to_string()
    }

    /// Placeholder for getting workflow status summary.
    pub async fn get_workflow_status_summary(&self) -> String {
        // In a real application, this would report on active or recently run workflows.
        "Active workflows: None, Last completed: 'build_and_test'".to_string()
    }

    /// Placeholder for getting plugin status summary.
    pub async fn get_plugin_status_summary(&self) -> String {
        // In a real application, this would list enabled plugins and their status.
        "Enabled plugins: Git Integration, LSP Client".to_string()
    }

    /// Placeholder for getting collaboration status summary.
    pub async fn get_collaboration_status_summary(&self) -> String {
        // In a real application, this would report on active collaboration sessions.
        "Collaboration session: Inactive".to_string()
    }

    /// Placeholder for getting drive status summary.
    pub async fn get_drive_status_summary(&self) -> String {
        // In a real application, this would report on cloud drive sync status.
        "Cloud drive sync: Last synced 5 minutes ago".to_string()
    }

    /// Placeholder for getting recent AI interactions summary.
    pub async fn get_recent_ai_interactions_summary(&self) -> String {
        // In a real application, this would summarize recent AI queries and responses.
        "Last query: 'How to implement a trait?', Last response: 'Use `impl Trait for Type` syntax.'".to_string()
    }
}

pub fn init() {
    info!("ai/context module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_context_full_context() {
        let context = AIContext::new();
        let full_context = context.get_full_context().await;

        assert!(full_context.contains("Current Shell State: Current working directory: /home/user/project, Last command: ls -l"));
        assert!(full_context.contains("Active File Content: fn main() { println!(\"Hello, world!\"); }"));
        assert!(full_context.contains("Selected Text: println!(\"Hello, world!\");"));
        assert!(full_context.contains("Recent Commands: git status, npm install, cargo build"));
        assert!(full_context.contains("Project Structure: Project root: /home/user/project, Contains: src/, tests/, Cargo.toml, README.md"));
        assert!(full_context.contains("Relevant Documentation: Rust documentation for `std::fs`, `tokio::fs`"));
        assert!(full_context.contains("User Preferences: Theme: Nord, Font Size: 14, AI Assistant: Enabled"));
        assert!(full_context.contains("System Info: OS: Linux, CPU: Intel i7, RAM: 16GB"));
        assert!(full_context.contains("Workflow Status: Active workflows: None, Last completed: 'build_and_test'"));
        assert!(full_context.contains("Plugin Status: Enabled plugins: Git Integration, LSP Client"));
        assert!(full_context.contains("Collaboration Status: Collaboration session: Inactive"));
        assert!(full_context.contains("Drive Status: Cloud drive sync: Last synced 5 minutes ago"));
        assert!(full_context.contains("Recent AI Interactions: Last query: 'How to implement a trait?', Last response: 'Use `impl Trait for Type` syntax.'"));
    }

    #[tokio::test]
    async fn test_ai_context_individual_summaries() {
        let mut context = AIContext::new();

        // Test with default values
        assert_eq!(context.get_shell_state_summary().await, "Current working directory: /home/user/project, Last command: ls -l");
        assert_eq!(context.get_active_file_content_summary().await, Some("fn main() { println!(\"Hello, world!\"); }".to_string()));
        assert_eq!(context.get_selected_text_summary().await, Some("println!(\"Hello, world!\");".to_string()));
        assert_eq!(context.get_recent_commands_summary().await, "git status, npm install, cargo build");

        // Test with custom values
        context.current_shell_state = "Current working directory: /app, Last command: docker ps".to_string();
        context.active_file_content = Some("console.log('Hello');".to_string());
        context.selected_text = Some("console.log".to_string());
        context.recent_commands = vec!["npm start".to_string(), "docker build".to_string()];

        assert_eq!(context.get_shell_state_summary().await, "Current working directory: /app, Last command: docker ps");
        assert_eq!(context.get_active_file_content_summary().await, Some("console.log('Hello');".to_string()));
        assert_eq!(context.get_selected_text_summary().await, Some("console.log".to_string()));
        assert_eq!(context.get_recent_commands_summary().await, "npm start, docker build");
    }
}
