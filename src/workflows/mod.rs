//! This module defines the core data structures and traits for workflows in NeoTerm.
//! Workflows are sequences of steps that can automate tasks, interact with the AI,
//! execute commands, and integrate with plugins.
//!
//! It includes definitions for `Workflow`, `WorkflowStep`, `WorkflowArgument`,
//! and related enums, along with parsing, validation, and search capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid; // Import Uuid for generating unique IDs for workflows and steps

pub mod parser;
pub mod manager;
pub mod executor;
pub mod ui;
pub mod debugger;

pub use parser::*;
pub use manager::*;
pub use executor::*;
pub use ui::*;
pub use debugger::*;

/// Represents a complete workflow, defining a sequence of automated steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// A unique identifier for the workflow.
    pub id: String,
    /// The name of the Workflow. Required.
    pub name: String,
    /// The description of the Workflow and what it does. Optional.
    pub description: Option<String>,
    /// An array of tags that are useful to categorize the Workflow. Optional.
    #[serde(default)]
    pub tags: Vec<String>,
    /// The URL from where the Workflow was originally generated from. Optional.
    pub source_url: Option<String>,
    /// The original author of the Workflow. Optional.
    pub author: Option<String>,
    /// The URL of original author of the Workflow. Optional.
    pub author_url: Option<String>,
    /// The list of shells where this Workflow is valid. Optional.
    /// Must be one of: zsh, bash, fish
    pub shells: Option<Vec<Shell>>,
    /// Parameterized arguments for the workflow. Optional.
    #[serde(default)]
    pub arguments: Vec<WorkflowArgument>,
    /// The sequence of steps to be executed in this workflow.
    pub steps: Vec<WorkflowStep>,
    /// Environment variables to be set for the entire workflow.
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Timeout for the entire workflow in seconds.
    pub timeout: Option<u64>,

    // Internal metadata
    #[serde(skip)]
    pub file_path: Option<PathBuf>,
    #[serde(skip)]
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip)]
    pub usage_count: u32,
}

/// Represents a single step within a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// A unique identifier for the step within the workflow.
    pub id: String,
    /// The name of the step.
    pub name: String,
    /// The type of action this step performs.
    #[serde(rename = "type")]
    pub step_type: WorkflowStepType,
    /// Optional description for the step.
    pub description: Option<String>,
    /// Environment variables specific to this step.
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Timeout for this specific step in seconds.
    pub timeout: Option<u64>,
    /// Number of times to retry this step on failure.
    #[serde(default)]
    pub retry_count: u32,
    /// A conditional expression that must evaluate to true for the step to execute.
    pub condition: Option<String>,
    /// How the output of this step should be processed.
    #[serde(default)]
    pub output_format: WorkflowOutputFormat,
    /// The name of the variable to store the output of this step.
    pub output_variable: Option<String>,
}

/// Defines the type of action a workflow step performs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStepType {
    /// Executes a shell command.
    Command {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        working_directory: Option<String>,
    },
    /// Prompts the user for input during workflow execution.
    AgentPrompt {
        message: String,
        #[serde(default)]
        input_variable: Option<String>, // Variable to store user's response
    },
    /// Calls an AI tool (e.g., list_files, read_file, execute_command).
    ToolCall {
        tool_name: String,
        arguments: serde_json::Value, // JSON object for tool arguments
    },
    /// Executes a sub-workflow.
    SubWorkflow {
        workflow_name: String,
        #[serde(default)]
        args: Vec<String>,
    },
    /// Runs a plugin function.
    PluginAction {
        plugin_name: String,
        action_name: String,
        #[serde(default)]
        arguments: serde_json::Value,
    },
}

/// Defines how the output of a workflow step should be processed.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowOutputFormat {
    #[default]
    PlainText,
    Json,
    Regex { pattern: String },
}

/// Supported shell types for workflow compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

/// Defines a parameterized argument for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowArgument {
    /// The name of the argument. Required.
    pub name: String,
    /// The description of the argument. Optional.
    pub description: Option<String>,
    /// The default value for the argument. Optional.
    pub default_value: Option<String>,
    /// The type of argument for validation. Optional.
    #[serde(default)]
    pub arg_type: ArgumentType,
    /// Whether this argument is required. Optional.
    #[serde(default)]
    pub required: bool,
    /// Possible values for this argument (for enum-like arguments). Optional.
    pub options: Option<Vec<String>>,
}

/// Defines the data type of a workflow argument for validation.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    #[default]
    String,
    Number,
    Boolean,
    Path,
    Url,
    Email,
    Enum,
}

/// Represents an active execution of a workflow.
#[derive(Debug, Clone)]
pub struct WorkflowExecution {
    pub workflow: Workflow,
    pub arguments: HashMap<String, String>,
    pub resolved_command: String,
    pub shell: Shell,
}

/// Represents the result of a workflow search.
#[derive(Debug, Clone)]
pub struct WorkflowSearchResult {
    pub workflow: Workflow,
    pub score: f32,
    pub matched_fields: Vec<String>,
}

/// Custom error types for workflow operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Error during parsing of workflow definition (e.g., invalid YAML).
    #[error("Parse error: {0}")]
    ParseError(String),
    /// Error during validation of workflow structure or content.
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// Error during file I/O operations (e.g., reading/writing workflow files).
    #[error("IO error: {0}")]
    IoError(String),
    /// Error related to workflow arguments (e.g., invalid format, missing).
    #[error("Argument error: {0}")]
    ArgumentError(String),
    /// The specified shell is not supported by the workflow or the system.
    #[error("Shell not supported: {0:?}")]
    UnsupportedShell(Shell),
    /// A required workflow argument was not provided.
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    /// An argument was provided with an invalid value or type.
    #[error("Invalid argument value: {0}")]
    InvalidArgumentValue(String),
    /// The requested workflow could not be found.
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    /// General error during workflow step execution.
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

impl Workflow {
    /// Parses a workflow from a YAML string.
    ///
    /// Ensures all steps have unique IDs and validates the workflow structure.
    ///
    /// # Arguments
    ///
    /// * `yaml_str` - The YAML string representing the workflow.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Workflow` or a `WorkflowError`.
    pub fn from_yaml(yaml_str: &str) -> Result<Self, WorkflowError> {
        let mut workflow: Workflow = serde_yaml::from_str(yaml_str)
            .map_err(|e| WorkflowError::ParseError(e.to_string()))?;
        
        // Ensure all steps have unique IDs if not provided
        for step in &mut workflow.steps {
            if step.id.is_empty() {
                step.id = Uuid::new_v4().to_string();
            }
        }

        workflow.validate()?;
        Ok(workflow)
    }

    /// Converts the workflow to a YAML string.
    ///
    /// # Returns
    ///
    /// A `Result` containing the YAML string or a `WorkflowError`.
    pub fn to_yaml(&self) -> Result<String, WorkflowError> {
        serde_yaml::to_string(self)
            .map_err(|e| WorkflowError::ParseError(e.to_string()))
    }

    /// Loads a workflow from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the workflow YAML file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Workflow` or a `WorkflowError`.
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, WorkflowError> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| WorkflowError::IoError(e.to_string()))?;
        
        let mut workflow = Self::from_yaml(&content)?;
        workflow.file_path = Some(path.as_ref().to_path_buf());
        Ok(workflow)
    }

    /// Saves the workflow to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the workflow YAML file should be saved.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `WorkflowError`.
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), WorkflowError> {
        let yaml_str = self.to_yaml()?;
        std::fs::write(&path, yaml_str)
            .map_err(|e| WorkflowError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Validates the workflow structure and content.
    ///
    /// Checks for required fields, valid shell types, argument consistency,
    /// and step-specific validations.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `WorkflowError` if validation fails.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        if self.name.trim().is_empty() {
            return Err(WorkflowError::ValidationError("Name is required".to_string()));
        }

        if self.id.trim().is_empty() {
            return Err(WorkflowError::ValidationError("ID is required".to_string()));
        }

        // Validate shell compatibility
        if let Some(shells) = &self.shells {
            if shells.is_empty() {
                return Err(WorkflowError::ValidationError("Shells array cannot be empty".to_string()));
            }
        }

        // Validate arguments
        for arg in &self.arguments {
            if arg.name.trim().is_empty() {
                return Err(WorkflowError::ValidationError("Argument name is required".to_string()));
            }

            // Validate enum options
            if arg.arg_type == ArgumentType::Enum && arg.options.is_none() {
                return Err(WorkflowError::ValidationError(
                    format!("Enum argument '{}' must have options", arg.name)
                ));
            }
        }

        // Validate steps
        for step in &self.steps {
            if step.id.trim().is_empty() {
                return Err(WorkflowError::ValidationError(format!("Step '{}' is missing an ID", step.name)));
            }
            if step.name.trim().is_empty() {
                return Err(WorkflowError::ValidationError(format!("Step with ID '{}' is missing a name", step.id)));
            }

            match &step.step_type {
                WorkflowStepType::Command { command, .. } => {
                    if command.trim().is_empty() {
                        return Err(WorkflowError::ValidationError(format!("Command step '{}' has an empty command", step.name)));
                    }
                },
                WorkflowStepType::AgentPrompt { message, .. } => {
                    if message.trim().is_empty() {
                        return Err(WorkflowError::ValidationError(format!("AgentPrompt step '{}' has an empty message", step.name)));
                    }
                },
                WorkflowStepType::ToolCall { tool_name, arguments } => {
                    if tool_name.trim().is_empty() {
                        return Err(WorkflowError::ValidationError(format!("ToolCall step '{}' has an empty tool_name", step.name)));
                    }
                    if !arguments.is_object() {
                        return Err(WorkflowError::ValidationError(format!("ToolCall step '{}' arguments must be a JSON object", step.name)));
                    }
                },
                WorkflowStepType::SubWorkflow { workflow_name, .. } => {
                    if workflow_name.trim().is_empty() {
                        return Err(WorkflowError::ValidationError(format!("SubWorkflow step '{}' has an empty workflow_name", step.name)));
                    }
                },
                WorkflowStepType::PluginAction { plugin_name, action_name, arguments } => {
                    if plugin_name.trim().is_empty() {
                        return Err(WorkflowError::ValidationError(format!("PluginAction step '{}' has an empty plugin_name", step.name)));
                    }
                    if action_name.trim().is_empty() {
                        return Err(WorkflowError::ValidationError(format!("PluginAction step '{}' has an empty action_name", step.name)));
                    }
                    if !arguments.is_object() {
                        return Err(WorkflowError::ValidationError(format!("PluginAction step '{}' arguments must be a JSON object", step.name)));
                    }
                },
            }
        }

        Ok(())
    }

    /// Extracts all placeholders (e.g., `{{arg_name}}`) from workflow arguments' default values
    /// and command steps.
    ///
    /// # Returns
    ///
    /// A `Vec<String>` containing the names of all found placeholders.
    pub fn extract_placeholders(&self) -> Vec<String> {
        let mut placeholders = Vec::new();
        
        // Check workflow arguments for placeholders in default values
        for arg in &self.arguments {
            if let Some(default_value) = &arg.default_value {
                let mut chars = default_value.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == '{' && chars.peek() == Some(&'{') {
                        chars.next(); // consume second '{'
                        let mut placeholder = String::new();
                        while let Some(ch) = chars.next() {
                            if ch == '}' && chars.peek() == Some(&'}') {
                                chars.next(); // consume second '}'
                                if !placeholder.is_empty() {
                                    placeholders.push(placeholder);
                                }
                                break;
                            } else {
                                placeholder.push(ch);
                            }
                        }
                    }
                }
            }
        }

        // Check command steps for placeholders
        for step in &self.steps {
            if let WorkflowStepType::Command { command, .. } = &step.step_type {
                let mut chars = command.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == '{' && chars.peek() == Some(&'{') {
                        chars.next(); // consume second '{'
                        let mut placeholder = String::new();
                        while let Some(ch) = chars.next() {
                            if ch == '}' && chars.peek() == Some(&'}') {
                                chars.next(); // consume second '}'
                                if !placeholder.is_empty() {
                                    placeholders.push(placeholder);
                                }
                                break;
                            } else {
                                placeholder.push(ch);
                            }
                        }
                    }
                }
            }
        }
        
        placeholders
    }

    /// Checks if the workflow is compatible with a given shell.
    ///
    /// If `shells` is `None` or empty, it's considered compatible with all shells.
    ///
    /// # Arguments
    ///
    /// * `shell` - The `Shell` to check compatibility against.
    ///
    /// # Returns
    ///
    /// `true` if compatible, `false` otherwise.
    pub fn is_compatible_with_shell(&self, shell: &Shell) -> bool {
        self.shells.as_ref().map_or(true, |shells| shells.contains(shell))
    }

    /// Determines the category of the workflow based on its tags.
    ///
    /// # Returns
    ///
    /// A `WorkflowCategory` enum value.
    pub fn get_category(&self) -> WorkflowCategory {
        for tag in &self.tags {
            match tag.to_lowercase().as_str() {
                "git" => return WorkflowCategory::Git,
                "docker" => return WorkflowCategory::Docker,
                "kubernetes" | "k8s" => return WorkflowCategory::Kubernetes,
                "aws" => return WorkflowCategory::Aws,
                "database" | "db" => return WorkflowCategory::Database,
                "network" => return WorkflowCategory::Network,
                "file" | "filesystem" => return WorkflowCategory::FileSystem,
                "system" => return WorkflowCategory::System,
                _ => continue,
            }
        }
        WorkflowCategory::Other
    }

    /// Calculates a search relevance score for the workflow based on a query.
    ///
    /// The score is influenced by matches in name, tags, description, commands,
    /// author, and usage frequency.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string.
    ///
    /// # Returns
    ///
    /// A `f32` representing the search score.
    pub fn calculate_search_score(&self, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0;

        // Name match (highest weight)
        if self.name.to_lowercase().contains(&query_lower) {
            score += 10.0;
            if self.name.to_lowercase() == query_lower {
                score += 20.0; // Exact match bonus
            }
        }

        // Tag match (high weight)
        for tag in &self.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 8.0;
                if tag.to_lowercase() == query_lower {
                    score += 12.0; // Exact match bonus
                }
            }
        }

        // Description match (medium weight)
        if let Some(description) = &self.description {
            if description.to_lowercase().contains(&query_lower) {
                score += 5.0;
            }
        }

        // Command match (lower weight) - now check within steps
        for step in &self.steps {
            if let WorkflowStepType::Command { command, .. } = &step.step_type {
                if command.to_lowercase().contains(&query_lower) {
                    score += 3.0;
                }
            }
        }

        // Author match (low weight)
        if let Some(author) = &self.author {
            if author.to_lowercase().contains(&query_lower) {
                score += 2.0;
            }
        }

        // Usage frequency bonus (logarithmic scale)
        score += (self.usage_count as f32).log10();

        score
    }
}

/// Categorization of workflows.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowCategory {
    Git,
    Docker,
    Kubernetes,
    Aws,
    Database,
    Network,
    FileSystem,
    System,
    Other,
}

impl std::fmt::Display for WorkflowCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowCategory::Git => write!(f, "Git"),
            WorkflowCategory::Docker => write!(f, "Docker"),
            WorkflowCategory::Kubernetes => write!(f, "Kubernetes"),
            WorkflowCategory::Aws => write!(f, "AWS"),
            WorkflowCategory::Database => write!(f, "Database"),
            WorkflowCategory::Network => write!(f, "Network"),
            WorkflowCategory::FileSystem => write!(f, "File System"),
            WorkflowCategory::System => write!(f, "System"),
            WorkflowCategory::Other => write!(f, "Other"),
        }
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shell::Zsh => write!(f, "zsh"),
            Shell::Bash => write!(f, "bash"),
            Shell::Fish => write!(f, "fish"),
        }
    }
}

impl std::str::FromStr for Shell {
    type Err = WorkflowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zsh" => Ok(Shell::Zsh),
            "bash" => Ok(Shell::Bash),
            "fish" => Ok(Shell::Fish),
            _ => Err(WorkflowError::UnsupportedShell(Shell::Bash)), // Default error
        }
    }
}
