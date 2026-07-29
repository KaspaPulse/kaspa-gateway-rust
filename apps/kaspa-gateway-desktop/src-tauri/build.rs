fn main() {
    let mut attributes = tauri_build::Attributes::new();

    if std::env::var_os("CARGO_FEATURE_E2E_TEST").is_some() {
        println!("cargo:rerun-if-changed=capabilities-e2e");
        attributes = attributes.capabilities_path_pattern("./capabilities*/**/*");
    }

    if let Err(error) = tauri_build::try_build(attributes) {
        println!("{error:#}");
        std::process::exit(1);
    }
}
