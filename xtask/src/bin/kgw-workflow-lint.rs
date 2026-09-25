#[path = "../verified_tool.rs"]
mod verified_tool;

#[path = "../workflow_lint.rs"]
mod workflow_lint;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] {
        println!("kgw-workflow-lint [--root PATH] [--archive PINNED_ARCHIVE] [--verify-only]");
        println!(
            "Verifies actionlint 1.7.12 before use. Execution requires Linux x86_64 and shellcheck."
        );
        return;
    }
    let result = workflow_lint::parse(args).and_then(workflow_lint::run);
    if let Err(error) = result {
        eprintln!("WORKFLOW_LINT=FAIL: {error}");
        std::process::exit(1);
    }
}
