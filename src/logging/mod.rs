use anyhow::Result;
use chrono::Local;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SerialLogger {
    file: std::sync::Arc<Mutex<File>>,
    connection_name: String,
}

impl SerialLogger {
    pub async fn new(path: &Path, connection_name: &str) -> Result<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        Ok(Self {
            file: std::sync::Arc::new(Mutex::new(file)),
            connection_name: connection_name.to_string(),
        })
    }

    pub async fn log_received(&self, data: &[u8]) -> Result<()> {
        self.log_data("RX", data).await
    }

    pub async fn log_sent(&self, data: &[u8]) -> Result<()> {
        self.log_data("TX", data).await
    }

    async fn log_data(&self, direction: &str, data: &[u8]) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let hex_data = data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let ascii_data: String = data
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();

        let log_line = format!(
            "[{}] {} | {} | {} bytes | HEX: {} | ASCII: {}\n",
            timestamp,
            self.connection_name,
            direction,
            data.len(),
            hex_data,
            ascii_data
        );

        let mut file = self.file.lock().await;
        file.write_all(log_line.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }
}
