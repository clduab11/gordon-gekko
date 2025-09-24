//! Ninja Gekko - Autonomous Trading Bot CLI
//!
//! This is the main entry point for the Ninja Gekko autonomous trading bot.

use clap::{Arg, Command};
use ninja_gekko::prelude::*;
use std::process;
use tracing::{info, error};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("ninja-gekko")
        .version(ninja_gekko::VERSION)
        .about("Next-Generation Rust-Powered Autonomous Trading Bot")
        .long_about(ninja_gekko::BUILD_INFO)
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config.toml"),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Operation mode")
                .value_parser(["stealth", "precision", "swarm"])
                .default_value("precision"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Run in simulation mode without real trades")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("mcp-servers")
                .long("mcp-servers")
                .value_name("SERVERS")
                .help("Comma-separated list of MCP servers to enable")
                .default_value("playwright,filesystem,github,supabase"),
        )
        .get_matches();

    // Parse configuration
    let config_path = matches.get_one::<String>("config").unwrap();
    let operation_mode = match matches.get_one::<String>("mode").unwrap().as_str() {
        "stealth" => OperationMode::Stealth,
        "precision" => OperationMode::Precision, 
        "swarm" => OperationMode::Swarm,
        _ => OperationMode::Precision,
    };
    let dry_run = matches.get_flag("dry-run");
    let mcp_servers: Vec<&str> = matches
        .get_one::<String>("mcp-servers")
        .unwrap()
        .split(',')
        .collect();

    info!("🥷 Starting Ninja Gekko v{}", ninja_gekko::VERSION);
    info!("📋 Config: {}", config_path);
    info!("🎯 Mode: {:?}", operation_mode);
    info!("🎭 MCP Servers: {:?}", mcp_servers);
    
    if dry_run {
        info!("🏃 Running in DRY RUN mode - no real trades will be executed");
    }

    // Initialize and start the bot
    match run_bot(config_path, operation_mode, mcp_servers, dry_run).await {
        Ok(_) => {
            info!("✅ Ninja Gekko completed successfully");
        }
        Err(e) => {
            error!("❌ Ninja Gekko failed: {}", e);
            process::exit(1);
        }
    }
}

async fn run_bot(
    _config_path: &str,
    operation_mode: OperationMode,
    mcp_servers: Vec<&str>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Initializing Ninja Gekko...");

    // This is a placeholder implementation for the Rust version
    // The actual implementation will be built as part of the migration
    
    let _bot = NinjaGekko::builder()
        .mode(operation_mode)
        .neural_backend(NeuralBackend::RuvFann)
        .mcp_servers(mcp_servers.into_iter().map(String::from).collect())
        .dry_run(dry_run)
        .build()
        .await?;

    info!("🚀 Ninja Gekko initialized successfully");
    info!("🎯 Operating in {:?} mode", operation_mode);
    
    // Start the main trading loop
    info!("🔄 Starting autonomous trading operations...");
    
    // For now, just run for a short time to demonstrate
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    info!("🛑 Ninja Gekko shutting down gracefully");
    
    Ok(())
}