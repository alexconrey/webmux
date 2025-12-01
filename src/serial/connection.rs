use crate::config::SerialConnectionConfig;
use crate::logging::SerialLogger;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_serial::SerialPortBuilderExt;
use tracing::{error, info, warn};

use super::{ConnectionStats, SerialData};

#[derive(Clone)]
pub struct SerialConnection {
    config: SerialConnectionConfig,
    tx: mpsc::Sender<SerialData>,
    rx: broadcast::Sender<SerialData>,
    stats: Arc<RwLock<Stats>>,
    shutdown_tx: Arc<RwLock<Option<mpsc::Sender<()>>>>,
}

#[derive(Debug)]
struct Stats {
    bytes_received: u64,
    bytes_sent: u64,
    is_connected: bool,
    start_time: Instant,
}

impl SerialConnection {
    pub async fn new(config: SerialConnectionConfig) -> Result<Self> {
        let (tx, mut write_rx) = mpsc::channel::<SerialData>(100);
        let (read_tx, _) = broadcast::channel::<SerialData>(1000);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        let stats = Arc::new(RwLock::new(Stats {
            bytes_received: 0,
            bytes_sent: 0,
            is_connected: true,
            start_time: Instant::now(),
        }));

        let logger = if config.logging.enabled {
            Some(SerialLogger::new(&config.logging.path, &config.name).await?)
        } else {
            None
        };

        // Open the serial port
        let port = tokio_serial::new(&config.port, config.baud_rate)
            .data_bits(config.data_bits.into())
            .stop_bits(config.stop_bits.into())
            .parity(config.parity.into())
            .flow_control(config.flow_control.into())
            .open_native_async()?;

        info!(
            "Opened serial port {} for connection {}",
            config.port, config.name
        );

        let (mut read_half, mut write_half) = tokio::io::split(port);

        // Clone necessary data for the tasks
        let read_tx_clone = read_tx.clone();
        let stats_clone = stats.clone();
        let config_clone = config.clone();
        let logger_clone = logger.clone();

        // Spawn read task
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 1024];

            loop {
                tokio::select! {
                    result = read_half.read(&mut buffer) => {
                        match result {
                            Ok(0) => {
                                warn!("Serial port {} closed", config_clone.port);
                                break;
                            }
                            Ok(n) => {
                                let data = buffer[..n].to_vec();

                                // Update stats
                                {
                                    let mut stats = stats_clone.write().await;
                                    stats.bytes_received += n as u64;
                                }

                                // Log if enabled
                                if let Some(ref logger) = logger_clone {
                                    if let Err(e) = logger.log_received(&data).await {
                                        error!("Failed to log data: {}", e);
                                    }
                                }

                                // Broadcast to subscribers
                                if let Err(e) = read_tx_clone.send(data) {
                                    error!("Failed to broadcast data: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Error reading from serial port {}: {}", config_clone.port, e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Shutting down read task for {}", config_clone.name);
                        break;
                    }
                }
            }

            let mut stats = stats_clone.write().await;
            stats.is_connected = false;
        });

        // Clone necessary data for write task
        let stats_clone = stats.clone();
        let config_clone = config.clone();

        // Spawn write task
        tokio::spawn(async move {
            while let Some(data) = write_rx.recv().await {
                match write_half.write_all(&data).await {
                    Ok(_) => {
                        let mut stats = stats_clone.write().await;
                        stats.bytes_sent += data.len() as u64;

                        if let Some(ref logger) = logger {
                            if let Err(e) = logger.log_sent(&data).await {
                                error!("Failed to log sent data: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error writing to serial port {}: {}", config_clone.port, e);
                    }
                }
            }
        });

        Ok(Self {
            config,
            tx,
            rx: read_tx,
            stats,
            shutdown_tx: Arc::new(RwLock::new(Some(shutdown_tx))),
        })
    }

    pub async fn send(&self, data: &[u8]) -> Result<()> {
        self.tx
            .send(data.to_vec())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send data: {}", e))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SerialData> {
        self.rx.subscribe()
    }

    pub fn config(&self) -> &SerialConnectionConfig {
        &self.config
    }

    pub async fn get_stats(&self) -> ConnectionStats {
        let stats = self.stats.read().await;
        ConnectionStats {
            name: self.config.name.clone(),
            port: self.config.port.clone(),
            bytes_received: stats.bytes_received,
            bytes_sent: stats.bytes_sent,
            is_connected: stats.is_connected,
            uptime_seconds: stats.start_time.elapsed().as_secs(),
        }
    }

    pub async fn stop(&mut self) {
        let mut shutdown = self.shutdown_tx.write().await;
        if let Some(tx) = shutdown.take() {
            let _ = tx.send(()).await;
        }
    }
}
