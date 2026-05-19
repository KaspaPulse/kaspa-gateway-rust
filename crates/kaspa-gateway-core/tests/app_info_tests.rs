use kaspa_gateway_core::AppInfo;

#[test]
fn app_info_has_name() {
    let info = AppInfo::default();
    assert_eq!(info.name, "Kaspa Gateway");
}
