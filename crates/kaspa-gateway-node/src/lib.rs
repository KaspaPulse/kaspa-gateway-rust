use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),

    #[error("invalid network: {0}")]
    InvalidNetwork(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, NodeError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeBinaryKind {
    KgwOwner,
    Unknown,
}

impl fmt::Display for NodeBinaryKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KgwOwner => formatter.write_str("kgw-owner"),
            Self::Unknown => formatter.write_str("unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiningConnectivityMode {
    IntegratedBridge,
    RemoteEndpoint,
    Disabled,
}

impl fmt::Display for MiningConnectivityMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntegratedBridge => formatter.write_str("integrated-bridge"),
            Self::RemoteEndpoint => formatter.write_str("remote-endpoint"),
            Self::Disabled => formatter.write_str("disabled"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeEndpoint {
    pub host: String,
    pub port: u16,
    pub tls: bool,
}

impl NodeEndpoint {
    pub fn new(host: impl Into<String>, port: u16, tls: bool) -> Result<Self> {
        let host = host.into().trim().to_string();
        validate_host(&host)?;

        if port == 0 {
            return Err(NodeError::InvalidEndpoint(
                "port cannot be zero".to_string(),
            ));
        }

        Ok(Self { host, port, tls })
    }

    pub fn display_address(&self) -> String {
        let scheme = if self.tls { "wss" } else { "ws" };
        format!("{scheme}://{}:{}", self.host, self.port)
    }

    pub fn socket_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn cli_value(&self) -> String {
        self.socket_address()
    }

    pub fn validate(&self) -> Result<()> {
        validate_host(&self.host)?;

        if self.port == 0 {
            return Err(NodeError::InvalidEndpoint(
                "port cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for NodeEndpoint {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 16110,
            tls: false,
        }
    }
}

impl fmt::Display for NodeEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_address())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub node_available: bool,
    pub rpc_available: bool,
    pub mining_endpoint_available: bool,
    pub mining_mode: MiningConnectivityMode,
    pub binary_kind: NodeBinaryKind,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            node_available: true,
            rpc_available: true,
            mining_endpoint_available: true,
            mining_mode: MiningConnectivityMode::IntegratedBridge,
            binary_kind: NodeBinaryKind::KgwOwner,
        }
    }
}

impl NodeCapabilities {
    pub fn describe(&self) -> String {
        format!(
            "node_available={}, rpc_available={}, mining_endpoint_available={}, mining_mode={}, binary_kind={}, runtime_owner=kgw-service-controller",
            self.node_available,
            self.rpc_available,
            self.mining_endpoint_available,
            self.mining_mode,
            self.binary_kind
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeManagerConfig {
    pub network: String,
    pub rpc_endpoint: NodeEndpoint,
    pub mining_mode: MiningConnectivityMode,
    pub mining_endpoint: Option<NodeEndpoint>,
    pub extra_args: Vec<String>,
}

impl Default for NodeManagerConfig {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            rpc_endpoint: NodeEndpoint::default(),
            mining_mode: MiningConnectivityMode::IntegratedBridge,
            mining_endpoint: None,
            extra_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeLaunchPlan {
    pub network: String,
    pub rpc_endpoint: NodeEndpoint,
    pub mining_mode: MiningConnectivityMode,
    pub mining_endpoint: Option<NodeEndpoint>,
    pub args: Vec<String>,
}

impl NodeLaunchPlan {
    pub fn validate(&self) -> Result<()> {
        validate_network(&self.network)?;
        self.rpc_endpoint.validate()?;

        if let Some(endpoint) = &self.mining_endpoint {
            endpoint.validate()?;
        }

        for arg in &self.args {
            validate_owner_arg(arg)?;
        }

        Ok(())
    }

    pub fn command_preview(&self) -> String {
        let mut parts = vec!["kgw-owner:kaspad".to_string()];
        parts.extend(self.args.clone());
        parts.join(" ")
    }

    pub fn safe_args(&self) -> &[String] {
        &self.args
    }
}

pub struct NodeCapabilityManager;

impl NodeCapabilityManager {
    pub fn inspect(config: &NodeManagerConfig) -> Result<NodeCapabilities> {
        validate_network(&config.network)?;
        config.rpc_endpoint.validate()?;

        if let Some(endpoint) = &config.mining_endpoint {
            endpoint.validate()?;
        }

        for arg in &config.extra_args {
            validate_owner_arg(arg)?;
        }

        Ok(NodeCapabilities::default())
    }

    pub fn build_launch_plan(config: &NodeManagerConfig) -> Result<NodeLaunchPlan> {
        Self::inspect(config)?;

        let network = normalize_network(&config.network);
        let mut args = Vec::new();

        match network.as_str() {
            "testnet10" => {
                args.push("--testnet".to_string());
                args.push("--netsuffix=10".to_string());
            }
            "testnet12" => {
                args.push("--testnet".to_string());
                args.push("--netsuffix=12".to_string());
            }
            _ => {}
        }

        args.push("--utxoindex".to_string());
        args.push(format!(
            "--rpclisten={}",
            config.rpc_endpoint.socket_address()
        ));
        args.push(format!("--appdir=kaspa-gateway-{network}"));

        for arg in &config.extra_args {
            args.push(arg.clone());
        }

        Ok(NodeLaunchPlan {
            network,
            rpc_endpoint: config.rpc_endpoint.clone(),
            mining_mode: config.mining_mode,
            mining_endpoint: config.mining_endpoint.clone(),
            args,
        })
    }
}

pub fn infer_binary_kind(_value: impl AsRef<str>) -> NodeBinaryKind {
    NodeBinaryKind::KgwOwner
}

pub fn normalize_network(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "mainnet" => "mainnet".to_string(),
        "testnet" | "testnet10" | "testnet-10" => "testnet10".to_string(),
        "testnet12" | "tn12" | "testnet-12" => "testnet12".to_string(),
        "simnet" => "simnet".to_string(),
        "devnet" => "devnet".to_string(),
        _ => "mainnet".to_string(),
    }
}

pub fn branch_for_network(value: &str) -> &'static str {
    match normalize_network(value).as_str() {
        "testnet12" => "tn12",
        _ => "master",
    }
}

fn validate_network(value: &str) -> Result<()> {
    match value.trim().to_ascii_lowercase().as_str() {
        "mainnet" | "testnet" | "testnet10" | "testnet-10" | "testnet12" | "tn12"
        | "testnet-12" | "simnet" | "devnet" => Ok(()),
        other => Err(NodeError::InvalidNetwork(other.to_string())),
    }
}

fn validate_host(value: &str) -> Result<()> {
    let trimmed = value.trim();

    if trimmed.is_empty()
        || value.chars().any(char::is_whitespace)
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('\0')
        || trimmed.contains("..")
    {
        return Err(NodeError::InvalidEndpoint(value.to_string()));
    }

    Ok(())
}

fn validate_owner_arg(value: &str) -> Result<()> {
    if value.contains('\0')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains(';')
        || value.contains('|')
    {
        return Err(NodeError::InvalidArgument(value.to_string()));
    }

    Ok(())
}
