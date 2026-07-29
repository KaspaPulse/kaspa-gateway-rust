use kaspa_grpc_client_live::GrpcClient;
use kaspa_rpc_core_live::api::rpc::RpcApi;
use serde_json::json;

fn argument_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].clone())
}

fn normalized_network(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let endpoint =
        argument_value(&args, "--rpc").unwrap_or_else(|| "grpc://127.0.0.1:16110".to_string());
    let expected_network =
        argument_value(&args, "--expect-network").unwrap_or_else(|| "mainnet".to_string());

    let result = probe(&endpoint, &expected_network).await;

    match result {
        Ok(report) => println!("{report}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

async fn probe(endpoint: &str, expected_network: &str) -> Result<String, String> {
    let client = GrpcClient::connect(endpoint.to_string())
        .await
        .map_err(|error| format!("RPC connect failed for {endpoint}: {error}"))?;

    let server = client
        .get_server_info()
        .await
        .map_err(|error| format!("get_server_info failed: {error}"))?;
    let dag = client
        .get_block_dag_info()
        .await
        .map_err(|error| format!("get_block_dag_info failed: {error}"))?;
    let peers = client
        .get_connected_peer_info()
        .await
        .map_err(|error| format!("get_connected_peer_info failed: {error}"))?;
    let sync = client
        .get_sync_status()
        .await
        .map_err(|error| format!("get_sync_status failed: {error}"))?;

    client
        .disconnect()
        .await
        .map_err(|error| format!("RPC disconnect failed: {error}"))?;

    let actual_network = server.network_id.to_string();
    if normalized_network(&actual_network) != normalized_network(expected_network) {
        return Err(format!(
            "network mismatch: expected={expected_network};actual={actual_network}"
        ));
    }

    Ok(json!({
        "endpoint": endpoint,
        "network": actual_network,
        "serverVersion": server.server_version,
        "isSynced": sync,
        "hasUtxoIndex": server.has_utxo_index,
        "peerCount": peers.peer_info.len(),
        "blockCount": dag.block_count,
        "headerCount": dag.header_count,
        "virtualDaaScore": dag.virtual_daa_score,
        "tipCount": dag.tip_hashes.len()
    })
    .to_string())
}
