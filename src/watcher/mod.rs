use anyhow::Result;
use log::info;
use tokio::sync::mpsc;
use notify::{RecommendedWatcher, Watcher as NotifyWatcher, RecursiveMode, EventKind};
use std::path::PathBuf;
use std::time::Duration;

/// Events from the file system watcher.
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    FileChanged { path: PathBuf },
    FileCreated { path: PathBuf },
    FileDeleted { path: PathBuf },
    DirectoryCreated { path: PathBuf },
    DirectoryDeleted { path: PathBuf },
    Error(String),
    // Add more event types as needed (e.g., Renamed, MetadataChanged)
}

pub struct Watcher {
    event_sender: mpsc::Sender<WatcherEvent>,
    // Store the actual watcher instance to keep it alive
    _watcher: Option<RecommendedWatcher>,
}

impl Watcher {
    pub fn new(event_sender: mpsc::Sender<WatcherEvent>) -> Self {
        Self {
            event_sender,
            _watcher: None,
        }
    }

    pub async fn init(&mut self) -> Result<()> { // Changed to mutable self
        info!("File system watcher initialized.");
        // Initialize the actual watcher here
        let sender_clone = self.event_sender.clone();
        let watcher_result = RecommendedWatcher::new(move |res| {
            match res {
                Ok(event) => {
                    // Filter for relevant event kinds and send
                    let event_to_send = match event.kind {
                        EventKind::Create(notify::event::CreateKind::File) => {
                            event.paths.first().map(|p| WatcherEvent::FileCreated { path: p.clone() })
                        },
                        EventKind::Create(notify::event::CreateKind::Folder) => {
                            event.paths.first().map(|p| WatcherEvent::DirectoryCreated { path: p.clone() })
                        },
                        EventKind::Remove(notify::event::RemoveKind::File) => {
                            event.paths.first().map(|p| WatcherEvent::FileDeleted { path: p.clone() })
                        },
                        EventKind::Remove(notify::event::RemoveKind::Folder) => {
                            event.paths.first().map(|p| WatcherEvent::DirectoryDeleted { path: p.clone() })
                        },
                        EventKind::Modify(_) => {
                            event.paths.first().map(|p| WatcherEvent::FileChanged { path: p.clone() })
                        },
                        _ => None, // Ignore other event types for now
                    };
                    if let Some(e) = event_to_send {
                        // Blocking send is fine here as it's a separate thread/task
                        if let Err(err) = tokio::runtime::Handle::current().block_on(sender_clone.send(e)) {
                            log::error!("Failed to send watcher event: {}", err);
                        }
                    }
                },
                Err(e) => {
                    log::error!("Watcher error: {}", e);
                    if let Err(err) = tokio::runtime::Handle::current().block_on(sender_clone.send(WatcherEvent::Error(e.to_string()))) {
                        log::error!("Failed to send watcher error event: {}", err);
                    }
                },
            }
        }, notify::Config::default().with_poll_interval(Duration::from_secs(1))); // Poll every second

        match watcher_result {
            Ok(watcher) => {
                self._watcher = Some(watcher);
                Ok(())
            },
            Err(e) => Err(anyhow!("Failed to create file system watcher: {}", e)),
        }
    }

    /// Starts watching a specific path.
    pub async fn watch_path(&mut self, path: &PathBuf, recursive: bool) -> Result<()> {
        if let Some(watcher) = &mut self._watcher {
            let mode = if recursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
            watcher.watch(path, mode)?;
            info!("Watching path: {:?} (recursive: {})", path, recursive);
            Ok(())
        } else {
            Err(anyhow!("Watcher not initialized."))
        }
    }

    /// Stops watching a specific path.
    pub async fn unwatch_path(&mut self, path: &PathBuf) -> Result<()> {
        if let Some(watcher) = &mut self._watcher {
            watcher.unwatch(path)?;
            info!("Unwatching path: {:?}", path);
            Ok(())
        } else {
            Err(anyhow!("Watcher not initialized."))
        }
    }
}

pub fn init() {
    info!("watcher module loaded");
}
