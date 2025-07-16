use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use anyhow::Result;
use tokio::time::{self, Duration};
use log::{info, error, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the data to be synchronized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncData {
    pub files: HashMap<String, String>, // Path -> Content
    pub settings: HashMap<String, String>,
    pub workflows: HashMap<String, String>,
    // Add other data types as needed
}

impl Default for SyncData {
    fn default() -> Self {
        Self {
            files: HashMap::new(),
            settings: HashMap::new(),
            workflows: HashMap::new(),
        }
    }
}

/// Events that the SyncManager can send to other parts of the application.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    SyncStarted,
    SyncCompleted(Result<()>),
    DataQueued,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTarget {
    VercelBlob,
    GitHubGist,
    Custom(String), // For custom endpoints
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub target: SyncTarget,
    pub credentials: HashMap<String, String>, // e.g., API keys, tokens
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 60,
            target: SyncTarget::VercelBlob,
            credentials: HashMap::new(),
        }
    }
}

pub struct SyncManager {
    last_sync_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    data_to_sync: Arc<RwLock<SyncData>>,
    sync_interval: Duration,
    event_sender: mpsc::Sender<SyncEvent>,
    event_receiver: mpsc::Receiver<SyncEvent>,
    config: Arc<RwLock<SyncConfig>>,
    // In a real app, you'd have a client for your cloud storage service
    // cloud_client: Arc<dyn CloudClient + Send + Sync>,
}

impl SyncManager {
    pub fn new(sync_interval_seconds: u64) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            last_sync_time: Arc::new(RwLock::new(None)),
            data_to_sync: Arc::new(RwLock::new(SyncData::default())),
            sync_interval: Duration::from_secs(sync_interval_seconds),
            event_sender: tx,
            event_receiver: rx,
            config: Arc::new(RwLock::new(SyncConfig::default())),
            // cloud_client: Arc::new(MockCloudClient::new()), // Replace with real client
        }
    }

    /// Starts the periodic synchronization task.
    pub async fn start_periodic_sync(&self) {
        let last_sync_time_clone = self.last_sync_time.clone();
        let data_to_sync_clone = self.data_to_sync.clone();
        let sync_interval = self.sync_interval;
        let event_sender_clone = self.event_sender.clone();
        let config_clone = self.config.clone();
        // let cloud_client_clone = self.cloud_client.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(sync_interval);
            interval.tick().await; // Initial tick to wait for the first interval

            loop {
                interval.tick().await;
                info!("Performing periodic sync...");
                let _ = event_sender_clone.send(SyncEvent::SyncStarted).await;

                // Check if sync is enabled
                let config = config_clone.read().await;
                if !config.enabled {
                    warn!("Sync is disabled. Skipping periodic sync.");
                    continue;
                }

                // In a real scenario, you'd send data_to_sync_clone to the cloud_client
                // and handle the response.
                let mut data_to_sync_write = data_to_sync_clone.write().await;
                if !data_to_sync_write.files.is_empty() || !data_to_sync_write.settings.is_empty() || !data_to_sync_write.workflows.is_empty() {
                    info!("Syncing data: {:?}", *data_to_sync_write);
                    // Simulate network delay and success/failure
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    // Simulate success
                    *last_sync_time_clone.write().await = Some(Utc::now());
                    data_to_sync_write.files.clear();
                    data_to_sync_write.settings.clear();
                    data_to_sync_write.workflows.clear();
                    let _ = event_sender_clone.send(SyncEvent::SyncCompleted(Ok(()))).await;
                    info!("Periodic sync completed successfully.");
                } else {
                    info!("No data to sync during periodic check.");
                    let _ = event_sender_clone.send(SyncEvent::SyncCompleted(Ok(()))).await; // Still report completion
                }
            }
        });
    }

    /// Manually triggers a synchronization.
    pub async fn trigger_manual_sync(&self) -> Result<()> {
        info!("Manual sync triggered.");
        let _ = self.event_sender.send(SyncEvent::SyncStarted).await;

        let config = self.config.read().await;
        if !config.enabled {
            warn!("Sync is disabled. Skipping manual sync.");
            return Ok(());
        }

        let mut data_to_sync_write = self.data_to_sync.write().await;
        if !data_to_sync_write.files.is_empty() || !data_to_sync_write.settings.is_empty() || !data_to_sync_write.workflows.is_empty() {
            info!("Syncing data: {:?}", *data_to_sync_write);
            // Simulate network delay and success/failure
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Simulate success
            *self.last_sync_time.write().await = Some(Utc::now());
            data_to_sync_write.files.clear();
            data_to_sync_write.settings.clear();
            data_to_sync_write.workflows.clear();
            let _ = self.event_sender.send(SyncEvent::SyncCompleted(Ok(()))).await;
            info!("Manual sync completed successfully.");
            Ok(())
        } else {
            info!("No data to sync during manual trigger.");
            let _ = self.event_sender.send(SyncEvent::SyncCompleted(Ok(()))).await;
            Ok(())
        }
    }

    /// Queues data to be synchronized.
    pub async fn queue_data(&self, data: SyncData) {
        let mut current_data = self.data_to_sync.write().await;
        current_data.files.extend(data.files);
        current_data.settings.extend(data.settings);
        current_data.workflows.extend(data.workflows);
        let _ = self.event_sender.send(SyncEvent::DataQueued).await;
        info!("Data queued for sync.");
    }

    /// Returns a receiver for sync events.
    pub fn subscribe_events(&self) -> mpsc::Receiver<SyncEvent> {
        self.event_receiver.clone()
    }

    /// Gets the last synchronization time.
    pub async fn get_last_sync_time(&self) -> Option<DateTime<Utc>> {
        *self.last_sync_time.read().await
    }

    /// Gets the current data queued for synchronization.
    pub async fn get_queued_data(&self) -> SyncData {
        self.data_to_sync.read().await.clone()
    }

    /// Gets the current configuration.
    pub async fn get_config(&self) -> SyncConfig {
        self.config.read().await.clone()
    }

    /// Sets the configuration.
    pub async fn set_config(&self, config: SyncConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
        info!("Configuration updated.");
    }
}

pub fn init() {
    info!("cloud/sync_manager module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_sync_manager_new() {
        let manager = SyncManager::new(60);
        assert!(manager.get_last_sync_time().await.is_none());
        assert!(manager.get_queued_data().await.files.is_empty());
    }

    #[tokio::test]
    async fn test_sync_manager_queue_data() {
        let manager = SyncManager::new(60);
        let mut rx = manager.subscribe_events();

        let mut data = SyncData::default();
        data.files.insert("file1.txt".to_string(), "content1".to_string());
        manager.queue_data(data.clone()).await;

        assert_eq!(manager.get_queued_data().await.files.len(), 1);
        assert_eq!(rx.recv().await, Some(SyncEvent::DataQueued));
    }

    #[tokio::test]
    async fn test_sync_manager_trigger_manual_sync() {
        let manager = SyncManager::new(60);
        let mut rx = manager.subscribe_events();

        let mut data = SyncData::default();
        data.files.insert("file1.txt".to_string(), "content1".to_string());
        manager.queue_data(data).await;
        let _ = rx.recv().await; // Consume DataQueued event

        let sync_result = manager.trigger_manual_sync().await;
        assert!(sync_result.is_ok());

        assert_eq!(rx.recv().await, Some(SyncEvent::SyncStarted));
        assert!(matches!(rx.recv().await, Some(SyncEvent::SyncCompleted(Ok(_)))));
        assert!(manager.get_last_sync_time().await.is_some());
        assert!(manager.get_queued_data().await.files.is_empty());
    }

    #[tokio::test]
    async fn test_sync_manager_periodic_sync() {
        let manager = SyncManager::new(1); // 1 second interval for testing
        let mut rx = manager.subscribe_events();

        let mut data = SyncData::default();
        data.files.insert("file_periodic.txt".to_string(), "content_periodic".to_string());
        manager.queue_data(data).await;
        let _ = rx.recv().await; // Consume DataQueued event

        manager.start_periodic_sync().await;

        // Wait for the first periodic sync to start and complete
        let event1 = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert_eq!(event1, SyncEvent::SyncStarted);

        let event2 = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert!(matches!(event2, SyncEvent::SyncCompleted(Ok(_))));

        assert!(manager.get_last_sync_time().await.is_some());
        assert!(manager.get_queued_data().await.files.is_empty());

        // Queue more data and wait for the next periodic sync
        let mut data2 = SyncData::default();
        data2.files.insert("file_periodic2.txt".to_string(), "content_periodic2".to_string());
        manager.queue_data(data2).await;
        let _ = rx.recv().await; // Consume DataQueued event

        let event3 = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert_eq!(event3, SyncEvent::SyncStarted);

        let event4 = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert!(matches!(event4, SyncEvent::SyncCompleted(Ok(_))));
    }

    #[tokio::test]
    async fn test_sync_manager_no_data_periodic_sync() {
        let manager = SyncManager::new(1); // 1 second interval for testing
        let mut rx = manager.subscribe_events();

        manager.start_periodic_sync().await;

        // Wait for the first periodic sync to start and complete (should be no data)
        let event1 = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert_eq!(event1, SyncEvent::SyncStarted);

        let event2 = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert!(matches!(event2, SyncEvent::SyncCompleted(Ok(_))));

        assert!(manager.get_last_sync_time().await.is_some()); // Should still update last sync time
        assert!(manager.get_queued_data().await.files.is_empty());
    }

    #[tokio::test]
    async fn test_sync_manager_config() {
        let manager = SyncManager::new(60);
        let mut config = manager.get_config().await;
        config.enabled = true;
        config.interval_minutes = 30;
        config.target = SyncTarget::GitHubGist;
        config.credentials.insert("api_key".to_string(), "12345".to_string());
        manager.set_config(config).await;

        let updated_config = manager.get_config().await;
        assert!(updated_config.enabled);
        assert_eq!(updated_config.interval_minutes, 30);
        assert_eq!(updated_config.target, SyncTarget::GitHubGist);
        assert_eq!(updated_config.credentials.get("api_key"), Some(&"12345".to_string()));
    }
}
