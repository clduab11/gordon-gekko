//! Core Ninja Gekko system implementation

use std::fmt;
use crate::mcp::McpManager;
use crate::neural::NeuralBackend;

/// Main Ninja Gekko bot struct
#[derive(Debug)]
pub struct NinjaGekko {
    /// Operation mode
    pub mode: OperationMode,
    /// Neural network backend
    pub neural_backend: NeuralBackend,
    /// MCP manager for protocol integrations
    pub mcp_manager: McpManager,
    /// Dry run flag
    pub dry_run: bool,
}

/// Operation modes for the trading bot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Stealth mode - minimal market impact
    Stealth,
    /// Precision mode - microsecond timing
    Precision,
    /// Swarm mode - distributed intelligence
    Swarm,
}

impl fmt::Display for OperationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationMode::Stealth => write!(f, "Stealth"),
            OperationMode::Precision => write!(f, "Precision"),
            OperationMode::Swarm => write!(f, "Swarm"),
        }
    }
}

/// Builder for NinjaGekko
pub struct NinjaGekkoBuilder {
    mode: OperationMode,
    neural_backend: NeuralBackend,
    mcp_servers: Vec<String>,
    dry_run: bool,
}

impl NinjaGekko {
    /// Create a new builder
    pub fn builder() -> NinjaGekkoBuilder {
        NinjaGekkoBuilder {
            mode: OperationMode::Precision,
            neural_backend: NeuralBackend::RuvFann,
            mcp_servers: vec![],
            dry_run: false,
        }
    }
    
    /// Start the bot
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🥷 Starting Ninja Gekko in {:?} mode", self.mode);
        
        // Initialize components based on mode
        match self.mode {
            OperationMode::Stealth => self.start_stealth_mode().await?,
            OperationMode::Precision => self.start_precision_mode().await?,
            OperationMode::Swarm => self.start_swarm_mode().await?,
        }
        
        Ok(())
    }
    
    async fn start_stealth_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🌙 Initializing stealth operations...");
        // TODO: Implement stealth mode logic
        Ok(())
    }
    
    async fn start_precision_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("⚡ Initializing precision operations...");
        // TODO: Implement precision mode logic
        Ok(())
    }
    
    async fn start_swarm_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🤖 Initializing swarm operations...");
        // TODO: Implement swarm mode logic
        Ok(())
    }
}

impl NinjaGekkoBuilder {
    /// Set operation mode
    pub fn mode(mut self, mode: OperationMode) -> Self {
        self.mode = mode;
        self
    }
    
    /// Set neural backend
    pub fn neural_backend(mut self, backend: NeuralBackend) -> Self {
        self.neural_backend = backend;
        self
    }
    
    /// Set MCP servers
    pub fn mcp_servers(mut self, servers: Vec<String>) -> Self {
        self.mcp_servers = servers;
        self
    }
    
    /// Set dry run mode
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
    
    /// Build the NinjaGekko instance
    pub async fn build(self) -> Result<NinjaGekko, Box<dyn std::error::Error>> {
        let mcp_manager = McpManager::new(self.mcp_servers).await?;
        
        Ok(NinjaGekko {
            mode: self.mode,
            neural_backend: self.neural_backend,
            mcp_manager,
            dry_run: self.dry_run,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operation_mode_display() {
        assert_eq!(OperationMode::Stealth.to_string(), "Stealth");
        assert_eq!(OperationMode::Precision.to_string(), "Precision");
        assert_eq!(OperationMode::Swarm.to_string(), "Swarm");
    }
    
    #[tokio::test]
    async fn test_builder_pattern() {
        let result = NinjaGekko::builder()
            .mode(OperationMode::Stealth)
            .neural_backend(NeuralBackend::RuvFann)
            .mcp_servers(vec!["test".to_string()])
            .dry_run(true)
            .build()
            .await;
        
        assert!(result.is_ok());
        let bot = result.unwrap();
        assert_eq!(bot.mode, OperationMode::Stealth);
        assert!(bot.dry_run);
    }
}