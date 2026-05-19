use kaspa_gateway_core::{AppInfo, KaspaAddress, MoneyAmount};

#[test]
fn app_info_has_name() {
    let info = AppInfo::default();
    assert_eq!(info.name, "Kaspa Gateway");
}

#[test]
fn valid_kaspa_address_is_accepted() {
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
    let parsed = KaspaAddress::parse(address).expect("address must parse");
    assert_eq!(parsed.as_str(), address);
}

#[test]
fn invalid_kaspa_address_is_rejected() {
    let result = KaspaAddress::parse("not-a-kaspa-address");
    assert!(result.is_err());
}

#[test]
fn money_amount_rejects_negative_values() {
    let result = MoneyAmount::new(-1.0, "USD");
    assert!(result.is_err());
}
