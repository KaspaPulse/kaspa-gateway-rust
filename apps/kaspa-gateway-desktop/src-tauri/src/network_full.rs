use kaspa_gateway_api::{ApiClientConfig, KaspaApiClient};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct NetworkMetricCard {
    pub key: String,
    pub title: String,
    pub value: String,
    pub unit: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullNetworkAnalyticsReport {
    pub cards: Vec<NetworkMetricCard>,
    pub hashrate: Option<Value>,
    pub max_hashrate: Option<Value>,
    pub network: Option<Value>,
    pub blockdag: Option<Value>,
    pub coin_supply: Option<Value>,
    pub halving: Option<Value>,
    pub block_reward: Option<Value>,
    pub raw_summary: String,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub async fn full_network_analytics_report() -> Result<FullNetworkAnalyticsReport, String> {
    let client =
        KaspaApiClient::new(ApiClientConfig::default()).map_err(|error| error.to_string())?;

    let hashrate = fetch_optional_json(&client, "/info/hashrate").await;
    let max_hashrate = fetch_optional_json(&client, "/info/maxhashrate").await;
    let network = fetch_optional_json(&client, "/info/network").await;
    let blockdag = fetch_optional_json(&client, "/info/blockdag").await;
    let coin_supply = fetch_optional_json(&client, "/info/coinsupply").await;
    let halving = fetch_optional_json(&client, "/info/halving").await;
    let block_reward = fetch_optional_json(&client, "/info/blockreward").await;

    let mut warnings = Vec::new();

    push_warning_if_missing(&mut warnings, "hashrate", &hashrate);
    push_warning_if_missing(&mut warnings, "maxhashrate", &max_hashrate);
    push_warning_if_missing(&mut warnings, "network", &network);
    push_warning_if_missing(&mut warnings, "blockdag", &blockdag);
    push_warning_if_missing(&mut warnings, "coinsupply", &coin_supply);
    push_warning_if_missing(&mut warnings, "halving", &halving);
    push_warning_if_missing(&mut warnings, "blockreward", &block_reward);

    let cards = vec![
        metric_card(
            "hashrate",
            "Hashrate",
            first_number_string(&hashrate, &["hashrate", "value", "hashRate"])
                .unwrap_or_else(|| "Unavailable".to_string()),
            "H/s",
            &hashrate,
        ),
        metric_card(
            "max_hashrate",
            "Max Hashrate",
            first_number_string(
                &max_hashrate,
                &["maxHashrate", "max_hashrate", "value", "hashrate"],
            )
            .unwrap_or_else(|| "Unavailable".to_string()),
            "H/s",
            &max_hashrate,
        ),
        metric_card(
            "difficulty",
            "Difficulty",
            first_number_string(&network, &["difficulty", "networkDifficulty"])
                .unwrap_or_else(|| "Unavailable".to_string()),
            "",
            &network,
        ),
        metric_card(
            "virtual_daa_score",
            "Virtual DAA Score",
            first_number_string(
                &blockdag,
                &["virtualDaaScore", "virtual_daa_score", "daaScore"],
            )
            .unwrap_or_else(|| "Unavailable".to_string()),
            "",
            &blockdag,
        ),
        metric_card(
            "block_count",
            "Block Count",
            first_number_string(&blockdag, &["blockCount", "block_count", "blocks"])
                .unwrap_or_else(|| "Unavailable".to_string()),
            "",
            &blockdag,
        ),
        metric_card(
            "coin_supply",
            "Coin Supply",
            first_number_string(
                &coin_supply,
                &[
                    "circulatingSupply",
                    "circulating_supply",
                    "supply",
                    "coinSupply",
                ],
            )
            .unwrap_or_else(|| "Unavailable".to_string()),
            "KAS",
            &coin_supply,
        ),
        metric_card(
            "halving",
            "Halving",
            first_number_string(
                &halving,
                &[
                    "nextHalvingAmount",
                    "next_halving_amount",
                    "nextHalvingTimestamp",
                    "value",
                ],
            )
            .unwrap_or_else(|| "Unavailable".to_string()),
            "",
            &halving,
        ),
        metric_card(
            "block_reward",
            "Block Reward",
            first_number_string(
                &block_reward,
                &["blockreward", "blockReward", "reward", "value"],
            )
            .unwrap_or_else(|| "Unavailable".to_string()),
            "KAS",
            &block_reward,
        ),
    ];

    let raw_summary = serde_json::to_string_pretty(&serde_json::json!({
        "hashrate": hashrate,
        "max_hashrate": max_hashrate,
        "network": network,
        "blockdag": blockdag,
        "coin_supply": coin_supply,
        "halving": halving,
        "block_reward": block_reward
    }))
    .map_err(|error| error.to_string())?;

    let parsed_summary: Value =
        serde_json::from_str(&raw_summary).map_err(|error| error.to_string())?;

    Ok(FullNetworkAnalyticsReport {
        cards,
        hashrate: parsed_summary.get("hashrate").cloned(),
        max_hashrate: parsed_summary.get("max_hashrate").cloned(),
        network: parsed_summary.get("network").cloned(),
        blockdag: parsed_summary.get("blockdag").cloned(),
        coin_supply: parsed_summary.get("coin_supply").cloned(),
        halving: parsed_summary.get("halving").cloned(),
        block_reward: parsed_summary.get("block_reward").cloned(),
        raw_summary,
        warnings,
    })
}

async fn fetch_optional_json(client: &KaspaApiClient, path: &str) -> Option<Value> {
    client.get_json::<Value>(path).await.ok()
}

fn push_warning_if_missing(warnings: &mut Vec<String>, name: &str, value: &Option<Value>) {
    if value.is_none() {
        warnings.push(format!(
            "{name} endpoint is unavailable or returned an unsupported response."
        ));
    }
}

fn metric_card(
    key: &str,
    title: &str,
    value: String,
    unit: &str,
    source: &Option<Value>,
) -> NetworkMetricCard {
    NetworkMetricCard {
        key: key.to_string(),
        title: title.to_string(),
        value,
        unit: unit.to_string(),
        status: if source.is_some() {
            "ok"
        } else {
            "unavailable"
        }
        .to_string(),
    }
}

fn first_number_string(value: &Option<Value>, keys: &[&str]) -> Option<String> {
    let value = value.as_ref()?;
    find_number_string(value, keys)
}

fn find_number_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map.get(*key)
                    && let Some(text) = value_to_string(found)
                {
                    return Some(text);
                }
            }

            for nested in map.values() {
                if let Some(text) = find_number_string(nested, keys) {
                    return Some(text);
                }
            }

            None
        }
        Value::Array(values) => {
            for nested in values {
                if let Some(text) = find_number_string(nested, keys) {
                    return Some(text);
                }
            }

            None
        }
        _ => value_to_string(value),
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Number(number) => Some(number.to_string()),
        Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
        _ => None,
    }
}
