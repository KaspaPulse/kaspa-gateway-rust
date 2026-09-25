#[path = "../ci_diagnostics.rs"]
mod ci_diagnostics;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] {
        println!(
            "kgw-ci-diagnostics <dependency-review|capture-cargo-deny|capture-cargo-machete> [--directory PATH]"
        );
        return;
    }
    match ci_diagnostics::parse(args).and_then(ci_diagnostics::run) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("CI_DIAGNOSTICS=FAIL: {error}");
            std::process::exit(1);
        }
    }
}
