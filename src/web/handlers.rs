use super::{ApiError, AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectionListItem {
    /// Name of the serial connection
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectionInfo {
    /// Name of the serial connection
    pub name: String,
    /// Serial port path (e.g., /dev/ttyUSB0, COM3)
    pub port: String,
    /// Baud rate in bits per second
    pub baud_rate: u32,
    /// Number of data bits (5, 6, 7, or 8)
    pub data_bits: String,
    /// Number of stop bits (1 or 2)
    pub stop_bits: String,
    /// Parity setting (None, Odd, or Even)
    pub parity: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendDataRequest {
    /// Data to send to the serial port
    pub data: String,
    /// Format of the data (text, hex, or base64)
    #[serde(default)]
    pub format: DataFormat,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DataFormat {
    /// Plain text format
    #[default]
    Text,
    /// Hexadecimal format (e.g., "48656c6c6f" or "48 65 6c 6c 6f")
    Hex,
    /// Base64 encoded format
    Base64,
}

/// List all configured serial connections
#[utoipa::path(
    get,
    path = "/api/connections",
    responses(
        (status = 200, description = "List of all serial connections", body = Vec<ConnectionListItem>),
    ),
    tag = "connections"
)]
pub async fn list_connections(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConnectionListItem>>, ApiError> {
    let connections = state.serial_manager.list_connections().await;
    let items = connections
        .into_iter()
        .map(|name| ConnectionListItem { name })
        .collect();
    Ok(Json(items))
}

/// Get detailed information about a specific serial connection
#[utoipa::path(
    get,
    path = "/api/connections/{name}",
    params(
        ("name" = String, Path, description = "Name of the serial connection")
    ),
    responses(
        (status = 200, description = "Connection information", body = ConnectionInfo),
    ),
    tag = "connections"
)]
pub async fn get_connection_info(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ConnectionInfo>, ApiError> {
    match state.serial_manager.get_connection(&name).await {
        Some(connection) => {
            let config = connection.config();
            Ok(Json(ConnectionInfo {
                name: config.name.clone(),
                port: config.port.clone(),
                baud_rate: config.baud_rate,
                data_bits: (match config.data_bits {
                    crate::config::DataBits::Five => "5",
                    crate::config::DataBits::Six => "6",
                    crate::config::DataBits::Seven => "7",
                    crate::config::DataBits::Eight => "8",
                }).to_string(),
                stop_bits: (match config.stop_bits {
                    crate::config::StopBits::One => "1",
                    crate::config::StopBits::Two => "2",
                }).to_string(),
                parity: format!("{:?}", config.parity),
            }))
        }
        None => {
            // Return empty strings for non-existent connections
            Ok(Json(ConnectionInfo {
                name,
                port: String::new(),
                baud_rate: 0,
                data_bits: String::new(),
                stop_bits: String::new(),
                parity: String::new(),
            }))
        }
    }
}

/// Send data to a serial connection
#[utoipa::path(
    post,
    path = "/api/connections/{name}/send",
    params(
        ("name" = String, Path, description = "Name of the serial connection")
    ),
    request_body = SendDataRequest,
    responses(
        (status = 200, description = "Data sent successfully", body = String),
        (status = 400, description = "Invalid data format"),
        (status = 404, description = "Connection not found"),
    ),
    tag = "data"
)]
pub async fn send_data(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<SendDataRequest>,
) -> Result<&'static str, ApiError> {
    let data = match request.format {
        DataFormat::Text => request.data.into_bytes(),
        DataFormat::Hex => hex::decode(request.data.replace(" ", ""))
            .map_err(|e| anyhow::anyhow!("Invalid hex data: {}", e))?,
        DataFormat::Base64 => general_purpose::STANDARD
            .decode(&request.data)
            .map_err(|e| anyhow::anyhow!("Invalid base64 data: {}", e))?,
    };

    state.serial_manager.send_data(&name, &data).await?;
    Ok("Data sent")
}

/// Get connection statistics
#[utoipa::path(
    get,
    path = "/api/connections/{name}/stats",
    params(
        ("name" = String, Path, description = "Name of the serial connection")
    ),
    responses(
        (status = 200, description = "Connection statistics", body = crate::serial::ConnectionStats),
        (status = 404, description = "Connection not found"),
    ),
    tag = "statistics"
)]
pub async fn get_stats(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<crate::serial::ConnectionStats>, ApiError> {
    let stats = state.serial_manager.get_stats(&name).await?;
    Ok(Json(stats))
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_connection(socket, state, name))
}

async fn websocket_connection(ws: WebSocket, state: AppState, connection_name: String) {
    info!("WebSocket connection established for {}", connection_name);

    let (mut ws_sender, mut ws_receiver) = ws.split();

    // Subscribe to serial data
    let mut serial_rx = match state.serial_manager.subscribe(&connection_name).await {
        Ok(rx) => rx,
        Err(e) => {
            error!(
                "Failed to subscribe to connection {}: {}",
                connection_name, e
            );
            let _ = ws_sender.send(Message::Text(format!("Error: {}", e))).await;
            return;
        }
    };

    let serial_manager = state.serial_manager.clone();
    let connection_name_clone = connection_name.clone();

    // Task to forward serial data to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(data) = serial_rx.recv().await {
            // Send as binary data
            if ws_sender.send(Message::Binary(data.clone())).await.is_err() {
                break;
            }
        }
    });

    // Task to receive data from WebSocket and send to serial port
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Binary(data) => {
                    if let Err(e) = serial_manager
                        .send_data(&connection_name_clone, &data)
                        .await
                    {
                        error!("Failed to send data to serial port: {}", e);
                        break;
                    }
                }
                Message::Text(text) => {
                    let data = text.into_bytes();
                    if let Err(e) = serial_manager
                        .send_data(&connection_name_clone, &data)
                        .await
                    {
                        error!("Failed to send data to serial port: {}", e);
                        break;
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket closed for {}", connection_name_clone);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("WebSocket connection closed for {}", connection_name);
}
