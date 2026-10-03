// Standalone synthetic same-process fixture; no Kaspa runtime or external network.
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};
fn value(args: &[String], name: &str) -> String {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .expect("fixture option")
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--kgw-live-smoke-parent") {
        let app = PathBuf::from(value(&args, "--appdir"));
        let network = value(&args, "--network");
        let endpoint = value(&args, "--rpc");
        let scenario = fs::read_to_string(app.join("fixture-scenario.txt"))
            .unwrap_or_else(|_| "healthy".to_owned());
        fs::create_dir_all(&app).unwrap();
        let mut seen = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(app.join("fixture-parent-argv.txt"))
            .unwrap();
        writeln!(seen, "PID={} ARGS={:?}", std::process::id(), args).unwrap();
        seen.sync_all().unwrap();
        println!("SYNTHETIC_SMOKE_PARENT network={network}");
        eprintln!("SYNTHETIC_SMOKE_DIAGNOSTIC");
        if scenario == "early-exit" {
            std::process::exit(37);
        }
        let listener = TcpListener::bind(&endpoint).unwrap();
        let mut count = 0_u64;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            count += 1;
            if scenario == "probe-timeout" {
                std::thread::sleep(Duration::from_secs(30));
            }
            let actual = if scenario == "wrong-network" {
                "mainnet"
            } else {
                network.as_str()
            };
            let peers = if scenario == "no-peers" { 0 } else { 2 };
            let delta = if scenario == "stagnant" { 0 } else { count };
            let report = format!(
                r#"{{"endpoint":"grpc://{endpoint}","network":"{actual}","serverVersion":"synthetic-2.1.0","isSynced":false,"peerCount":{peers},"virtualDaaScore":{},"blockCount":{},"headerCount":{}}}"#,
                9007199254740993_u64 + delta,
                100 + delta,
                200 + delta
            );
            if stream.write_all(report.as_bytes()).is_err() {
                continue;
            }
        }
    } else {
        let endpoint = value(&args, "--rpc");
        let expected = value(&args, "--expect-network");
        let address: SocketAddr = endpoint.strip_prefix("grpc://").unwrap().parse().unwrap();
        let deadline = Instant::now() + Duration::from_millis(300);
        let mut stream = loop {
            match TcpStream::connect_timeout(&address, Duration::from_millis(50)) {
                Ok(stream) => break stream,
                Err(error) => {
                    if Instant::now() >= deadline {
                        eprintln!("SYNTHETIC_CONNECT_FAILURE {error}");
                        std::process::exit(23);
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .unwrap();
        let mut text = String::new();
        if let Err(error) = stream.read_to_string(&mut text) {
            eprintln!("SYNTHETIC_READ_FAILURE {error}");
            std::process::exit(24);
        }
        if !text.contains(&format!(r#""network":"{expected}""#)) {
            eprintln!("SYNTHETIC_NETWORK_MISMATCH");
            std::process::exit(25);
        }
        println!("{text}");
    }
}
