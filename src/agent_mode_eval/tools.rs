use std::collections::HashMap;
use serde_json::{Value, json};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use tokio::process::Command;
use std::path::PathBuf;
use std::env;
use std::fs;
use log::info;

/// Trait for defining a tool that the AI can use.
pub trait Tool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn execute(&self, arguments: String) -> Result<String>;
}

/// Trait for defining an asynchronous tool that the AI can use.
#[async_trait]
pub trait AsyncTool: Send + Sync {
    async fn execute_async(&self, arguments: Value) -> Result<String>;
}

pub struct ToolManager {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub async fn register_default_tools(&mut self) -> Result<()> {
        self.register_tool(Box::new(ListFilesTool));
        self.register_tool(Box::new(ReadFileTool));
        self.register_tool(Box::new(WriteFileTool));
        self.register_tool(Box::new(ExecuteCommandTool));
        self.register_tool(Box::new(ChangeDirectoryTool));
        Ok(())
    }
}

// --- Concrete Tool Implementations ---

pub struct ListFilesTool;

impl Tool for ListFilesTool {
    fn name(&self) -> String { "list_files".to_string() }
    fn description(&self) -> String { "Lists files and directories in a given path.".to_string() }
    fn execute(&self, arguments: String) -> Result<String> {
        let path_str = if arguments.is_empty() { "." } else { &arguments };
        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Ok(format!("Error: Path '{}' does not exist.", path_str));
        }
        if !path.is_dir() {
            return Ok(format!("Error: Path '{}' is not a directory.", path_str));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap_or_default();
            let metadata = entry.metadata()?;
            let entry_type = if metadata.is_dir() { "DIR" } else if metadata.is_file() { "FILE" } else { "OTHER" };
            entries.push(format!("{} ({})", file_name, entry_type));
        }
        Ok(entries.join("\n"))
    }
}

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> String { "read_file".to_string() }
    fn description(&self) -> String { "Reads the content of a specified file.".to_string() }
    fn execute(&self, arguments: String) -> Result<String> {
        let path_str = arguments;
        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Ok(format!("Error: File '{}' does not exist.", path_str));
        }
        if !path.is_file() {
            return Ok(format!("Error: Path '{}' is not a file.", path_str));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        Ok(content)
    }
}

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> String { "write_file".to_string() }
    fn description(&self) -> String { "Writes content to a specified file, overwriting if it exists.".to_string() }
    fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let path_str = args["path"].as_str().ok_or_else(|| anyhow!("Missing 'path' argument"))?;
        let content = args["content"].as_str().ok_or_else(|| anyhow!("Missing 'content' argument"))?;
        let path = PathBuf::from(path_str);

        tokio::fs::write(&path, content).await?;
        Ok(format!("Successfully wrote to file: {}", path_str))
    }
}

pub struct ExecuteCommandTool;

impl Tool for ExecuteCommandTool {
    fn name(&self) -> String { "execute_command".to_string() }
    fn description(&self) -> String { "Executes a shell command and returns its stdout and stderr.".to_string() }
    fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let command_str = args["command"].as_str().ok_or_else(|| anyhow!("Missing 'command' argument"))?;
        let command_args: Vec<String> = args["args"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        let dir = args["dir"].as_str();

        let mut cmd = Command::new(command_str);
        cmd.args(&command_args);
        if let Some(d) = dir {
            cmd.current_dir(d);
        }

        let output = cmd.output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!("Command executed successfully.\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr))
        } else {
            Err(anyhow!("Command failed with exit code {:?}.\nSTDOUT:\n{}\nSTDERR:\n{}", output.status.code(), stdout, stderr))
        }
    }
}

pub struct ChangeDirectoryTool;

impl Tool for ChangeDirectoryTool {
    fn name(&self) -> String { "change_directory".to_string() }
    fn description(&self) -> String { "Changes the current working directory of the shell environment.".to_string() }
    fn execute(&self, arguments: String) -> Result<String> {
        let path_str = arguments;
        let path = PathBuf::from(path_str);

        if !path.exists() {
            return Ok(format!("Error: Path '{}' does not exist.", path_str));
        }
        if !path.is_dir() {
            return Ok(format!("Error: Path '{}' is not a directory.", path_str));
        }

        match env::set_current_dir(&path) {
            Ok(_) => Ok(format!("Successfully changed directory to: {}", path.display())),
            Err(e) => Err(anyhow!("Failed to change directory to '{}': {:?}", path.display(), e)),
        }
    }
}

// Example Tool (Mock)
struct FileSystemTool;

impl Tool for FileSystemTool {
    fn name(&self) -> String {
        "file_system_tool".to_string()
    }

    fn description(&self) -> String {
        "A tool for interacting with the file system.".to_string()
    }

    fn execute(&self, arguments: String) -> Result<String> {
        info!("Executing file system tool with arguments: {}", arguments);
        // Implement file system operations here (read, write, list, etc.)
        Ok("File system operation result (mock)".to_string())
    }
}

pub fn init() {
    info!("agent_mode_eval/tools module loaded");
}
