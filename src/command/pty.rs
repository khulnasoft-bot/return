use anyhow::{anyhow, Result};
use log::{error, info};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::task;

#[cfg(unix)]
use portable_pty::{native_pty, CommandBuilder, PtySize};
#[cfg(windows)]
use portable_pty::{native_pty, CommandBuilder, PtySize};

/// Represents an active Pseudo-Terminal (PTY) session.
#[derive(Clone)]
pub struct PtySession {
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _child: Arc<Mutex<Child>>, // Keep child handle to ensure process is managed
}

impl PtySession {
    /// Creates a new PTY session and spawns a command within it.
    pub async fn new(command: &str, args: &[&str]) -> Result<Self> {
        info!("Spawning PTY for command: {} {:?}", command, args);

        let pty_system = native_pty().map_err(|e| anyhow!("Failed to create native PTY system: {}", e))?;

        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| anyhow!("Failed to open PTY: {}", e))?;

        let mut cmd_builder = CommandBuilder::new(command);
        cmd_builder.args(args);

        let child = pair
            .slave
            .spawn_command(cmd_builder)
            .map_err(|e| anyhow!("Failed to spawn command in PTY: {}", e))?;

        let master_reader = pair.master.try_clone_reader().map_err(|e| anyhow!("Failed to clone PTY reader: {}", e))?;
        let master_writer = pair.master.try_clone_writer().map_err(|e| anyhow!("Failed to clone PTY writer: {}", e))?;

        Ok(Self {
            master: Arc::new(Mutex::new(pair.master)),
            reader: Arc::new(Mutex::new(master_reader)),
            writer: Arc::new(Mutex::new(master_writer)),
            _child: Arc::new(Mutex::new(child)),
        })
    }

    /// Reads output from the PTY.
    pub async fn read_output(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let reader_clone = self.reader.clone();
        task::spawn_blocking(move || {
            let mut reader = reader_clone.lock().unwrap();
            reader.read(buffer).map_err(|e| anyhow!("Failed to read from PTY: {}", e))
        })
        .await?
    }

    /// Writes input to the PTY.
    pub async fn write_input(&self, input: &str) -> Result<()> {
        let writer_clone = self.writer.clone();
        let input_bytes = input.as_bytes().to_vec(); // Clone input for the blocking task
        task::spawn_blocking(move || {
            let mut writer = writer_clone.lock().unwrap();
            writer.write_all(&input_bytes).map_err(|e| anyhow!("Failed to write to PTY: {}", e))?;
            writer.flush().map_err(|e| anyhow!("Failed to flush PTY writer: {}", e))
        })
        .await?
    }

    /// Waits for the command running in the PTY to exit.
    pub async fn wait(&self) -> Result<std::process::ExitStatus> {
        let child_clone = self._child.clone();
        task::spawn_blocking(move || {
            let mut child = child_clone.lock().unwrap();
            child.wait().map_err(|e| anyhow!("Failed to wait for child process: {}", e))
        })
        .await?
    }

    /// Terminates the command running in the PTY.
    pub async fn terminate(&self) -> Result<()> {
        let child_clone = self._child.clone();
        task::spawn_blocking(move || {
            let mut child = child_clone.lock().unwrap();
            child.kill().map_err(|e| anyhow!("Failed to kill child process: {}", e))
        })
        .await?
    }

    /// Resizes the PTY.
    pub async fn resize(&self, rows: u16, cols: u16) -> Result<()> {
        let master_clone = self.master.clone();
        task::spawn_blocking(move || {
            let master = master_clone.lock().unwrap();
            master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| anyhow!("Failed to resize PTY: {}", e))
        })
        .await?
    }
}

impl Drop for PtySession {
    /// Attempts to kill the child process when the PtySession is dropped.
    /// This is a best-effort attempt as `drop` cannot be async.
    fn drop(&mut self) {
        info!("Dropping PtySession for child process.");
        let child_clone = self._child.clone();
        tokio::task::block_in_place(move || {
            let mut child = child_clone.lock().unwrap();
            if let Err(e) = child.kill() {
                error!("Failed to kill child process during PtySession drop: {}", e);
            } else {
                info!("Child process killed during PtySession drop.");
            }
        });
    }
}

pub fn init() {
    info!("command/pty module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_pty_session_new_and_read() {
        let mut pty = PtySession::new("echo", &["hello world"]).await.unwrap();
        let mut buf = vec![0; 1024];
        let n = pty.read_output(&mut buf).await.unwrap();
        let output = String::from_utf8_lossy(&buf[..n]);
        assert!(output.contains("hello world"));
        let status = pty.wait().await.unwrap();
        assert!(status.success());
    }

    #[tokio::test]
    async fn test_pty_session_write_and_read() {
        #[cfg(unix)]
        let mut pty = PtySession::new("cat", &[]).await.unwrap();
        #[cfg(windows)]
        let mut pty = PtySession::new("more", &[]).await.unwrap(); // `more` waits for input on Windows

        pty.write_input("test input\n").await.unwrap();

        let mut buf = vec![0; 1024];
        let n = pty.read_output(&mut buf).await.unwrap();
        let output = String::from_utf8_lossy(&buf[..n]);
        assert!(output.contains("test input"));

        pty.terminate().await.unwrap();
        let status = pty.wait().await.unwrap();
        assert!(!status.success()); // Should be killed
    }

    #[tokio::test]
    async fn test_pty_session_terminate() {
        #[cfg(unix)]
        let pty = PtySession::new("sleep", &["5"]).await.unwrap();
        #[cfg(windows)]
        let pty = PtySession::new("ping", &["-n", "5", "127.0.0.1"]).await.unwrap();

        pty.terminate().await.unwrap();
        let status = pty.wait().await.unwrap();
        assert!(!status.success()); // Should be killed
    }

    #[tokio::test]
    async fn test_pty_session_resize() {
        let pty = PtySession::new("echo", &["hello"]).await.unwrap();
        let result = pty.resize(30, 100).await;
        assert!(result.is_ok());
        // No direct way to verify resize effect in test without inspecting internal state or specific command output
    }
}
