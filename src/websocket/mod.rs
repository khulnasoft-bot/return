use anyhow::Result;
use log::info;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use std::collections::HashMap;

/// Events from the WebSocket server.
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    ClientConnected(SocketAddr),
    ClientDisconnected(SocketAddr),
    MessageReceived(SocketAddr, String), // (client_addr, message_content)
    Error(SocketAddr, String), // (client_addr, error_message)
    ServerStarted(SocketAddr),
    ServerError(String),
}

pub struct WebSocketServer {
    event_sender: mpsc::Sender<WebSocketEvent>,
    listen_addr: SocketAddr,
    clients: HashMap<SocketAddr, mpsc::Sender<Message>>, // Store active client connections for broadcasting
}

impl WebSocketServer {
    pub fn new() -> Self {
        let (tx, _) = mpsc::channel(100); // Dummy sender, will be replaced if needed
        Self {
            event_sender: tx,
            listen_addr: "127.0.0.1:9000".parse().unwrap(), // Default listen address
            clients: HashMap::new(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        info!("WebSocket server initialized.");
        Ok(())
    }

    pub fn set_event_sender(&mut self, sender: mpsc::Sender<WebSocketEvent>) {
        self.event_sender = sender;
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("WebSocket server listening on {}", self.listen_addr);
        self.event_sender.send(WebSocketEvent::ServerStarted(self.listen_addr)).await?;

        let sender_clone = self.event_sender.clone();
        let mut clients = self.clients.clone();
        tokio::spawn(async move {
            while let Ok((stream, peer_addr)) = listener.accept().await {
                info!("New WebSocket connection from: {}", peer_addr);
                let sender_clone_inner = sender_clone.clone();
                let mut clients_inner = clients.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_connection(stream, peer_addr, sender_clone_inner, &mut clients_inner).await {
                        log::error!("Error handling WebSocket connection from {}: {}", peer_addr, e);
                        let _ = sender_clone.send(WebSocketEvent::Error(peer_addr, e.to_string())).await;
                    }
                });
            }
            let _ = sender_clone.send(WebSocketEvent::ServerError("Listener stopped unexpectedly".to_string())).await;
        });
        Ok(())
    }

    async fn handle_connection(stream: TcpStream, peer_addr: SocketAddr, event_sender: mpsc::Sender<WebSocketEvent>, clients: &mut HashMap<SocketAddr, mpsc::Sender<Message>>) -> Result<()> {
        event_sender.send(WebSocketEvent::ClientConnected(peer_addr)).await?;
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        let (client_tx, mut client_rx) = mpsc::channel(100);
        clients.insert(peer_addr, client_tx.clone());

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    info!("Received message from {}: {}", peer_addr, text);
                    event_sender.send(WebSocketEvent::MessageReceived(peer_addr, text.clone())).await?;
                    // Echo back for demonstration
                    write.send(Message::Text(format!("Echo: {}", text))).await?;
                },
                Ok(Message::Binary(bin)) => {
                    info!("Received binary message from {}: {:?}", peer_addr, bin);
                    // Handle binary messages
                },
                Ok(Message::Ping(ping)) => {
                    write.send(Message::Pong(ping)).await?;
                },
                Ok(Message::Pong(_)) => {},
                Ok(Message::Close(close_frame)) => {
                    info!("Client {} requested close: {:?}", peer_addr, close_frame);
                    break;
                },
                Ok(Message::Frame(_)) => {
                    // This should not happen with `split()`
                },
                Err(e) => {
                    log::error!("WebSocket error for {}: {}", peer_addr, e);
                    event_sender.send(WebSocketEvent::Error(peer_addr, e.to_string())).await?;
                    break;
                }
            }
        }

        clients.remove(&peer_addr);
        event_sender.send(WebSocketEvent::ClientDisconnected(peer_addr)).await?;
        info!("Client {} disconnected.", peer_addr);
        Ok(())
    }

    /// Sends a message to a specific client.
    pub async fn send_to_client(&self, client_addr: SocketAddr, message: String) -> Result<()> {
        info!("Sending message to client {}: {}", client_addr, message);
        if let Some(sender) = self.clients.get(&client_addr) {
            sender.send(Message::Text(message)).await?;
        }
        Ok(())
    }

    /// Broadcasts a message to all connected clients.
    pub async fn broadcast(&self, message: String) -> Result<()> {
        info!("Broadcasting message to all clients: {}", message);
        for sender in self.clients.values() {
            sender.send(Message::Text(message.clone())).await?;
        }
        Ok(())
    }
}

pub fn init() {
    info!("websocket module loaded");
}
