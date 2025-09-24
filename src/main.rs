//! Ninja Gekko - Autonomous Trading Bot CLI
//!
//! This is the main entry point for the Ninja Gekko autonomous trading bot.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,

    /// Operation mode
    #[arg(short, long, default_value = "precision")]
    mode: String,

    /// Enable sandbox mode (no real trading)
    #[arg(long)]
    sandbox: bool,

    /// Log level (debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Enable GPU acceleration for neural networks
    #[arg(long)]
    gpu: bool,

    /// MCP servers to enable
    #[arg(long, default_value = "playwright,filesystem,github,supabase")]
    mcp_servers: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing subscriber
    init_tracing(&args.log_level)?;

    info!("🥷 Starting Ninja Gekko v{}", env!("CARGO_PKG_VERSION"));
    info!("📊 Configuration: {}", args.config);
    info!("🎯 Operation mode: {}", args.mode);
    info!("🏖️  Sandbox mode: {}", args.sandbox);
    info!("🔥 GPU acceleration: {}", args.gpu);
    info!("🎭 MCP servers: {}", args.mcp_servers);

    // Load configuration - using placeholder for now
    info!("✅ Configuration loaded successfully");

    // Create trading system - placeholder implementation
    info!("✅ Ninja Gekko initialized");

    // Setup graceful shutdown
    let shutdown_handle = setup_shutdown_handler();

    // Start the trading system
    info!("🎯 Starting autonomous trading operations...");

    // Simulate running for a short time (placeholder)
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Wait for shutdown signal or completion
    shutdown_handle.await;

    // Perform cleanup
    info!("🛑 Shutting down Ninja Gekko...");
    info!("✅ Ninja Gekko shut down gracefully");

    Ok(())
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