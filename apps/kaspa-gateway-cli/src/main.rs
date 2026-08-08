use clap::{Parser, Subcommand};
use kaspa_gateway_api::{ApiClientConfig, KaspaApiClient};
use kaspa_gateway_config::{default_config_path, default_user_data_dir};
use kaspa_gateway_core::{AppInfo, KaspaAddress};
use kaspa_gateway_db::{AddressRecord, DatabaseManager, DatabasePaths};
use kaspa_gateway_node::{
    MiningConnectivityMode, NodeCapabilityManager, NodeEndpoint, NodeManagerConfig,
};
use kaspa_gateway_runtime::runtime_check_default;
use kaspa_gateway_security::redact_url;
#[derive(Debug, Parser)]
#[command(name = "kaspa-gateway")]
#[command(about = "Kaspa Gateway Rust development CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Info,
    Architecture,
    ConfigPath,
    RuntimeCheck,
    DbPaths,
    DbInit,
    DbAddAddress {
        address: String,
        name: String,
    },
    DbListAddresses,
    ValidateAddress {
        address: String,
    },
    RedactUrl {
        url: String,
    },
    ApiBalanceUrl {
        address: String,
    },
    ApiNetworkUrl,
    NodeCapabilities,
    NodeOwnerKind,
    NodePlan {
        #[arg(long, default_value = "mainnet")]
        network: String,
        #[arg(long, default_value = "127.0.0.1")]
        rpc_host: String,
        #[arg(long, default_value_t = 16110)]
        rpc_port: u16,
    },
    NodeOwnerCheck,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => {
            let info = AppInfo::default();
            println!("{} {}", info.name, info.version);
        }
        Commands::Architecture => {
            println!("Kaspa Gateway Rust workspace is ready.");
            println!(
                "Core, config, security, API, DB, node, runtime, and observability crates are separated."
            );
            println!(
                "Bridge design note: mining connectivity is modeled as a capability, not a fixed external process."
            );
        }
        Commands::ConfigPath => match default_config_path() {
            Ok(path) => println!("{}", path.display()),
            Err(error) => {
                eprintln!("Failed to resolve config path: {error}");
                std::process::exit(1);
            }
        },
        Commands::RuntimeCheck => match runtime_check_default() {
            Ok(report) => {
                for line in report.to_lines() {
                    println!("{line}");
                }
            }
            Err(error) => {
                eprintln!("runtime check failed: {error}");
                std::process::exit(1);
            }
        },
        Commands::DbPaths => {
            let paths = match database_paths_from_default_user_dir() {
                Ok(paths) => paths,
                Err(error) => {
                    eprintln!("failed to resolve DB paths: {error}");
                    std::process::exit(1);
                }
            };

            println!("root={}", paths.root.display());
            println!("app_data={}", paths.app_data.display());
            println!("addresses={}", paths.addresses.display());
            println!("transactions={}", paths.transactions.display());
        }
        Commands::DbInit => {
            let manager = match database_manager_from_default_user_dir() {
                Ok(manager) => manager,
                Err(error) => {
                    eprintln!("failed to create DB manager: {error}");
                    std::process::exit(1);
                }
            };

            if let Err(error) = manager.initialize_all() {
                eprintln!("failed to initialize databases: {error}");
                std::process::exit(1);
            }

            println!("databases initialized");
            println!("root={}", manager.paths().root.display());
        }
        Commands::DbAddAddress { address, name } => {
            if let Err(error) = KaspaAddress::parse(&address) {
                eprintln!("invalid address: {error}");
                std::process::exit(1);
            }

            let manager = match database_manager_from_default_user_dir() {
                Ok(manager) => manager,
                Err(error) => {
                    eprintln!("failed to create DB manager: {error}");
                    std::process::exit(1);
                }
            };

            let repository = match manager.addresses_repository() {
                Ok(repository) => repository,
                Err(error) => {
                    eprintln!("failed to open addresses repository: {error}");
                    std::process::exit(1);
                }
            };

            let record = match AddressRecord::new(address, name, "mainnet") {
                Ok(record) => record,
                Err(error) => {
                    eprintln!("invalid address record: {error}");
                    std::process::exit(1);
                }
            };

            if let Err(error) = repository.upsert(&record) {
                eprintln!("failed to save address: {error}");
                std::process::exit(1);
            }

            println!("address saved");
        }
        Commands::DbListAddresses => {
            let manager = match database_manager_from_default_user_dir() {
                Ok(manager) => manager,
                Err(error) => {
                    eprintln!("failed to create DB manager: {error}");
                    std::process::exit(1);
                }
            };

            let repository = match manager.addresses_repository() {
                Ok(repository) => repository,
                Err(error) => {
                    eprintln!("failed to open addresses repository: {error}");
                    std::process::exit(1);
                }
            };

            match repository.list() {
                Ok(records) => {
                    for record in records {
                        println!("{} | {} | {}", record.address, record.name, record.network);
                    }
                }
                Err(error) => {
                    eprintln!("failed to list addresses: {error}");
                    std::process::exit(1);
                }
            }
        }
        Commands::ValidateAddress { address } => match KaspaAddress::parse(address) {
            Ok(address) => println!("valid: {}", address.masked()),
            Err(error) => {
                eprintln!("invalid: {error}");
                std::process::exit(1);
            }
        },
        Commands::RedactUrl { url } => {
            println!("{}", redact_url(&url));
        }
        Commands::ApiBalanceUrl { address } => {
            if let Err(error) = KaspaAddress::parse(&address) {
                eprintln!("invalid address: {error}");
                std::process::exit(1);
            }

            let client = match KaspaApiClient::new(ApiClientConfig::default()) {
                Ok(client) => client,
                Err(error) => {
                    eprintln!("failed to create API client: {error}");
                    std::process::exit(1);
                }
            };

            match client.address_balance_url(&address) {
                Ok(url) => println!("{url}"),
                Err(error) => {
                    eprintln!("failed to build balance URL: {error}");
                    std::process::exit(1);
                }
            }
        }
        Commands::ApiNetworkUrl => {
            let client = match KaspaApiClient::new(ApiClientConfig::default()) {
                Ok(client) => client,
                Err(error) => {
                    eprintln!("failed to create API client: {error}");
                    std::process::exit(1);
                }
            };

            match client.network_info_url() {
                Ok(url) => println!("{url}"),
                Err(error) => {
                    eprintln!("failed to build network URL: {error}");
                    std::process::exit(1);
                }
            }
        }
        Commands::NodeCapabilities => {
            let config = NodeManagerConfig::default();

            match NodeCapabilityManager::inspect(&config) {
                Ok(capabilities) => println!("{}", capabilities.describe()),
                Err(error) => {
                    eprintln!("failed to inspect node capabilities: {error}");
                    std::process::exit(1);
                }
            }
        }
        Commands::NodeOwnerKind => {
            println!("kgw-owner");
        }
        Commands::NodePlan {
            network,
            rpc_host,
            rpc_port,
        } => {
            let rpc_endpoint = match NodeEndpoint::new(rpc_host, rpc_port, false) {
                Ok(endpoint) => endpoint,
                Err(error) => {
                    eprintln!("invalid RPC endpoint: {error}");
                    std::process::exit(1);
                }
            };

            let config = NodeManagerConfig {
                network,
                rpc_endpoint,
                mining_mode: MiningConnectivityMode::IntegratedBridge,
                mining_endpoint: None,
                extra_args: Vec::new(),
            };

            match NodeCapabilityManager::build_launch_plan(&config) {
                Ok(plan) => println!("{}", plan.command_preview()),
                Err(error) => {
                    eprintln!("failed to build KGW owner plan: {error}");
                    std::process::exit(1);
                }
            }
        }
        Commands::NodeOwnerCheck => {
            let config = NodeManagerConfig::default();

            match NodeCapabilityManager::inspect(&config) {
                Ok(capabilities) => {
                    println!("owner=kgw-service-controller");
                    println!("{}", capabilities.describe());
                }
                Err(error) => {
                    eprintln!("failed to inspect KGW owner capabilities: {error}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn database_manager_from_default_user_dir() -> Result<DatabaseManager, Box<dyn std::error::Error>> {
    let paths = database_paths_from_default_user_dir()?;
    Ok(DatabaseManager::new(paths))
}

fn database_paths_from_default_user_dir() -> Result<DatabasePaths, Box<dyn std::error::Error>> {
    let root = default_user_data_dir()?.join("databases");
    Ok(DatabasePaths::new(root)?)
}
