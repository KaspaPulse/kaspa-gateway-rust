#[path = "../dependabot_auto_merge.rs"]
mod dependabot_auto_merge;

fn main() {
    if std::env::args().len() != 1 {
        eprintln!("DEPENDABOT_AUTO_MERGE=FAIL: this command accepts no CLI overrides");
        std::process::exit(2);
    }
    match dependabot_auto_merge::from_environment().and_then(dependabot_auto_merge::run) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("DEPENDABOT_AUTO_MERGE=FAIL: {error}");
            std::process::exit(1);
        }
    }
}
