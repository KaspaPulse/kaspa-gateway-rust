use kaspa_gateway_api::{
    build_api_url, validate_address_for_url, AddressBalanceRaw, ApiClientConfig, KaspaApiClient,
};
use serde_json::json;
use url::Url;

#[test]
fn default_api_config_uses_https() {
    let config = ApiClientConfig::default();
    assert_eq!(config.base_url.scheme(), "https");
}

#[test]
fn api_config_rejects_plain_http() {
    let config = ApiClientConfig::new("http://api.kaspa.org");
    assert!(config.is_err());
}

#[test]
fn build_api_url_rejects_absolute_url_paths() {
    let base = Url::parse("https://api.kaspa.org").expect("valid base URL");
    let result = build_api_url(&base, "https://evil.example.com/test");
    assert!(result.is_err());
}

#[test]
fn build_api_url_normalizes_relative_paths() {
    let base = Url::parse("https://api.kaspa.org").expect("valid base URL");
    let url = build_api_url(&base, "info/network").expect("URL must build");
    assert_eq!(url.as_str(), "https://api.kaspa.org/info/network");
}

#[test]
fn client_builds_address_balance_url() {
    let client = KaspaApiClient::default_client().expect("client must build");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
    let url = client
        .address_balance_url(address)
        .expect("balance URL must build");

    assert_eq!(
        url.as_str(),
        "https://api.kaspa.org/addresses/kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g/balance"
    );
}

#[test]
fn unsafe_address_is_rejected_for_url() {
    let result = validate_address_for_url("kaspa:abc/../../bad");
    assert!(result.is_err());
}

#[test]
fn numeric_balance_parses_as_sompi_and_kas() {
    let raw = AddressBalanceRaw {
        balance: json!(100000000u64),
    };

    assert_eq!(raw.balance_as_sompi().expect("sompi"), 100000000);
    assert_eq!(raw.balance_as_kas().expect("kas"), 1.0);
}

#[test]
fn string_balance_parses_as_sompi() {
    let raw = AddressBalanceRaw {
        balance: json!("250000000"),
    };

    assert_eq!(raw.balance_as_sompi().expect("sompi"), 250000000);
}

#[test]
fn invalid_balance_is_rejected() {
    let raw = AddressBalanceRaw {
        balance: json!({ "bad": true }),
    };

    assert!(raw.balance_as_sompi().is_err());
}
