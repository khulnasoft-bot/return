use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod pty;

/// Manages the execution and lifecycle of commands via PTY sessions.
pub struct CommandManager {
    active_ptys: Arc<Mutex<HashMap<String, pty::PtySession>>>, // command_id -> PtySession
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            active_ptys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Executes a command in a new PTY session.
    /// Returns a unique command ID and the PtySession.
    pub async fn execute_command(&self, command: &str, args: &[&str]) -> Result<(String, pty::PtySession)> {
        info!("Executing command: {} with args: {:?}", command, args);
        let command_id = uuid::Uuid::new_v4().to_string();

        let pty_session = pty::PtySession::new(command, args).await?;
        let pty_session_clone = pty_session.clone(); // Clone for storage

        let mut active_ptys = self.active_ptys.lock().await;
        active_ptys.insert(command_id.clone(), pty_session_clone);

        info!("Command '{}' started with ID: {}", command, command_id);

        // Spawn a task to clean up the PTY session when the command finishes
        let active_ptys_clone = self.active_ptys.clone();
        let command_id_clone = command_id.clone();
        tokio::spawn(async move {
            let exit_status = pty_session.wait().await;
            match exit_status {
                Ok(status) => info!("Command {} (ID: {}) finished with status: {:?}", command, command_id_clone, status),
                Err(e) => error!("Error waiting for command {} (ID: {}): {}", command, command_id_clone, e),
            }
            let mut active_ptys = active_ptys_clone.lock().await;
            active_ptys.remove(&command_id_clone);
            info!("PTY session for command ID {} removed.", command_id_clone);
        });

        Ok((command_id, pty_session))
    }

    /// Sends input to a running command's PTY session.
    pub async fn send_input(&self, command_id: &str, input: &str) -> Result<()> {
        let active_ptys = self.active_ptys.lock().await;
        if let Some(pty_session) = active_ptys.get(command_id) {
            info!("Sending input to command ID {}: {:?}", command_id, input);
            pty_session.write_input(input).await
        } else {
            warn!("Command with ID {} not found or not active.", command_id);
            Err(anyhow!("Command with ID {} not found or not active.", command_id))
        }
    }

    /// Terminates a running command's PTY session.
    pub async fn terminate_command(&self, command_id: &str) -> Result<()> {
        let mut active_ptys = self.active_ptys.lock().await;
        if let Some(pty_session) = active_ptys.remove(command_id) {
            info!("Terminating command with ID: {}", command_id);
            pty_session.terminate().await?;
            info!("Command with ID {} terminated successfully.", command_id);
            Ok(())
        } else {
            warn!("Command with ID {} not found or not active.", command_id);
            Err(anyhow!("Command with ID {} not found or not active.", command_id))
        }
    }

    /// Lists all currently active command IDs.
    pub async fn list_active_commands(&self) -> Vec<String> {
        let active_ptys = self.active_ptys.lock().await;
        active_ptys.keys().cloned().collect()
    }
}

pub fn init() {
    info!("command module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_execute_command_and_read_output() {
        let manager = CommandManager::new();
        let (command_id, mut pty_session) = manager.execute_command("echo", &["hello world"]).await.unwrap();

        let mut output = String::new();
        let mut buf = vec![0; 1024];
        let n = pty_session.read_output(&mut buf).await.unwrap();
        output.push_str(&String::from_utf8_lossy(&buf[..n]));

        assert!(output.contains("hello world"));

        // Ensure the command is cleaned up
        sleep(Duration::from_millis(100)).await; // Give cleanup task time to run
        let active_commands = manager.list_active_commands().await;
        assert!(!active_commands.contains(&command_id));
    }

    #[tokio::test]
    async fn test_send_input_to_command() {
        let manager = CommandManager::new();
        // Use a command that waits for input, like `cat` on Unix or `more` on Windows
        #[cfg(unix)]
        let (command_id, mut pty_session) = manager.execute_command("cat", &[]).await.unwrap();
        #[cfg(windows)]
        let (command_id, mut pty_session) = manager.execute_command("more", &[]).await.unwrap();

        manager.send_input(&command_id, "test input\n").await.unwrap();

        let mut output = String::new();
        let mut buf = vec![0; 1024];
        let n = pty_session.read_output(&mut buf).await.unwrap();
        output.push_str(&String::from_utf8_lossy(&buf[..n]));

        assert!(output.contains("test input"));

        manager.terminate_command(&command_id).await.unwrap();
        let active_commands = manager.list_active_commands().await;
        assert!(!active_commands.contains(&command_id));
    }

    #[tokio::test]
    async fn test_terminate_command() {
        let manager = CommandManager::new();
        // Use a long-running command
        #[cfg(unix)]
        let (command_id, _) = manager.execute_command("sleep", &["5"]).await.unwrap();
        #[cfg(windows)]
        let (command_id, _) = manager.execute_command("ping", &["-n", "5", "127.0.0.1"]).await.unwrap();

        let active_commands = manager.list_active_commands().await;
        assert!(active_commands.contains(&command_id));

        manager.terminate_command(&command_id).await.unwrap();

        let active_commands = manager.list_active_commands().await;
        assert!(!active_commands.contains(&command_id));
    }

    #[tokio::test]
    async fn test_send_input_to_non_existent_command() {
        let manager = CommandManager::new();
        let result = manager.send_input("non-existent-id", "input").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Command with ID non-existent-id not found or not active.");
    }

    #[tokio::test]
    async fn test_terminate_non_existent_command() {
        let manager = CommandManager::new();
        let result = manager.terminate_command("non-existent-id").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Command with ID non-existent-id not found or not active.");
    }
}
