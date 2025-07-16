use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message, WebSocketStream};
use url::Url;

/// Represents a unique session ID.
pub type SessionId = String;

/// Represents a participant in a collaboration session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Participant {
    pub id: String,   // Unique ID for the participant
    pub name: String, // Display name
    pub is_host: bool,
}

/// Messages exchanged during a collaboration session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollaborationMessage {
    /// Sent by a new participant joining the session.
    Hello {
        participant: Participant,
    },
    /// Sent by the host to acknowledge a new participant and provide session info.
    Welcome {
        session_id: SessionId,
        host: Participant,
        active_participants: Vec<Participant>,
    },
    /// Sent when a participant joins or leaves.
    ParticipantUpdate {
        participants: Vec<Participant>,
    },
    /// Text content update (e.g., code, document).
    TextUpdate {
        file_path: String,
        content: String,
    },
    /// Cursor position update.
    CursorUpdate {
        participant_id: String,
        line: usize,
        column: usize,
    },
    /// Command execution request (from client to host).
    CommandRequest {
        command: String,
    },
    /// Command execution result (from host to client).
    CommandResult {
        command: String,
        output: String,
        success: bool,
    },
    /// General chat message.
    Chat {
        sender_id: String,
        message: String,
    },
    /// Error message.
    Error {
        message: String,
    },
    /// Signal to close the session.
    Close,
}

/// Events emitted by the SessionSharingManager for the UI or other modules.
#[derive(Debug, Clone)]
pub enum SessionSharingEvent {
    SessionStarted {
        session_id: SessionId,
        is_host: bool,
    },
    SessionEnded {
        session_id: SessionId,
    },
    ParticipantJoined {
        participant: Participant,
    },
    ParticipantLeft {
        participant: Participant,
    },
    MessageReceived(CollaborationMessage),
    Error(String),
}

/// Represents an active shared session, managed by the host.
#[derive(Debug)]
pub struct SharedSession {
    pub id: SessionId,
    pub host: Participant,
    pub participants: Arc<RwLock<HashMap<String, Participant>>>, // participant_id -> Participant
    pub message_tx: broadcast::Sender<CollaborationMessage>, // For broadcasting messages to all participants
}

impl SharedSession {
    pub fn new(host: Participant) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let (tx, _rx) = broadcast::channel(100); // Channel for broadcasting messages
        let mut participants = HashMap::new();
        participants.insert(host.id.clone(), host.clone());
        Self {
            id,
            host,
            participants: Arc::new(RwLock::new(participants)),
            message_tx: tx,
        }
    }

    pub async fn add_participant(&self, participant: Participant) {
        let mut participants = self.participants.write().await;
        participants.insert(participant.id.clone(), participant);
        info!("Participant added. Current participants: {:?}", participants.keys().collect::<Vec<_>>());
        self.notify_participants_update().await;
    }

    pub async fn remove_participant(&self, participant_id: &str) {
        let mut participants = self.participants.write().await;
        participants.remove(participant_id);
        info!("Participant removed. Current participants: {:?}", participants.keys().collect::<Vec<_>>());
        self.notify_participants_update().await;
    }

    pub async fn get_participants_list(&self) -> Vec<Participant> {
        self.participants.read().await.values().cloned().collect()
    }

    async fn notify_participants_update(&self) {
        let participants_list = self.get_participants_list().await;
        let msg = CollaborationMessage::ParticipantUpdate {
            participants: participants_list,
        };
        if let Err(e) = self.message_tx.send(msg) {
            error!("Failed to broadcast participant update: {}", e);
        }
    }
}

pub struct SessionSharingManager {
    is_host_active: Arc<RwLock<bool>>,
    active_session: Arc<RwLock<Option<Arc<SharedSession>>>>,
    event_sender: broadcast::Sender<SessionSharingEvent>,
    _event_receiver: broadcast::Receiver<SessionSharingEvent>, // Keep one receiver to prevent channel from closing
    my_participant_id: String,
    my_participant_name: String,
}

impl SessionSharingManager {
    pub fn new(my_participant_id: String, my_participant_name: String) -> Self {
        let (tx, rx) = broadcast::channel(100);
        Self {
            is_host_active: Arc::new(RwLock::new(false)),
            active_session: Arc::new(RwLock::new(None)),
            event_sender: tx,
            _event_receiver: rx,
            my_participant_id,
            my_participant_name,
        }
    }

    /// Subscribes to session sharing events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<SessionSharingEvent> {
        self.event_sender.subscribe()
    }

    /// Starts a new collaboration session as the host.
    pub async fn start_host_session(&self, bind_address: &str) -> Result<SessionId> {
        let mut is_host_active = self.is_host_active.write().await;
        if *is_host_active {
            return Err(anyhow!("Host session already active."));
        }

        let host_participant = Participant {
            id: self.my_participant_id.clone(),
            name: self.my_participant_name.clone(),
            is_host: true,
        };
        let session = Arc::new(SharedSession::new(host_participant.clone()));
        let session_id = session.id.clone();

        *self.active_session.write().await = Some(session.clone());
        *is_host_active = true;

        info!("Starting host session with ID: {}", session_id);
        let _ = self.event_sender.send(SessionSharingEvent::SessionStarted {
            session_id: session_id.clone(),
            is_host: true,
        });

        let listener = TcpListener::bind(bind_address).await?;
        info!("Listening for WebSocket connections on {}", bind_address);

        let session_clone = session.clone();
        let event_sender_clone = self.event_sender.clone();
        let is_host_active_clone = self.is_host_active.clone();

        tokio::spawn(async move {
            while *is_host_active_clone.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New WebSocket connection from: {}", addr);
                        let session_arc = session_clone.clone();
                        let event_tx = event_sender_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, session_arc, event_tx).await {
                                error!("Error handling WebSocket connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Error accepting TCP connection: {}", e);
                        // If listener fails, perhaps stop hosting
                        let _ = event_sender_clone.send(SessionSharingEvent::Error(format!("Host listener error: {}", e)));
                        break;
                    }
                }
            }
            info!("Host session listener stopped.");
            let _ = event_sender_clone.send(SessionSharingEvent::SessionEnded { session_id: session_clone.id.clone() });
            *is_host_active_clone.write().await = false;
        });

        Ok(session_id)
    }

    /// Connects to an existing collaboration session as a client.
    pub async fn connect_to_session(&self, host_url: &str, session_id: &str) -> Result<()> {
        let mut is_host_active = self.is_host_active.write().await;
        if *is_host_active {
            return Err(anyhow!("Cannot connect as client while hosting a session."));
        }
        if self.active_session.read().await.is_some() {
            return Err(anyhow!("Already connected to a session."));
        }

        let url = format!("ws://{}/{}", host_url, session_id);
        info!("Connecting to WebSocket host: {}", url);

        let (ws_stream, _) = connect_async(Url::parse(&url)?).await?;
        info!("WebSocket connection established.");

        let (mut write, mut read) = ws_stream.split();

        let my_participant = Participant {
            id: self.my_participant_id.clone(),
            name: self.my_participant_name.clone(),
            is_host: false,
        };

        // Send Hello message
        let hello_msg = CollaborationMessage::Hello {
            participant: my_participant.clone(),
        };
        write.send(Message::Text(serde_json::to_string(&hello_msg)?)).await?;

        let event_sender_clone = self.event_sender.clone();
        let active_session_clone = self.active_session.clone();

        // Handle incoming messages from the host
        tokio::spawn(async move {
            while let Some(msg_res) = read.next().await {
                match msg_res {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<CollaborationMessage>(&text) {
                            Ok(msg) => {
                                match msg {
                                    CollaborationMessage::Welcome { session_id, host, active_participants } => {
                                        info!("Received Welcome from host. Session ID: {}", session_id);
                                        let session = Arc::new(SharedSession {
                                            id: session_id.clone(),
                                            host,
                                            participants: Arc::new(RwLock::new(active_participants.into_iter().map(|p| (p.id.clone(), p)).collect())),
                                            message_tx: broadcast::channel(100).0, // Client doesn't broadcast, just receives
                                        });
                                        *active_session_clone.write().await = Some(session.clone());
                                        let _ = event_sender_clone.send(SessionSharingEvent::SessionStarted {
                                            session_id: session_id.clone(),
                                            is_host: false,
                                        });
                                    },
                                    CollaborationMessage::ParticipantUpdate { participants } => {
                                        if let Some(session) = active_session_clone.read().await.as_ref() {
                                            let mut current_participants = session.participants.write().await;
                                            let old_participants: HashMap<String, Participant> = current_participants.drain().collect();
                                            for p in participants {
                                                if old_participants.get(&p.id).is_none() {
                                                    let _ = event_sender_clone.send(SessionSharingEvent::ParticipantJoined { participant: p.clone() });
                                                }
                                                current_participants.insert(p.id.clone(), p);
                                            }
                                            for p in old_participants.values() {
                                                if current_participants.get(&p.id).is_none() {
                                                    let _ = event_sender_clone.send(SessionSharingEvent::ParticipantLeft { participant: p.clone() });
                                                }
                                            }
                                        }
                                        let _ = event_sender_clone.send(SessionSharingEvent::MessageReceived(msg));
                                    },
                                    CollaborationMessage::Close => {
                                        info!("Host closed the session.");
                                        if let Some(session) = active_session_clone.read().await.take() {
                                            let _ = event_sender_clone.send(SessionSharingEvent::SessionEnded { session_id: session.id.clone() });
                                        }
                                        break;
                                    },
                                    _ => {
                                        let _ = event_sender_clone.send(SessionSharingEvent::MessageReceived(msg));
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize WebSocket message: {}", e);
                                let _ = event_sender_clone.send(SessionSharingEvent::Error(format!("Deserialization error: {}", e)));
                            }
                        }
                    }
                    Ok(Message::Binary(_)) => warn!("Received binary message, not supported yet."),
                    Ok(Message::Ping(_)) => info!("Received WebSocket ping."),
                    Ok(Message::Pong(_)) => info!("Received WebSocket pong."),
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed by peer.");
                        if let Some(session) = active_session_clone.read().await.take() {
                            let _ = event_sender_clone.send(SessionSharingEvent::SessionEnded { session_id: session.id.clone() });
                        }
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket read error: {}", e);
                        let _ = event_sender_clone.send(SessionSharingEvent::Error(format!("WebSocket read error: {}", e)));
                        if let Some(session) = active_session_clone.read().await.take() {
                            let _ = event_sender_clone.send(SessionSharingEvent::SessionEnded { session_id: session.id.clone() });
                        }
                        break;
                    }
                }
            }
            info!("Client WebSocket handler stopped.");
        });

        Ok(())
    }

    /// Sends a message to the active session. Only applicable for clients or host broadcasting.
    pub async fn send_message(&self, message: CollaborationMessage) -> Result<()> {
        let active_session_guard = self.active_session.read().await;
        if let Some(session) = active_session_guard.as_ref() {
            if session.host.id == self.my_participant_id {
                // If I am the host, broadcast the message
                if let Err(e) = session.message_tx.send(message) {
                    error!("Failed to broadcast message from host: {}", e);
                    return Err(anyhow!("Failed to broadcast message: {}", e));
                }
            } else {
                // If I am a client, I need to send it to the host.
                // This requires a separate WebSocket writer for the client.
                // For simplicity, this example assumes client only receives,
                // or a dedicated client-side WebSocket writer would be passed around.
                // TODO: Implement client-side message sending to host.
                warn!("Client attempting to send message, but direct sending to host is not yet implemented.");
                return Err(anyhow!("Client-side message sending not implemented."));
            }
            Ok(())
        } else {
            Err(anyhow!("No active session to send message to."))
        }
    }

    /// Ends the current active session, whether as host or client.
    pub async fn end_session(&self) -> Result<()> {
        let mut active_session_guard = self.active_session.write().await;
        if let Some(session) = active_session_guard.take() {
            info!("Ending session: {}", session.id);
            if *self.is_host_active.read().await {
                // If host, send close message to all participants
                if let Err(e) = session.message_tx.send(CollaborationMessage::Close) {
                    error!("Failed to send close message to participants: {}", e);
                }
                *self.is_host_active.write().await = false;
            }
            let _ = self.event_sender.send(SessionSharingEvent::SessionEnded { session_id: session.id.clone() });
            Ok(())
        } else {
            Err(anyhow!("No active session to end."))
        }
    }

    /// Handles a single WebSocket connection for the host.
    async fn handle_connection(
        stream: TcpStream,
        session: Arc<SharedSession>,
        event_tx: broadcast::Sender<SessionSharingEvent>,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        let mut msg_rx = session.message_tx.subscribe();

        // Task to send messages from the session's broadcast channel to the client
        let mut write_task = tokio::spawn(async move {
            while let Ok(msg) = msg_rx.recv().await {
                if let Err(e) = write.send(Message::Text(serde_json::to_string(&msg).unwrap())).await {
                    error!("Failed to send message to client: {}", e);
                    break;
                }
            }
            info!("WebSocket write task ended.");
        });

        let mut participant_id: Option<String> = None;

        // Task to receive messages from the client
        while let Some(msg_res) = read.next().await {
            match msg_res {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<CollaborationMessage>(&text) {
                        Ok(msg) => {
                            match msg {
                                CollaborationMessage::Hello { participant } => {
                                    info!("Received Hello from new participant: {}", participant.name);
                                    session.add_participant(participant.clone()).await;
                                    participant_id = Some(participant.id.clone());

                                    // Send Welcome message back to the new participant
                                    let welcome_msg = CollaborationMessage::Welcome {
                                        session_id: session.id.clone(),
                                        host: session.host.clone(),
                                        active_participants: session.get_participants_list().await,
                                    };
                                    if let Err(e) = write.send(Message::Text(serde_json::to_string(&welcome_msg).unwrap())).await {
                                        error!("Failed to send welcome message: {}", e);
                                        break;
                                    }
                                    let _ = event_tx.send(SessionSharingEvent::ParticipantJoined { participant });
                                },
                                CollaborationMessage::Close => {
                                    info!("Client requested session close.");
                                    break;
                                },
                                _ => {
                                    // Re-broadcast other messages from this client to all other participants
                                    if let Err(e) = session.message_tx.send(msg) {
                                        error!("Failed to re-broadcast client message: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to deserialize WebSocket message from client: {}", e);
                            let _ = event_tx.send(SessionSharingEvent::Error(format!("Deserialization error from client: {}", e)));
                        }
                    }
                }
                Ok(Message::Binary(_)) => warn!("Received binary message from client, not supported yet."),
                Ok(Message::Ping(_)) => info!("Received WebSocket ping from client."),
                Ok(Message::Pong(_)) => info!("Received WebSocket pong from client."),
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by client.");
                    break;
                }
                Err(e) => {
                    error!("WebSocket read error from client: {}", e);
                    let _ = event_tx.send(SessionSharingEvent::Error(format!("WebSocket read error from client: {}", e)));
                    break;
                }
            }
        }

        // Clean up: remove participant and abort write task
        write_task.abort();
        if let Some(p_id) = participant_id {
            session.remove_participant(&p_id).await;
            let _ = event_tx.send(SessionSharingEvent::ParticipantLeft {
                participant: Participant {
                    id: p_id,
                    name: "Unknown".to_string(), // Name might not be available here
                    is_host: false,
                },
            });
        }
        info!("WebSocket read task ended for a client.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    async fn setup_host() -> (SessionSharingManager, String, broadcast::Receiver<SessionSharingEvent>) {
        let host_id = "host1".to_string();
        let host_name = "HostUser".to_string();
        let manager = SessionSharingManager::new(host_id, host_name);
        let rx = manager.subscribe_events();
        let bind_addr = "127.0.0.1:8080";
        let session_id = manager.start_host_session(bind_addr).await.unwrap();
        // Wait for SessionStarted event
        let _ = rx.recv().await.unwrap();
        (manager, bind_addr.to_string(), rx)
    }

    #[tokio::test]
    async fn test_host_session_lifecycle() {
        let (host_manager, bind_addr, mut rx) = setup_host().await;

        // Verify host is active
        assert!(*host_manager.is_host_active.read().await);
        assert!(host_manager.active_session.read().await.is_some());

        // End session
        host_manager.end_session().await.unwrap();
        assert!(!*host_manager.is_host_active.read().await);
        assert!(host_manager.active_session.read().await.is_none());

        // Verify SessionEnded event
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, SessionSharingEvent::SessionEnded { .. }));
    }

    #[tokio::test]
    async fn test_client_connect_and_disconnect() {
        let (host_manager, bind_addr, mut host_rx) = setup_host().await;
        let session_id = host_manager.active_session.read().await.as_ref().unwrap().id.clone();

        let client_id = "client1".to_string();
        let client_name = "ClientUser".to_string();
        let client_manager = SessionSharingManager::new(client_id.clone(), client_name.clone());
        let mut client_rx = client_manager.subscribe_events();

        // Client connects
        client_manager.connect_to_session(&bind_addr, &session_id).await.unwrap();

        // Verify client SessionStarted event
        let client_event = client_rx.recv().await.unwrap();
        assert!(matches!(client_event, SessionSharingEvent::SessionStarted { is_host: false, .. }));

        // Verify host receives ParticipantJoined event
        let host_event = host_rx.recv().await.unwrap();
        assert!(matches!(host_event, SessionSharingEvent::ParticipantJoined { participant, .. } if participant.id == client_id));

        // Verify host receives ParticipantUpdate event
        let host_event = host_rx.recv().await.unwrap();
        assert!(matches!(host_event, SessionSharingEvent::MessageReceived(CollaborationMessage::ParticipantUpdate { participants }) if participants.len() == 2));

        // Client disconnects
        client_manager.end_session().await.unwrap();

        // Verify client SessionEnded event
        let client_event = client_rx.recv().await.unwrap();
        assert!(matches!(client_event, SessionSharingEvent::SessionEnded { .. }));

        // Verify host receives ParticipantLeft event
        let host_event = host_rx.recv().await.unwrap();
        assert!(matches!(host_event, SessionSharingEvent::ParticipantLeft { participant, .. } if participant.id == client_id));

        // Verify host receives ParticipantUpdate event
        let host_event = host_rx.recv().await.unwrap();
        assert!(matches!(host_event, SessionSharingEvent::MessageReceived(CollaborationMessage::ParticipantUpdate { participants }) if participants.len() == 1));

        host_manager.end_session().await.unwrap();
    }

    #[tokio::test]
    async fn test_host_broadcasts_message() {
        let (host_manager, bind_addr, mut host_rx) = setup_host().await;
        let session_id = host_manager.active_session.read().await.as_ref().unwrap().id.clone();

        let client_manager = SessionSharingManager::new("client1".to_string(), "ClientUser".to_string());
        let mut client_rx = client_manager.subscribe_events();
        client_manager.connect_to_session(&bind_addr, &session_id).await.unwrap();
        let _ = client_rx.recv().await; // SessionStarted
        let _ = host_rx.recv().await; // ParticipantJoined
        let _ = host_rx.recv().await; // ParticipantUpdate

        let chat_message = CollaborationMessage::Chat {
            sender_id: host_manager.my_participant_id.clone(),
            message: "Hello everyone!".to_string(),
        };

        host_manager.send_message(chat_message.clone()).await.unwrap();

        // Client should receive the message
        let client_event = client_rx.recv().await.unwrap();
        assert!(matches!(client_event, SessionSharingEvent::MessageReceived(msg) if msg == chat_message));

        host_manager.end_session().await.unwrap();
        client_manager.end_session().await.unwrap();
    }

    #[tokio::test]
    async fn test_client_cannot_send_message_directly() {
        let (host_manager, bind_addr, _) = setup_host().await;
        let session_id = host_manager.active_session.read().await.as_ref().unwrap().id.clone();

        let client_manager = SessionSharingManager::new("client1".to_string(), "ClientUser".to_string());
        client_manager.connect_to_session(&bind_addr, &session_id).await.unwrap();
        sleep(Duration::from_millis(100)).await; // Give time for connection to establish

        let chat_message = CollaborationMessage::Chat {
            sender_id: client_manager.my_participant_id.clone(),
            message: "I am a client trying to send!".to_string(),
        };

        let result = client_manager.send_message(chat_message).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Client-side message sending not implemented.");

        host_manager.end_session().await.unwrap();
        client_manager.end_session().await.unwrap();
    }
}
