use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::sync::Arc;
use crate::ai::assistant::Tool; // Import the Tool trait
use crate::virtual_fs::VirtualFileSystem;
use crate::command::CommandManager;
use log::info;
use serde_json::Value;
use tokio::sync::mpsc; // For command execution output

/// Tool for listing files in a directory.
pub struct ListFilesTool {
    fs: Arc<VirtualFileSystem>,
}

impl ListFilesTool {
    pub fn new(fs: Arc<VirtualFileSystem>) -> Self {
        Self { fs }
    }
}

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> String {
        "list_files".to_string()
    }

    fn description(&self) -> String {
        "Lists files and directories in a specified path. Arguments: {\"path\": \"string\"}".to_string()
    }

    async fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing 'path' argument for list_files"))?;
        
        info!("Executing list_files for path: {}", path);
        let entries = self.fs.list_dir(path).await?;
        Ok(serde_json::to_string_pretty(&entries)?)
    }
}

/// Tool for reading the content of a file.
pub struct ReadFileTool {
    fs: Arc<VirtualFileSystem>,
}

impl ReadFileTool {
    pub fn new(fs: Arc<VirtualFileSystem>) -> Self {
        Self { fs }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> String {
        "read_file".to_string()
    }

    fn description(&self) -> String {
        "Reads the content of a specified file. Arguments: {\"path\": \"string\"}".to_string()
    }

    async fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing 'path' argument for read_file"))?;
        
        info!("Executing read_file for path: {}", path);
        let content = self.fs.read_file(path).await?;
        Ok(content)
    }
}

/// Tool for writing content to a file.
pub struct WriteFileTool {
    fs: Arc<VirtualFileSystem>,
}

impl WriteFileTool {
    pub fn new(fs: Arc<VirtualFileSystem>) -> Self {
        Self { fs }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> String {
        "write_file".to_string()
    }

    fn description(&self) -> String {
        "Writes content to a specified file. Arguments: {\"path\": \"string\", \"content\": \"string\"}".to_string()
    }

    async fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing 'path' argument for write_file"))?;
        let content = args["content"].as_str().ok_or(anyhow!("Missing 'content' argument for write_file"))?;
        
        info!("Executing write_file for path: {}", path);
        self.fs.write_file(path, content.to_string()).await?;
        Ok(format!("Successfully wrote to file: {}", path))
    }
}

/// Tool for executing a shell command.
pub struct ExecuteCommandTool {
    command_manager: Arc<CommandManager>,
}

impl ExecuteCommandTool {
    pub fn new(command_manager: Arc<CommandManager>) -> Self {
        Self { command_manager }
    }
}

#[async_trait]
impl Tool for ExecuteCommandTool {
    fn name(&self) -> String {
        "execute_command".to_string()
    }

    fn description(&self) -> String {
        "Executes a shell command and returns its stdout and stderr. Arguments: {\"command\": \"string\"}".to_string()
    }

    async fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let command_str = args["command"].as_str().ok_or(anyhow!("Missing 'command' argument for execute_command"))?;
        
        info!("Executing command: {}", command_str);
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("No command provided to execute_command"));
        }
        let cmd_executable = parts[0].to_string();
        let cmd_args = parts[1..].iter().map(|s| s.to_string()).collect();

        let (tx, mut rx) = mpsc::channel(100); // Channel to capture command output

        let cmd_obj = crate::command::Command {
            id: uuid::Uuid::new_v4().to_string(),
            name: cmd_executable.clone(),
            description: format!("Tool executed: {}", command_str),
            executable: cmd_executable,
            args: cmd_args,
            env: std::collections::HashMap::new(),
            working_dir: None,
            output_format: crate::command::CommandOutputFormat::PlainText,
        };

        self.command_manager.execute_command_with_output_channel(cmd_obj, tx).await?;

        let mut full_output = String::new();
        let mut stderr_output = String::new();
        let mut exit_code: Option<i32> = None;

        while let Some(output) = rx.recv().await {
            match output.status {
                crate::command::pty::CommandStatus::Running => {
                    if !output.stdout.is_empty() {
                        full_output.push_str(&output.stdout);
                    }
                    if !output.stderr.is_empty() {
                        stderr_output.push_str(&output.stderr);
                    }
                }
                crate::command::pty::CommandStatus::Completed(code) => {
                    exit_code = Some(code);
                    break;
                }
                crate::command::pty::CommandStatus::Failed(error) => {
                    return Err(anyhow!("Command failed: {}", error));
                }
                crate::command::pty::CommandStatus::Killed => {
                    return Err(anyhow!("Command was killed."));
                }
            }
        }

        let result_json = json!({
            "stdout": full_output,
            "stderr": stderr_output,
            "exit_code": exit_code.unwrap_or(-1),
        });
        Ok(serde_json::to_string_pretty(&result_json)?)
    }
}

/// Tool for changing the current working directory.
pub struct ChangeDirectoryTool {
    fs: Arc<VirtualFileSystem>, // VFS can manage current directory
}

impl ChangeDirectoryTool {
    pub fn new(fs: Arc<VirtualFileSystem>) -> Self {
        Self { fs }
    }
}

#[async_trait]
impl Tool for ChangeDirectoryTool {
    fn name(&self) -> String {
        "change_directory".to_string()
    }

    fn description(&self) -> String {
        "Changes the current working directory. Arguments: {\"path\": \"string\"}".to_string()
    }

    async fn execute(&self, arguments: String) -> Result<String> {
        let args: Value = serde_json::from_str(&arguments)?;
        let path = args["path"].as_str().ok_or(anyhow!("Missing 'path' argument for change_directory"))?;
        
        info!("Executing change_directory to path: {}", path);
        self.fs.set_current_dir(path.to_string()).await?;
        Ok(format!("Successfully changed directory to: {}", path))
    }
}

pub fn init() {
    info!("agent_mode_eval/tools module loaded");
}
