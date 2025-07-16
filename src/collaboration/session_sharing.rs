use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::{info, warn, error};
use std::collections::HashMap;

/// Events related to collaboration sessions.
#[derive(Debug, Clone)]
pub enum CollaborationEvent {
    /// A collaboration session has started.
    SessionStarted { address: SocketAddr },
    /// A collaboration session has ended.
    SessionEnded,
    /// A peer has connected to the session.
    PeerConnected { peer_id: String },
    /// A peer has disconnected from the session.
    PeerDisconnected { peer_id: String },
    /// Text content has been updated by a peer.
    TextUpdate { content: String, cursor_pos: usize },
    /// A command has been executed by a peer.
    CommandExecuted { command: String },
    /// An error occurred in the collaboration session.
    Error(String),
}

/// Manages real-time collaboration sessions.
pub struct SessionSharingManager {
    event_sender: mpsc::Sender<CollaborationEvent>,
    // State for active session, connected peers, shared documents, etc.
    is_session_active: bool,
    connected_peers: HashMap<String, SocketAddr>,
    active_sessions: HashMap<Uuid, SharedSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedSession {
    pub id: Uuid,
    pub host_user_id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub active_users: Vec<String>,
    // In a real system, this would contain a snapshot or stream of terminal state
    // For simplicity, we'll just track some metadata.
    pub last_activity: DateTime<Utc>,
}

impl SessionSharingManager {
    pub fn new(event_sender: mpsc::Sender<CollaborationEvent>) -> Self {
        Self {
            event_sender,
            is_session_active: false,
            connected_peers: HashMap::new(),
            active_sessions: HashMap::new(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        info!("Collaboration session sharing manager initialized.");
        Ok(())
    }

    /// Starts a new collaboration session.
    pub async fn start_session(&mut self, bind_address: SocketAddr) -> Result<()> {
        if self.is_session_active {
            warn!("A collaboration session is already active.");
            return Err(anyhow!("Session already active"));
        }

        info!("Starting collaboration session on {}", bind_address);
        // Simulate network listener setup
        self.is_session_active = true;
        self.event_sender.send(CollaborationEvent::SessionStarted { address: bind_address }).await?;
        Ok(())
    }

    /// Ends the current collaboration session.
    pub async fn end_session(&mut self) -> Result<()> {
        if !self.is_session_active {
            warn!("No active collaboration session to end.");
            return Err(anyhow!("No active session"));
        }

        info!("Ending collaboration session.");
        // Simulate network listener teardown, disconnect peers
        self.is_session_active = false;
        self.connected_peers.clear();
        self.active_sessions.clear();
        self.event_sender.send(CollaborationEvent::SessionEnded).await?;
        Ok(())
    }

    /// Simulates a peer connecting to the session.
    pub async fn simulate_peer_connect(&mut self, peer_id: String, peer_addr: SocketAddr) -> Result<()> {
        if !self.is_session_active {
            return Err(anyhow!("No active session to connect to."));
        }
        info!("Simulating peer {} connecting from {}", peer_id, peer_addr);
        self.connected_peers.insert(peer_id.clone(), peer_addr);
        self.event_sender.send(CollaborationEvent::PeerConnected { peer_id }).await?;
        Ok(())
    }

    /// Simulates a peer disconnecting from the session.
    pub async fn simulate_peer_disconnect(&mut self, peer_id: String) -> Result<()> {
        if !self.is_session_active {
            return Err(anyhow!("No active session."));
        }
        if self.connected_peers.remove(&peer_id).is_some() {
            info!("Simulating peer {} disconnecting.", peer_id);
            self.event_sender.send(CollaborationEvent::PeerDisconnected { peer_id }).await?;
        } else {
            warn!("Attempted to disconnect unknown peer: {}", peer_id);
        }
        Ok(())
    }

    /// Sends a text update to all connected peers (mock).
    pub async fn send_text_update(&self, content: String, cursor_pos: usize) -> Result<()> {
        if !self.is_session_active {
            return Err(anyhow!("No active session to send updates."));
        }
        info!("Sending text update (mock): len={}, cursor={}", content.len(), cursor_pos);
        // In a real system, this would send data over network to peers
        // For now, just log and potentially send a local event for testing
        self.event_sender.send(CollaborationEvent::TextUpdate { content, cursor_pos }).await?;
        Ok(())
    }

    /// Sends a command execution event to all connected peers (mock).
    pub async fn send_command_executed(&self, command: String) -> Result<()> {
        if !self.is_session_active {
            return Err(anyhow!("No active session to send commands."));
        }
        info!("Sending command executed event (mock): {}", command);
        self.event_sender.send(CollaborationEvent::CommandExecuted { command }).await?;
        Ok(())
    }

    pub async fn start_host_session(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("Listening for collaboration connections on {}", addr);
        self.event_sender.send(CollaborationEvent::SessionStarted { address: addr }).await?;

        let sender_clone = self.event_sender.clone();
        tokio::spawn(async move {
            while let Ok((stream, peer_addr)) = listener.accept().await {
                info!("New peer connected from: {}", peer_addr);
                let peer_id = format!("{}", peer_addr); // Simple ID for now
                let _ = sender_clone.send(CollaborationEvent::PeerConnected { peer_id: peer_id.clone() }).await;
                
                let ws_stream = match accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        error!("Error during websocket handshake: {:?}", e);
                        let _ = sender_clone.send(CollaborationEvent::Error(format!("Handshake error: {}", e))).await;
                        continue;
                    }
                };

                let (mut write, mut read) = ws_stream.split();
                let peer_id_clone = peer_id.clone();
                let sender_clone_2 = sender_clone.clone();

                tokio::spawn(async move {
                    while let Some(message) = read.next().await {
                        match message {
                            Ok(msg) => {
                                if msg.is_text() {
                                    let text = msg.to_text().unwrap();
                                    match serde_json::from_str::<CollaborationMessage>(text) {
                                        Ok(collab_msg) => {
                                            match collab_msg {
                                                CollaborationMessage::Hello { peer_id: _ } => { /* Already handled by PeerConnected */ },
                                                CollaborationMessage::TextUpdate { content, cursor_pos } => {
                                                    let _ = sender_clone_2.send(CollaborationEvent::TextUpdate { content, cursor_pos }).await;
                                                },
                                                CollaborationMessage::Command { command } => {
                                                    let _ = sender_clone_2.send(CollaborationEvent::CommandExecuted { command }).await;
                                                },
                                                CollaborationMessage::Goodbye { peer_id: _ } => {
                                                    info!("Peer {} sent goodbye.", peer_id_clone);
                                                    break; // Exit loop on goodbye
                                                },
                                            }
                                        },
                                        Err(e) => error!("Failed to parse collaboration message: {:?} from {}", e, text),
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Error receiving message from peer {}: {:?}", peer_id_clone, e);
                                break;
                            }
                        }
                    }
                    info!("Peer {} disconnected.", peer_id_clone);
                    let _ = sender_clone_2.send(CollaborationEvent::PeerDisconnected { peer_id: peer_id_clone }).await;
                });
            }
            let _ = sender_clone.send(CollaborationEvent::SessionEnded).await;
            info!("Host session ended.");
        });
        Ok(())
    }

    pub async fn connect_to_session(&self, addr: SocketAddr) -> Result<()> {
        info!("Attempting to connect to collaboration session at {}", addr);
        let (ws_stream, _) = tokio_tungstenite::connect_async(format!("ws://{}", addr)).await?;
        info!("Connected to collaboration session at {}", addr);

        let (mut write, mut read) = ws_stream.split();
        let sender_clone = self.event_sender.clone();
        let peer_id = uuid::Uuid::new_v4().to_string(); // Client's own ID

        // Send initial Hello message
        let hello_msg = serde_json::to_string(&CollaborationMessage::Hello { peer_id: peer_id.clone() })?;
        write.send(Message::Text(hello_msg)).await?;

        // Spawn task to send updates (e.g., from local editor changes)
        let mut tx_channel_rx = self.event_sender.subscribe(); // Assuming event_sender is a broadcast channel
        tokio::spawn(async move {
            while let Ok(event) = tx_channel_rx.recv().await {
                match event {
                    CollaborationEvent::TextUpdate { content, cursor_pos } => {
                        let msg = CollaborationMessage::TextUpdate { content, cursor_pos };
                        if let Ok(json_msg) = serde_json::to_string(&msg) {
                            if let Err(e) = write.send(Message::Text(json_msg)).await {
                                error!("Failed to send text update: {:?}", e);
                                break;
                            }
                        }
                    },
                    CollaborationEvent::CommandExecuted { command } => {
                        let msg = CollaborationMessage::Command { command };
                        if let Ok(json_msg) = serde_json::to_string(&msg) {
                            if let Err(e) = write.send(Message::Text(json_msg)).await {
                                error!("Failed to send command: {:?}", e);
                                break;
                            }
                        }
                    },
                    _ => {} // Ignore other events for sending
                }
            }
            info!("Client sender task stopped.");
        });

        // Spawn task to receive updates from host
        let sender_clone_2 = sender_clone.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        if msg.is_text() {
                            let text = msg.to_text().unwrap();
                            match serde_json::from_str::<CollaborationMessage>(text) {
                                Ok(collab_msg) => {
                                    match collab_msg {
                                        CollaborationMessage::TextUpdate { content, cursor_pos } => {
                                            let _ = sender_clone_2.send(CollaborationEvent::TextUpdate { content, cursor_pos }).await;
                                        },
                                        CollaborationMessage::Command { command } => {
                                            let _ = sender_clone_2.send(CollaborationEvent::CommandExecuted { command }).await;
                                        },
                                        CollaborationMessage::Hello { peer_id: host_id } => {
                                            info!("Received Hello from host: {}", host_id);
                                            let _ = sender_clone_2.send(CollaborationEvent::PeerConnected { peer_id: host_id }).await;
                                        },
                                        CollaborationMessage::Goodbye { peer_id: host_id } => {
                                            info!("Host {} sent goodbye.", host_id);
                                            let _ = sender_clone_2.send(CollaborationEvent::PeerDisconnected { peer_id: host_id }).await;
                                            break;
                                        },
                                    }
                                },
                                Err(e) => error!("Failed to parse collaboration message from host: {:?} from {}", e, text),
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error receiving message from host: {:?}", e);
                        break;
                    }
                }
            }
            info!("Client receiver task stopped.");
            let _ = sender_clone_2.send(CollaborationEvent::SessionEnded).await;
        });

        Ok(())
    }

    pub fn create_session(&mut self, host_user_id: String, title: String) -> Result<Uuid, String> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let session = SharedSession {
            id: session_id,
            host_user_id: host_user_id.clone(),
            title,
            created_at: now,
            active_users: vec![host_user_id],
            last_activity: now,
        };
        self.active_sessions.insert(session_id, session);
        Ok(session_id)
    }

    pub fn join_session(&mut self, session_id: Uuid, user_id: String) -> Result<(), String> {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            if !session.active_users.contains(&user_id) {
                session.active_users.push(user_id.clone());
                session.last_activity = Utc::now();
                Ok(())
            } else {
                Err("User already in session.".to_string())
            }
        } else {
            Err("Session not found.".to_string())
        }
    }

    pub fn leave_session(&mut self, session_id: Uuid, user_id: String) -> Result<(), String> {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.active_users.retain(|u| u != &user_id);
            session.last_activity = Utc::now();
            if session.active_users.is_empty() {
                self.end_session(session_id)?;
            }
            Ok(())
        } else {
            Err("Session not found.".to_string())
        }
    }

    pub fn get_session_info(&self, session_id: Uuid) -> Option<&SharedSession> {
        self.active_sessions.get(&session_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationMessage {
    Hello { peer_id: String },
    TextUpdate { content: String, cursor_pos: usize },
    Command { command: String },
    Goodbye { peer_id: String },
}

pub fn init() {
    info!("Collaboration session sharing module loaded");
}
