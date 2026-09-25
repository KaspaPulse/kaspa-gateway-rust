#[path = "../live_network_smoke/mod.rs"]
mod live_network_smoke;
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] {
        println!(
            "kgw-live-network-smoke [--repository PATH] [--networks mainnet,testnet10] [--ready-timeout-seconds N] [--observation-seconds N] [--skip-build] [--plan-only]"
        );
        println!(
            "Execution is Windows-only and may start the selected real nodes. --plan-only does not build, launch, install or modify application data."
        );
        return;
    }
    if let Err(error) = live_network_smoke::parse(args).and_then(live_network_smoke::run) {
        eprintln!("LIVE_NETWORK_SMOKE=FAIL: {error}");
        std::process::exit(1);
    }
}
