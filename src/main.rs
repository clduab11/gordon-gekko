use anyhow::Result;
use clap::Parser;
use gordon_gekko_core::TradingSystem;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,

    /// Enable sandbox mode (no real trading)
    #[arg(long)]
    sandbox: bool,

    /// Log level (debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Enable GPU acceleration for neural networks
    #[arg(long)]
    gpu: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing subscriber
    init_tracing(&args.log_level)?;

    info!("🚀 Starting Gordon Gekko Trading System v{}", env!("CARGO_PKG_VERSION"));
    info!("📊 Configuration: {}", args.config);
    info!("🏖️  Sandbox mode: {}", args.sandbox);
    info!("🔥 GPU acceleration: {}", args.gpu);

    // Load configuration
    let config = Arc::new(gordon_gekko_core::config::load_config(&args.config)?);
    info!("✅ Configuration loaded successfully");

    // Create trading system
    let mut trading_system = TradingSystem::new(config.clone(), args.sandbox, args.gpu).await?;
    info!("✅ Trading system initialized");

    // Setup graceful shutdown
    let shutdown_handle = setup_shutdown_handler();

    // Start the trading system
    info!("🎯 Starting trading operations...");
    let result = trading_system.run().await;

    // Wait for shutdown signal
    shutdown_handle.await;

    // Perform cleanup
    info!("🛑 Shutting down trading system...");
    trading_system.shutdown().await;
    info!("✅ Trading system shut down gracefully");

    match result {
        Ok(_) => {
            info!("✨ Trading session completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("💥 Trading session failed: {:?}", e);
            Err(e)
        }
    }
}

/// Initialize tracing subscriber based on log level
fn init_tracing(log_level: &str) -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(match log_level.to_lowercase().as_str() {
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => {
                warn!("Invalid log level '{}', defaulting to 'info'", log_level);
                tracing::Level::INFO
            }
        })
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

/// Setup graceful shutdown handler
async fn setup_shutdown_handler() {
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("📡 Received shutdown signal (Ctrl+C)");
        }
        Err(err) => {
            error!("💥 Failed to listen for shutdown signal: {:?}", err);
        }
    }
}