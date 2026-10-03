#[path = "../native_settings_preview.rs"]
mod native_settings_preview;

fn main() {
    if let Err(error) = native_settings_preview::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
