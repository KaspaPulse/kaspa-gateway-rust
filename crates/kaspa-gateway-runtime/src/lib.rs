use kaspa_gateway_api::{ApiClientConfig, KaspaApiClient};
use kaspa_gateway_config::{
    default_config_path, default_user_data_dir, load_or_create_config, GatewayConfig,
};
use kaspa_gateway_core::AppInfo;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use kaspa_gateway_node::{NodeCapabilities, NodeCapabilityManager, NodeManagerConfig};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("config error: {0}")]
    Config(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("api error: {0}")]
    Api(String),

    #[error("node error: {0}")]
    Node(String),

    #[error("runtime error: {0}")]
    Runtime(String),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimePaths {
    pub config_path: PathBuf,
    pub database_root: PathBuf,
}

impl RuntimePaths {
    pub fn default_paths() -> Result<Self> {
        let config_path =
            default_config_path().map_err(|error| RuntimeError::Config(error.to_string()))?;

        let database_root = default_user_data_dir()
            .map_err(|error| RuntimeError::Config(error.to_string()))?
            .join("databases");

        Ok(Self {
            config_path,
            database_root,
        })
    }

    pub fn new(config_path: impl AsRef<Path>, database_root: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            database_root: database_root.as_ref().to_path_buf(),
        }
    }

    pub fn from_root(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();

        Self {
            config_path: root.join("config.json"),
            database_root: root.join("databases"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeHealthReport {
    pub app_name: String,
    pub app_version: String,
    pub config_loaded: bool,
    pub database_initialized: bool,
    pub api_client_ready: bool,
    pub api_network_url: String,
    pub node_capabilities: String,
}

impl RuntimeHealthReport {
    pub fn is_healthy(&self) -> bool {
        self.config_loaded && self.database_initialized && self.api_client_ready
    }

    pub fn to_lines(&self) -> Vec<String> {
        vec![
            format!("app={} {}", self.app_name, self.app_version),
            format!("config_loaded={}", self.config_loaded),
            format!("database_initialized={}", self.database_initialized),
            format!("api_client_ready={}", self.api_client_ready),
            format!("api_network_url={}", self.api_network_url),
            format!("node_capabilities={}", self.node_capabilities),
            format!("healthy={}", self.is_healthy()),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReadinessReport {
    pub paths: RuntimePaths,
    pub config_path_exists: bool,
    pub database_root_exists: bool,
    pub app_data_db_exists: bool,
    pub addresses_db_exists: bool,
    pub transactions_db_exists: bool,
    pub config_valid: bool,
    pub database_initialized: bool,
    pub api_config_valid: bool,
    pub node_config_valid: bool,
    pub warnings: Vec<String>,
}

impl RuntimeReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.config_valid
            && self.database_initialized
            && self.api_config_valid
            && self.node_config_valid
    }

    pub fn to_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("config_path={}", self.paths.config_path.display()),
            format!("database_root={}", self.paths.database_root.display()),
            format!("config_path_exists={}", self.config_path_exists),
            format!("database_root_exists={}", self.database_root_exists),
            format!("app_data_db_exists={}", self.app_data_db_exists),
            format!("addresses_db_exists={}", self.addresses_db_exists),
            format!("transactions_db_exists={}", self.transactions_db_exists),
            format!("config_valid={}", self.config_valid),
            format!("database_initialized={}", self.database_initialized),
            format!("api_config_valid={}", self.api_config_valid),
            format!("node_config_valid={}", self.node_config_valid),
            format!("ready={}", self.is_ready()),
        ];

        for warning in &self.warnings {
            lines.push(format!("warning={warning}"));
        }

        lines
    }
}

#[derive(Debug)]
pub struct AppRuntime {
    app_info: AppInfo,
    paths: RuntimePaths,
    config: GatewayConfig,
    database_manager: DatabaseManager,
    api_client: KaspaApiClient,
    node_config: NodeManagerConfig,
}

impl AppRuntime {
    pub fn initialize_default() -> Result<Self> {
        let paths = RuntimePaths::default_paths()?;
        Self::initialize_from_paths(paths)
    }

    pub fn initialize_from_paths(paths: RuntimePaths) -> Result<Self> {
        let app_info = AppInfo::default();

        let config = load_or_create_config(&paths.config_path)
            .map_err(|error| RuntimeError::Config(error.to_string()))?;

        config
            .validate()
            .map_err(|error| RuntimeError::Config(error.to_string()))?;

        let database_paths = DatabasePaths::new(&paths.database_root)
            .map_err(|error| RuntimeError::Database(error.to_string()))?;

        let database_manager = DatabaseManager::new(database_paths);

        database_manager
            .initialize_all()
            .map_err(|error| RuntimeError::Database(error.to_string()))?;

        let active_profile = config.active_api_profile();

        let api_config = ApiClientConfig::new(&active_profile.base_url)
            .map_err(|error| RuntimeError::Api(error.to_string()))?
            .with_retry_attempts(config.performance.retry_attempts)
            .map_err(|error| RuntimeError::Api(error.to_string()))?
            .with_supported_currencies(config.display.supported_currencies.clone())
            .map_err(|error| RuntimeError::Api(error.to_string()))?;

        let api_client = KaspaApiClient::new(api_config)
            .map_err(|error| RuntimeError::Api(error.to_string()))?;

        let node_config = NodeManagerConfig::default();

        NodeCapabilityManager::inspect(&node_config)
            .map_err(|error| RuntimeError::Node(error.to_string()))?;

        Ok(Self {
            app_info,
            paths,
            config,
            database_manager,
            api_client,
            node_config,
        })
    }

    pub fn app_info(&self) -> &AppInfo {
        &self.app_info
    }

    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    pub fn paths(&self) -> &RuntimePaths {
        &self.paths
    }

    pub fn database_manager(&self) -> &DatabaseManager {
        &self.database_manager
    }

    pub fn api_client(&self) -> &KaspaApiClient {
        &self.api_client
    }

    pub fn node_config(&self) -> &NodeManagerConfig {
        &self.node_config
    }

    pub fn health_report(&self) -> Result<RuntimeHealthReport> {
        let api_network_url = self
            .api_client
            .network_info_url()
            .map_err(|error| RuntimeError::Api(error.to_string()))?
            .to_string();

        let node_capabilities: NodeCapabilities = NodeCapabilityManager::inspect(&self.node_config)
            .map_err(|error| RuntimeError::Node(error.to_string()))?;

        Ok(RuntimeHealthReport {
            app_name: self.app_info.name.clone(),
            app_version: self.app_info.version.clone(),
            config_loaded: true,
            database_initialized: true,
            api_client_ready: true,
            api_network_url,
            node_capabilities: node_capabilities.describe(),
        })
    }

    pub fn readiness_report(&self) -> RuntimeReadinessReport {
        runtime_readiness_from_paths(self.paths.clone())
    }
}

pub fn runtime_check_default() -> Result<RuntimeHealthReport> {
    let runtime = AppRuntime::initialize_default()?;
    runtime.health_report()
}

pub fn runtime_check_from_paths(paths: RuntimePaths) -> Result<RuntimeHealthReport> {
    let runtime = AppRuntime::initialize_from_paths(paths)?;
    runtime.health_report()
}

pub fn runtime_readiness_default() -> Result<RuntimeReadinessReport> {
    let paths = RuntimePaths::default_paths()?;
    Ok(runtime_readiness_from_paths(paths))
}

pub fn runtime_readiness_from_paths(paths: RuntimePaths) -> RuntimeReadinessReport {
    let mut warnings = Vec::new();

    let config_path_exists = paths.config_path.exists();
    let database_root_exists = paths.database_root.exists();

    let db_paths = match DatabasePaths::new(&paths.database_root) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(format!("database paths invalid: {error}"));

            return RuntimeReadinessReport {
                paths,
                config_path_exists,
                database_root_exists,
                app_data_db_exists: false,
                addresses_db_exists: false,
                transactions_db_exists: false,
                config_valid: false,
                database_initialized: false,
                api_config_valid: false,
                node_config_valid: false,
                warnings,
            };
        }
    };

    let app_data_db_exists = db_paths.app_data.exists();
    let addresses_db_exists = db_paths.addresses.exists();
    let transactions_db_exists = db_paths.transactions.exists();

    let config_valid = match load_or_create_config(&paths.config_path) {
        Ok(config) => {
            if let Err(error) = config.validate() {
                warnings.push(format!("config validation failed: {error}"));
                false
            } else {
                true
            }
        }
        Err(error) => {
            warnings.push(format!("config load failed: {error}"));
            false
        }
    };

    let database_initialized = {
        let manager = DatabaseManager::new(db_paths);

        match manager.initialize_all() {
            Ok(()) => true,
            Err(error) => {
                warnings.push(format!("database initialization failed: {error}"));
                false
            }
        }
    };

    let api_config_valid = match load_or_create_config(&paths.config_path) {
        Ok(config) => {
            let active_profile = config.active_api_profile();

            match ApiClientConfig::new(&active_profile.base_url) {
                Ok(api_config) => KaspaApiClient::new(api_config).is_ok(),
                Err(error) => {
                    warnings.push(format!("api config invalid: {error}"));
                    false
                }
            }
        }
        Err(_) => false,
    };

    let node_config_valid = match NodeCapabilityManager::inspect(&NodeManagerConfig::default()) {
        Ok(_) => true,
        Err(error) => {
            warnings.push(format!("node config invalid: {error}"));
            false
        }
    };

    RuntimeReadinessReport {
        paths,
        config_path_exists,
        database_root_exists,
        app_data_db_exists,
        addresses_db_exists,
        transactions_db_exists,
        config_valid,
        database_initialized,
        api_config_valid,
        node_config_valid,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_runtime_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time valid")
            .as_nanos();

        std::env::temp_dir().join(format!("kaspa_gateway_runtime_{stamp}"))
    }

    #[test]
    fn runtime_paths_from_root_are_stable() {
        let root = unique_runtime_root();
        let paths = RuntimePaths::from_root(&root);

        assert_eq!(paths.config_path, root.join("config.json"));
        assert_eq!(paths.database_root, root.join("databases"));
    }

    #[test]
    fn runtime_initializes_from_temp_paths() {
        let root = unique_runtime_root();
        let paths = RuntimePaths::from_root(&root);

        let runtime = AppRuntime::initialize_from_paths(paths.clone()).expect("runtime");

        assert!(runtime.config().validate().is_ok());
        assert!(runtime.paths().config_path.exists());
        assert!(runtime.paths().database_root.exists());

        let report = runtime.health_report().expect("health");
        assert!(report.is_healthy());
        assert!(report.api_network_url.contains("/info/network"));
    }

    #[test]
    fn readiness_report_initializes_required_files() {
        let root = unique_runtime_root();
        let paths = RuntimePaths::from_root(&root);

        let report = runtime_readiness_from_paths(paths);

        assert!(report.is_ready());
        assert!(report.config_valid);
        assert!(report.database_initialized);
        assert!(report.api_config_valid);
        assert!(report.node_config_valid);
    }
}

pub mod transaction_sync;
