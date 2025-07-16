use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::io::AsyncReadExt;

pub mod pty;

/// Represents a command to be executed.
#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub description: String,
    pub executable: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub output_format: CommandOutputFormat,
}

/// Defines the expected output format of a command.
#[derive(Debug, Clone)]
pub enum CommandOutputFormat {
    PlainText,
    // Add other formats like Json, Yaml, etc. if needed
}

/// Represents the current status of a running command.
#[derive(Debug, Clone)]
pub enum CommandStatus {
    Running,
    Completed(i32),
    Failed(String),
    Killed,
}

/// Represents a chunk of output from a command.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: CommandStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Events emitted by the CommandManager regarding command lifecycle.
#[derive(Debug, Clone)]
pub enum CommandEvent {
    Output {
        id: String,
        data: Vec<u8>,
        is_stdout: bool,
    },
    Completed {
        id: String,
        exit_code: i32,
    },
    Error {
        id: String,
        message: String,
    },
    Killed {
        id: String,
    },
}

/// Manages the execution and lifecycle of commands via PTY sessions.
pub struct CommandManager {
    active_ptys: Arc<Mutex<HashMap<String, pty::PtySession>>>, // command_id -> PtySession
    event_sender: mpsc::Sender<CommandEvent>,
}

impl CommandManager {
    pub fn new(event_sender: mpsc::Sender<CommandEvent>) -> Self {
        Self {
            active_ptys: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
        }
    }

    /// Executes a command in a new PTY session.
    /// Returns a unique command ID and the PtySession.
    /// This method is for internal use or when direct PtySession control is needed.
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
        let event_sender_clone = self.event_sender.clone();
        tokio::spawn(async move {
            let exit_status = pty_session.wait().await;
            let status_msg = match exit_status {
                Ok(status) => {
                    info!("Command {} (ID: {}) finished with status: {:?}", command, command_id_clone, status);
                    let exit_code = status.code().unwrap_or(-1);
                    let _ = event_sender_clone.send(CommandEvent::Completed { id: command_id_clone.clone(), exit_code }).await;
                    format!("Completed with exit code: {}", exit_code)
                },
                Err(e) => {
                    error!("Error waiting for command {} (ID: {}): {}", command, command_id_clone, e);
                    let _ = event_sender_clone.send(CommandEvent::Error { id: command_id_clone.clone(), message: e.to_string() }).await;
                    format!("Failed: {}", e)
                },
            };
            let mut active_ptys = active_ptys_clone.lock().await;
            active_ptys.remove(&command_id_clone);
            info!("PTY session for command ID {} removed. Status: {}", command_id_clone, status_msg);
        });

        Ok((command_id, pty_session))
    }

    /// Executes a command and streams its output and status updates via an MPSC channel.
    pub async fn execute_command_with_output_channel(
        &self,
        cmd: Command,
        output_tx: mpsc::Sender<CommandOutput>,
    ) -> Result<()> {
        info!("Executing command (with output channel): {} with args: {:?}", cmd.executable, cmd.args);
        let command_id = cmd.id.clone();

        let mut pty_session = pty::PtySession::new(&cmd.executable, &cmd.args.iter().map(|s| s.as_str()).collect::<Vec<&str>>()).await?;
        let pty_session_clone_for_storage = pty_session.clone();

        {
            let mut active_ptys = self.active_ptys.lock().await;
            active_ptys.insert(command_id.clone(), pty_session_clone_for_storage);
        }

        info!("Command '{}' started with ID: {}", cmd.executable, command_id);

        let active_ptys_clone = self.active_ptys.clone();
        let event_sender_clone = self.event_sender.clone();

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            let mut stdout_buffer = String::new();
            let mut stderr_buffer = String::new();
            let mut is_running = true;

            // Read output loop
            while is_running {
                tokio::select! {
                    // Attempt to read output from PTY
                    read_result = pty_session.read_output(&mut buf) => {
                        match read_result {
                            Ok(n) if n > 0 => {
                                let output_chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                                // Simple heuristic: if output contains common error indicators, treat as stderr
                                // A more robust solution would involve parsing shell prompts or using specific PTY features
                                let is_stderr = output_chunk.contains("error:") || output_chunk.contains("failed") || output_chunk.contains("command not found");
                                
                                if is_stderr {
                                    stderr_buffer.push_str(&output_chunk);
                                } else {
                                    stdout_buffer.push_str(&output_chunk);
                                }

                                // Send output chunk to the UI
                                if let Err(e) = output_tx.send(CommandOutput {
                                    status: CommandStatus::Running,
                                    stdout: stdout_buffer.clone(),
                                    stderr: stderr_buffer.clone(),
                                }).await {
                                    error!("Failed to send output chunk for command ID {}: {}", command_id, e);
                                    break; // Stop reading if UI channel is closed
                                }
                                stdout_buffer.clear(); // Clear buffers after sending
                                stderr_buffer.clear();
                            },
                            Ok(_) => {
                                // No bytes read, but not an error. Could mean EOF or temporary pause.
                                // Continue loop, but don't break immediately.
                            },
                            Err(e) => {
                                error!("Error reading from PTY for command ID {}: {}", command_id, e);
                                if let Err(send_err) = output_tx.send(CommandOutput {
                                    status: CommandStatus::Failed(format!("Error reading output: {}", e)),
                                    stdout: stdout_buffer.clone(),
                                    stderr: stderr_buffer.clone(),
                                }).await {
                                    error!("Failed to send error status for command ID {}: {}", command_id, send_err);
                                }
                                is_running = false; // Break out of loop on read error
                            }
                        }
                    }
                    // Wait for the command to complete
                    exit_status_result = pty_session.wait() => {
                        match exit_status_result {
                            Ok(status) => {
                                let exit_code = status.code().unwrap_or(-1);
                                info!("Command {} (ID: {}) finished with status: {:?}", cmd.executable, command_id, status);
                                if let Err(e) = output_tx.send(CommandOutput {
                                    status: CommandStatus::Completed(exit_code),
                                    stdout: stdout_buffer.clone(),
                                    stderr: stderr_buffer.clone(),
                                }).await {
                                    error!("Failed to send completed status for command ID {}: {}", command_id, e);
                                }
                                is_running = false;
                            },
                            Err(e) => {
                                error!("Error waiting for command {} (ID: {}): {}", cmd.executable, command_id, e);
                                if let Err(send_err) = output_tx.send(CommandOutput {
                                    status: CommandStatus::Failed(format!("Error waiting for command: {}", e)),
                                    stdout: stdout_buffer.clone(),
                                    stderr: stderr_buffer.clone(),
                                }).await {
                                    error!("Failed to send failed status for command ID {}: {}", command_id, send_err);
                                }
                                is_running = false;
                            }
                        }
                    }
                }
            }

            // Ensure cleanup of active_ptys map
            let mut active_ptys = active_ptys_clone.lock().await;
            if active_ptys.remove(&command_id).is_some() {
                info!("PTY session for command ID {} removed from active_ptys map.", command_id);
            } else {
                warn!("PTY session for command ID {} was not found in active_ptys map during cleanup.", command_id);
            }
        });

        Ok(())
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
            let _ = self.event_sender.send(CommandEvent::Killed { id: command_id.to_string() }).await;
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
        let (tx, mut rx) = mpsc::channel(100);
        let manager = CommandManager::new(tx);
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
        let (tx, mut rx) = mpsc::channel(100);
        let manager = CommandManager::new(tx);
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
        let (tx, mut rx) = mpsc::channel(100);
        let manager = CommandManager::new(tx);
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
        let (tx, mut rx) = mpsc::channel(100);
        let manager = CommandManager::new(tx);
        let result = manager.send_input("non-existent-id", "input").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Command with ID non-existent-id not found or not active.");
    }

    #[tokio::test]
    async fn test_terminate_non_existent_command() {
        let (tx, mut rx) = mpsc::channel(100);
        let manager = CommandManager::new(tx);
        let result = manager.terminate_command("non-existent-id").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Command with ID non-existent-id not found or not active.");
    }

    #[tokio::test]
    async fn test_execute_command_with_output_channel() {
        let (event_tx, mut event_rx) = mpsc::channel(100);
        let manager = CommandManager::new(event_tx);

        let (output_tx, mut output_rx) = mpsc::channel(100);
        let command_id = uuid::Uuid::new_v4().to_string();
        let cmd = Command {
            id: command_id.clone(),
            name: "echo_test".to_string(),
            description: "Test echo command".to_string(),
            executable: "echo".to_string(),
            args: vec!["hello from channel".to_string()],
            env: HashMap::new(),
            working_dir: None,
            output_format: CommandOutputFormat::PlainText,
        };

        manager.execute_command_with_output_channel(cmd, output_tx).await.unwrap();

        let mut received_output = String::new();
        let mut completed = false;

        while let Some(output) = output_rx.recv().await {
            received_output.push_str(&output.stdout);
            received_output.push_str(&output.stderr);
            if let CommandStatus::Completed(code) = output.status {
                assert_eq!(code, 0);
                completed = true;
                break;
            } else if let CommandStatus::Failed(err) = output.status {
                panic!("Command failed: {}", err);
            }
        }

        assert!(completed);
        assert!(received_output.contains("hello from channel"));

        // Ensure the command is cleaned up from active_ptys
        sleep(Duration::from_millis(100)).await; // Give cleanup task time to run
        let active_commands = manager.list_active_commands().await;
        assert!(!active_commands.contains(&command_id));
    }

    #[tokio::test]
    async fn test_pty_session_drop_kills_child() {
        #[cfg(unix)]
        let pty = pty::PtySession::new("sleep", &["5"]).await.unwrap();
        #[cfg(windows)]
        let pty = pty::PtySession::new("ping", &["-n", "5", "127.0.0.1"]).await.unwrap();

        let child_pid = {
            let child_lock = pty._child.lock().unwrap();
            child_lock.id().unwrap()
        };

        // Drop the pty session, which should trigger the kill
        drop(pty);

        // Give some time for the process to terminate
        sleep(Duration::from_millis(500)).await;

        // Check if the process is still running (platform-specific)
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            let result = kill(nix::unistd::Pid::from_raw(child_pid as i32), None);
            // If kill returns ESRCH, it means the process does not exist
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), nix::errno::Errno::ESRCH);
        }
        #[cfg(windows)]
        {
            use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
            use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
            use winapi::shared::minwindef::FALSE;
            use std::ptr::null_mut;

            let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, child_pid) };
            // If handle is null, process does not exist or access denied
            assert!(handle.is_null());
        }
    }
}
