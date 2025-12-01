use crate::config::SerialConnectionConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::info;

pub mod connection;
pub use connection::SerialConnection;

pub type SerialData = Vec<u8>;

#[derive(Clone)]
pub struct SerialManager {
    connections: Arc<RwLock<HashMap<String, SerialConnection>>>,
}

impl Default for SerialManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, config: SerialConnectionConfig) -> Result<()> {
        if !config.enabled {
            info!("Connection {} is disabled, skipping", config.name);
            return Ok(());
        }

        info!(
            "Adding serial connection: {} at {}",
            config.name, config.port
        );

        let connection = SerialConnection::new(config.clone()).await?;

        let mut connections = self.connections.write().await;
        connections.insert(config.name.clone(), connection);

        Ok(())
    }

    pub async fn remove_connection(&self, name: &str) -> Result<()> {
        let mut connections = self.connections.write().await;

        if let Some(mut connection) = connections.remove(name) {
            connection.stop().await;
            info!("Removed serial connection: {}", name);
            Ok(())
        } else {
            anyhow::bail!("Connection not found: {}", name)
        }
    }

    pub async fn get_connection(&self, name: &str) -> Option<SerialConnection> {
        let connections = self.connections.read().await;
        connections.get(name).cloned()
    }

    pub async fn list_connections(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    pub async fn send_data(&self, name: &str, data: &[u8]) -> Result<()> {
        let connections = self.connections.read().await;

        if let Some(connection) = connections.get(name) {
            connection.send(data).await
        } else {
            anyhow::bail!("Connection not found: {}", name)
        }
    }

    pub async fn subscribe(&self, name: &str) -> Result<broadcast::Receiver<SerialData>> {
        let connections = self.connections.read().await;

        if let Some(connection) = connections.get(name) {
            Ok(connection.subscribe())
        } else {
            anyhow::bail!("Connection not found: {}", name)
        }
    }

    pub async fn get_stats(&self, name: &str) -> Result<ConnectionStats> {
        let connections = self.connections.read().await;

        if let Some(connection) = connections.get(name) {
            Ok(connection.get_stats().await)
        } else {
            anyhow::bail!("Connection not found: {}", name)
        }
    }

    pub async fn shutdown(&self) {
        let mut connections = self.connections.write().await;

        for (name, mut connection) in connections.drain() {
            info!("Shutting down connection: {}", name);
            connection.stop().await;
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ConnectionStats {
    /// Name of the serial connection
    pub name: String,
    /// Serial port path
    pub port: String,
    /// Total bytes received from the serial port
    pub bytes_received: u64,
    /// Total bytes sent to the serial port
    pub bytes_sent: u64,
    /// Whether the connection is currently active
    pub is_connected: bool,
    /// Connection uptime in seconds
    pub uptime_seconds: u64,
}
