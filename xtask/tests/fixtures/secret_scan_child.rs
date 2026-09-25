// Synthetic scanner output only. No credential validation or repository scan.
use std::io::{self, Write};
fn main() {
    let mode = std::env::args().nth(1).expect("fixture mode required");
    let allowed = r#"{"SourceMetadata":{"Data":{"Git":{"commit":"8f209ba516707b11098bd962972da38157346833","file":"crates/kaspa-gateway-security/src/lib.rs","line":374}}},"DetectorName":"URI","DecoderName":"PLAIN","Verified":false,"Raw":"SYNTHETIC_VALUE_MUST_NOT_BE_PRINTED"}"#;
    let text = match mode.as_str() {
        "empty" => String::new(),
        "invalid" | "failed" => "not-json\n".to_owned(),
        "scalar" => "[]\n".to_owned(),
        "unexpected" => "{\"Raw\":\"SYNTHETIC_VALUE_MUST_NOT_BE_PRINTED\"}\n".to_owned(),
        "allowed" => format!("{allowed}\n"),
        "duplicate" => format!("{allowed}\n{allowed}\n"),
        _ => panic!("invalid fixture mode"),
    };
    io::stdout().write_all(text.as_bytes()).unwrap();
    io::stdout().flush().unwrap();
    std::process::exit(if mode == "failed" { 37 } else { 0 });
}
