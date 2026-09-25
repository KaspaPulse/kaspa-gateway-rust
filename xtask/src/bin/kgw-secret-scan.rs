#[path = "../secret_scan.rs"]
mod secret_scan;
#[path = "../trufflehog_policy.rs"]
mod trufflehog_policy;
#[path = "../verified_tool.rs"]
mod verified_tool;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] {
        println!("kgw-secret-scan [--root PATH] [--archive PINNED_ARCHIVE] [--verify-only]");
        return;
    }
    if let Err(error) = secret_scan::parse(args).and_then(secret_scan::run) {
        eprintln!("SECRET_SCAN=FAIL: {error}");
        std::process::exit(error.code);
    }
}
