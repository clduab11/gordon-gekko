//! Swarm intelligence integration for Ninja Gekko

/// Swarm intelligence manager for distributed decision making
#[derive(Debug)]
pub struct SwarmIntelligence {
    /// Swarm topology type
    topology: SwarmTopology,
    /// Active agents in the swarm
    agents: Vec<SwarmAgent>,
}

/// Swarm topology types
#[derive(Debug, Clone, Copy)]
pub enum SwarmTopology {
    /// Mesh topology - all agents connected
    Mesh,
    /// Star topology - central coordinator
    Star,
    /// Ring topology - circular connections
    Ring,
    /// Hierarchical topology - tree structure
    Hierarchical,
}

/// Individual swarm agent
#[derive(Debug, Clone)]
pub struct SwarmAgent {
    /// Agent identifier
    pub id: String,
    /// Agent role/specialization
    pub role: AgentRole,
    /// Agent status
    pub status: AgentStatus,
}

/// Agent roles in the swarm
#[derive(Debug, Clone, Copy)]
pub enum AgentRole {
    /// Data collector agent
    Collector,
    /// Analysis agent
    Analyzer,
    /// Trading execution agent
    Executor,
    /// Risk monitoring agent
    RiskMonitor,
    /// Coordinator agent
    Coordinator,
}

/// Agent status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is active and working
    Active,
    /// Agent is idle
    Idle,
    /// Agent is busy processing
    Busy,
    /// Agent has failed
    Failed,
}

impl SwarmIntelligence {
    /// Create a new swarm intelligence system
    pub fn new(topology: SwarmTopology) -> Self {
        SwarmIntelligence {
            topology,
            agents: vec![],
        }
    }
}
