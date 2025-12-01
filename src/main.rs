use anyhow::Result;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webmux::config::Config;
use webmux::serial::SerialManager;
use webmux::web;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "terminal_access_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Terminal Access Server");

    // Load configuration
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.yaml".to_string());

    info!("Loading configuration from: {}", config_path);

    let config = Config::from_file(&config_path).map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    config.validate().map_err(|e| {
        error!("Configuration validation failed: {}", e);
        e
    })?;

    info!(
        "Configuration loaded successfully with {} connection(s)",
        config.serial_connections.len()
    );

    // Create serial manager
    let serial_manager = SerialManager::new();

    // Initialize serial connections
    for conn_config in config.serial_connections {
        match serial_manager.add_connection(conn_config.clone()).await {
            Ok(_) => info!("Successfully initialized connection: {}", conn_config.name),
            Err(e) => error!(
                "Failed to initialize connection {}: {}",
                conn_config.name, e
            ),
        }
    }

    // Create web server
    let app = web::create_router(serial_manager.clone());

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting web server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!("Server is ready and listening on {}", bind_addr);
    info!("API endpoints:");
    info!("  GET  /health");
    info!("  GET  /api/connections");
    info!("  GET  /api/connections/:name");
    info!("  POST /api/connections/:name/send");
    info!("  GET  /api/connections/:name/stats");
    info!("  WS   /api/connections/:name/ws");

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Shutting down serial connections...");
    serial_manager.shutdown().await;

    info!("Server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}
