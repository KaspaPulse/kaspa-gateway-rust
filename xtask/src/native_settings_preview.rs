//! Read-only native settings-preview contract gate.
//! The application must already be running with a loopback CDP endpoint.
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

type GateResult<T> = Result<T, String>;
const LIMIT: usize = 1024 * 1024;
const HEADER_LIMIT: usize = 16 * 1024;
const TIMEOUT: Duration = Duration::from_secs(15);
const WS_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const WS_ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
static MASK_COUNTER: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Eq, PartialEq)]
struct WsEndpoint {
    host: String,
    port: u16,
    path: String,
}

fn port_value(raw: Option<&str>) -> GateResult<u16> {
    let raw = raw
        .filter(|value| !value.is_empty())
        .unwrap_or("49377")
        .trim();
    let parse_radix = |digits: &str, radix| {
        u64::from_str_radix(digits, radix)
            .map(|number| number as f64)
            .map_err(|_| "Invalid local CDP port".to_owned())
    };
    let value = if let Some(value) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        parse_radix(value, 16)
    } else if let Some(value) = raw.strip_prefix("0o").or_else(|| raw.strip_prefix("0O")) {
        parse_radix(value, 8)
    } else if let Some(value) = raw.strip_prefix("0b").or_else(|| raw.strip_prefix("0B")) {
        parse_radix(value, 2)
    } else {
        raw.parse::<f64>()
            .map_err(|_| "Invalid local CDP port".to_owned())
    }?;
    if !value.is_finite() || value.fract() != 0.0 || !(1.0..=65535.0).contains(&value) {
        return Err("Invalid local CDP port".to_owned());
    }
    Ok(value as u16)
}

fn connect_loopback(host: &str, port: u16) -> GateResult<TcpStream> {
    let normalized = host.trim_matches(['[', ']']);
    if let Ok(ip) = normalized.parse::<IpAddr>() {
        if !ip.is_loopback() {
            return Err("Only a loopback native debug endpoint is allowed".to_owned());
        }
    } else if !normalized.eq_ignore_ascii_case("localhost") {
        return Err("Only a loopback native debug endpoint is allowed".to_owned());
    }
    let addresses = (normalized, port)
        .to_socket_addrs()
        .map_err(|error| format!("Cannot resolve native debug endpoint: {error}"))?
        .filter(|address| address.ip().is_loopback())
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err("Native debug endpoint did not resolve to loopback".to_owned());
    }
    let mut last = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, TIMEOUT) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(TIMEOUT))
                    .map_err(|e| e.to_string())?;
                stream
                    .set_write_timeout(Some(TIMEOUT))
                    .map_err(|e| e.to_string())?;
                return Ok(stream);
            }
            Err(error) => last = Some(error),
        }
    }
    Err(format!(
        "Cannot connect to native debug endpoint: {}",
        last.map_or_else(|| "no loopback address".to_owned(), |e| e.to_string())
    ))
}

fn read_limited(stream: &mut TcpStream) -> GateResult<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = stream
            .read(&mut buffer)
            .map_err(|e| format!("Native CDP read failed: {e}"))?;
        if count == 0 {
            break;
        }
        if output.len().saturating_add(count) > LIMIT {
            return Err("Native CDP response exceeds 1 MiB".to_owned());
        }
        output.extend_from_slice(&buffer[..count]);
    }
    Ok(output)
}

fn http_body(bytes: &[u8]) -> GateResult<&[u8]> {
    let boundary = bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .ok_or_else(|| "Invalid native CDP HTTP response".to_owned())?;
    let header = std::str::from_utf8(&bytes[..boundary])
        .map_err(|_| "Invalid native CDP HTTP header".to_owned())?;
    let status = header.lines().next().unwrap_or_default();
    if !(status.starts_with("HTTP/1.1 200 ") || status.starts_with("HTTP/1.0 200 ")) {
        return Err(format!("Native CDP target query failed: {status}"));
    }
    if header
        .lines()
        .any(|line| line.eq_ignore_ascii_case("Transfer-Encoding: chunked"))
    {
        return Err("Chunked native CDP target responses are not supported".to_owned());
    }
    Ok(&bytes[boundary + 4..])
}

fn fetch_targets(port: u16) -> GateResult<Value> {
    let mut stream = connect_loopback("127.0.0.1", port)?;
    let request =
        format!("GET /json/list HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Native CDP request failed: {e}"))?;
    let bytes = read_limited(&mut stream)?;
    serde_json::from_slice(http_body(&bytes)?)
        .map_err(|error| format!("Invalid native CDP target JSON: {error}"))
}

fn select_socket_url(targets: &Value) -> GateResult<String> {
    let targets = targets
        .as_array()
        .ok_or_else(|| "Native CDP target list must be an array".to_owned())?;
    for target in targets {
        let kind = target.get("type").and_then(Value::as_str);
        let url = target
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if kind == Some("page")
            && (url.starts_with("http://tauri.localhost")
                || url.starts_with("http://127.0.0.1:1430/"))
        {
            return target
                .get("webSocketDebuggerUrl")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| "Native KGW WebView is missing its debugger socket".to_owned());
        }
    }
    Err("The native KGW WebView must already be running".to_owned())
}

fn parse_ws_endpoint(raw: &str) -> GateResult<WsEndpoint> {
    let rest = raw
        .strip_prefix("ws://")
        .ok_or_else(|| "Native debugger socket must use local ws:// transport".to_owned())?;
    let slash = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..slash];
    let path = if slash == rest.len() {
        "/"
    } else {
        &rest[slash..]
    };
    let (host, port) = if let Some(after) = authority.strip_prefix('[') {
        let end = after
            .find(']')
            .ok_or_else(|| "Invalid IPv6 debugger socket".to_owned())?;
        let host = &after[..end];
        let suffix = &after[end + 1..];
        let port = suffix
            .strip_prefix(':')
            .ok_or_else(|| "Debugger socket port is required".to_owned())?;
        (host.to_owned(), port)
    } else {
        let (host, port) = authority
            .rsplit_once(':')
            .ok_or_else(|| "Debugger socket port is required".to_owned())?;
        (host.to_owned(), port)
    };
    let port = port
        .parse::<u16>()
        .map_err(|_| "Invalid debugger socket port".to_owned())?;
    let allowed = host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1";
    if !allowed {
        return Err("Only a loopback native debug endpoint is allowed".to_owned());
    }
    if path.contains('#') {
        return Err("Debugger socket fragments are not allowed".to_owned());
    }
    Ok(WsEndpoint {
        host,
        port,
        path: path.to_owned(),
    })
}

fn read_http_header(stream: &mut TcpStream) -> GateResult<String> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        if bytes.len() >= HEADER_LIMIT {
            return Err("Native debugger handshake header is too large".to_owned());
        }
        let mut byte = [0u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|e| format!("Debugger handshake read failed: {e}"))?;
        bytes.push(byte[0]);
    }
    String::from_utf8(bytes).map_err(|_| "Invalid debugger handshake encoding".to_owned())
}

fn websocket(endpoint: &WsEndpoint) -> GateResult<TcpStream> {
    let mut stream = connect_loopback(&endpoint.host, endpoint.port)?;
    let host = if endpoint.host == "::1" {
        format!("[::1]:{}", endpoint.port)
    } else {
        format!("{}:{}", endpoint.host, endpoint.port)
    };
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
        endpoint.path, host, WS_KEY
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("Debugger handshake write failed: {e}"))?;
    let response = read_http_header(&mut stream)?;
    let mut lines = response.lines();
    let status = lines.next().unwrap_or_default();
    if !status.contains(" 101 ") {
        return Err(format!("Debugger WebSocket upgrade failed: {status}"));
    }
    let mut upgrade = false;
    let mut connection = false;
    let mut accept = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match name.trim().to_ascii_lowercase().as_str() {
            "upgrade" => upgrade |= value.eq_ignore_ascii_case("websocket"),
            "connection" => {
                connection |= value
                    .split(',')
                    .any(|v| v.trim().eq_ignore_ascii_case("upgrade"))
            }
            "sec-websocket-accept" => accept |= value == WS_ACCEPT,
            _ => {}
        }
    }
    if !(upgrade && connection && accept) {
        return Err("Debugger WebSocket handshake validation failed".to_owned());
    }
    Ok(stream)
}

fn mask_key() -> [u8; 4] {
    let counter = MASK_COUNTER.fetch_add(1, Ordering::Relaxed);
    let ticks = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.subsec_nanos());
    let mut value = ticks ^ counter.rotate_left(13) ^ std::process::id();
    value ^= value << 13;
    value ^= value >> 17;
    value ^= value << 5;
    value.to_be_bytes()
}

fn write_frame(mut writer: impl Write, opcode: u8, payload: &[u8]) -> GateResult<()> {
    if payload.len() > LIMIT {
        return Err("Native debugger message exceeds 1 MiB".to_owned());
    }
    let mut header = vec![0x80 | (opcode & 0x0f)];
    match payload.len() {
        0..=125 => header.push(0x80 | payload.len() as u8),
        126..=65535 => {
            header.push(0x80 | 126);
            header.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        }
        _ => {
            header.push(0x80 | 127);
            header.extend_from_slice(&(payload.len() as u64).to_be_bytes());
        }
    }
    let mask = mask_key();
    header.extend_from_slice(&mask);
    writer
        .write_all(&header)
        .map_err(|e| format!("Debugger WebSocket write failed: {e}"))?;
    let encoded = payload
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ mask[index % 4])
        .collect::<Vec<_>>();
    writer
        .write_all(&encoded)
        .map_err(|e| format!("Debugger WebSocket write failed: {e}"))
}

#[derive(Debug, Eq, PartialEq)]
enum Frame {
    Text(String),
    Ping(Vec<u8>),
    Pong,
    Close,
}

fn read_frame(mut reader: impl Read) -> GateResult<Frame> {
    let mut head = [0u8; 2];
    reader
        .read_exact(&mut head)
        .map_err(|e| format!("Debugger WebSocket read failed: {e}"))?;
    if head[0] & 0x80 == 0 {
        return Err("Fragmented debugger WebSocket messages are not supported".to_owned());
    }
    if head[1] & 0x80 != 0 {
        return Err("Debugger server frames must not be masked".to_owned());
    }
    let opcode = head[0] & 0x0f;
    let mut length = u64::from(head[1] & 0x7f);
    if length == 126 {
        let mut bytes = [0u8; 2];
        reader.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        length = u64::from(u16::from_be_bytes(bytes));
    } else if length == 127 {
        let mut bytes = [0u8; 8];
        reader.read_exact(&mut bytes).map_err(|e| e.to_string())?;
        length = u64::from_be_bytes(bytes);
        if length >> 63 != 0 {
            return Err("Invalid debugger WebSocket length".to_owned());
        }
    }
    if length > LIMIT as u64 {
        return Err("Native debugger message exceeds 1 MiB".to_owned());
    }
    let mut payload = vec![0u8; length as usize];
    reader
        .read_exact(&mut payload)
        .map_err(|e| format!("Debugger WebSocket payload failed: {e}"))?;
    match opcode {
        0x1 => String::from_utf8(payload)
            .map(Frame::Text)
            .map_err(|_| "Debugger WebSocket text is not UTF-8".to_owned()),
        0x8 => Ok(Frame::Close),
        0x9 => Ok(Frame::Ping(payload)),
        0xA => Ok(Frame::Pong),
        _ => Err(format!("Unsupported debugger WebSocket opcode: {opcode}")),
    }
}

struct Cdp {
    stream: TcpStream,
    next_id: u64,
}

impl Cdp {
    fn connect(raw: &str) -> GateResult<Self> {
        let endpoint = parse_ws_endpoint(raw)?;
        Ok(Self {
            stream: websocket(&endpoint)?,
            next_id: 0,
        })
    }

    fn evaluate(&mut self, expression: &str) -> GateResult<Value> {
        self.next_id += 1;
        let id = self.next_id;
        let request = json!({
            "id": id,
            "method": "Runtime.evaluate",
            "params": {"expression": expression, "awaitPromise": true, "returnByValue": true}
        });
        write_frame(&mut self.stream, 0x1, request.to_string().as_bytes())?;
        loop {
            match read_frame(&mut self.stream)? {
                Frame::Ping(payload) => write_frame(&mut self.stream, 0xA, &payload)?,
                Frame::Pong => {}
                Frame::Close => return Err("Native debugger socket closed".to_owned()),
                Frame::Text(text) => {
                    let response: Value = serde_json::from_str(&text)
                        .map_err(|e| format!("Invalid debugger JSON: {e}"))?;
                    if response.get("id").and_then(Value::as_u64) != Some(id) {
                        continue;
                    }
                    if let Some(error) = response.get("error") {
                        return Err(format!("Native debugger returned an error: {error}"));
                    }
                    let result = response
                        .get("result")
                        .ok_or_else(|| "Native debugger response is missing result".to_owned())?;
                    if result.get("exceptionDetails").is_some() {
                        return Err("Native WebView evaluation failed".to_owned());
                    }
                    return Ok(result
                        .get("result")
                        .and_then(|remote| remote.get("value"))
                        .cloned()
                        .unwrap_or(Value::Null));
                }
            }
        }
    }

    fn close(&mut self) {
        let _ = write_frame(&mut self.stream, 0x8, &[]);
    }
}

fn preview_expression(payload: &Value) -> String {
    format!(
        "(async () => {{ if (!window.__TAURI__?.core?.invoke) throw Error(\"Native Tauri IPC unavailable\"); try {{ return {{ ok: true, value: await window.__TAURI__.core.invoke(\"kgw_runtime_settings_preview_v1\", {}) }}; }} catch (error) {{ return {{ ok: false, error: String(error) }}; }} }})()",
        payload
    )
}

fn require(condition: bool, message: impl Into<String>) -> GateResult<()> {
    condition.then_some(()).ok_or_else(|| message.into())
}

fn negative_payload(kind: &str, role: Value) -> Value {
    json!({
        "network": "mainnet",
        "runtimeRole": role,
        "nodeKind": if kind == "node" { "integrated-inproc" } else { "remote" },
        "bridgeKind": if kind == "node" { "disable" } else { "official-external-node" }
    })
}

pub fn run() -> GateResult<()> {
    let port = port_value(std::env::var("KGW_NATIVE_CDP_PORT").ok().as_deref())?;
    let result = run_at_port(port)?;
    let case_count = result
        .get("cases")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let positive_cases = result
        .get("positiveCases")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    println!(
        "NATIVE_SETTINGS_PREVIEW state=VERIFIED_SUCCESS cases={case_count} positive_cases={positive_cases}"
    );
    Ok(())
}

fn run_at_port(port: u16) -> GateResult<Value> {
    let targets = fetch_targets(port)?;
    let socket = select_socket_url(&targets)?;
    let mut cdp = Cdp::connect(&socket)?;
    let result = (|| {
        let mut cases = Vec::new();
        for kind in ["node", "bridge"] {
            for role in [
                Value::Null,
                json!(kind),
                json!(format!(" {} ", kind.to_ascii_uppercase())),
                json!("invalid"),
            ] {
                let payload = negative_payload(kind, role.clone());
                let value = cdp.evaluate(&preview_expression(&payload))?;
                require(
                    value.get("ok").and_then(Value::as_bool) == Some(false),
                    format!("Missing typed {kind} settings must reject role {role}"),
                )?;
                let error = value
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let expected = if role == json!("invalid") {
                    "runtimeRole must be"
                } else if kind == "node" {
                    "effectiveNodeSettings is required"
                } else {
                    "effectiveBridgeSettings is required"
                };
                require(
                    error.contains(expected),
                    format!("Unexpected {kind} rejection: {error}"),
                )?;
                cases.push(json!({"kind":kind,"role":role,"rejected":true,"error":error}));
            }
        }
        let payload = json!({
            "network":"mainnet","nodeKind":"integrated-inproc","bridgeKind":"disable",
            "runtimeRole":" NODE ","effectiveNodeSettings":{
                "asyncThreads":2,"ramScale":0.1,"outboundTarget":0,"inboundLimit":0,
                "disableDnsSeeding":true,"p2pListen":"127.0.0.1:16111"
            }
        });
        let explicit = cdp.evaluate(&preview_expression(&payload))?;
        require(
            explicit.get("ok").and_then(Value::as_bool) == Some(true),
            "Explicit settings preview must succeed",
        )?;
        require(
            explicit
                .pointer("/value/runtimeRole")
                .and_then(Value::as_str)
                == Some("node"),
            "Explicit preview did not normalize runtimeRole",
        )?;
        require(
            explicit
                .pointer("/value/effectiveNodeSettings/asyncThreads")
                .and_then(Value::as_i64)
                == Some(2),
            "Explicit preview changed asyncThreads",
        )?;
        let mut inferred_payload = payload.clone();
        inferred_payload["runtimeRole"] = Value::Null;
        let inferred = cdp.evaluate(&preview_expression(&inferred_payload))?;
        require(
            inferred == explicit,
            "Inferred and normalized role must resolve the same payload",
        )?;
        let time = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|e| format!("Cannot format verification time: {e}"))?;
        Ok(json!({
            "state":"VERIFIED_SUCCESS",
            "method":"actual-native-read-only-IPC",
            "time":time,
            "cases":cases,
            "positiveCases":2
        }))
    })();
    cdp.close();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn port_contract_matches_node_number_boundary() {
        assert_eq!(port_value(None).unwrap(), 49377);
        assert_eq!(port_value(Some("")).unwrap(), 49377);
        assert_eq!(port_value(Some("0xC0E1")).unwrap(), 49377);
        assert_eq!(port_value(Some("65535")).unwrap(), 65535);
        for invalid in [" ", "0", "65536", "1.5", "NaN", "-1"] {
            assert!(port_value(Some(invalid)).is_err(), "{invalid}");
        }
    }

    #[test]
    fn target_selection_preserves_native_page_contract() {
        let targets = json!([
            {"type":"page","url":"https://example.invalid","webSocketDebuggerUrl":"ws://127.0.0.1:1/a"},
            {"type":"page","url":"http://tauri.localhost/","webSocketDebuggerUrl":"ws://127.0.0.1:49377/devtools/page/1"}
        ]);
        assert_eq!(
            select_socket_url(&targets).unwrap(),
            "ws://127.0.0.1:49377/devtools/page/1"
        );
        assert!(select_socket_url(&json!([])).is_err());
    }

    #[test]
    fn websocket_endpoint_is_loopback_only() {
        assert_eq!(
            parse_ws_endpoint("ws://127.0.0.1:49377/devtools/page/1")
                .unwrap()
                .host,
            "127.0.0.1"
        );
        assert_eq!(
            parse_ws_endpoint("ws://[::1]:49377/devtools/page/1")
                .unwrap()
                .host,
            "::1"
        );
        assert!(parse_ws_endpoint("ws://192.0.2.1:49377/devtools/page/1").is_err());
        assert!(parse_ws_endpoint("wss://127.0.0.1:49377/devtools/page/1").is_err());
        assert!(parse_ws_endpoint("ws://127.0.0.1/devtools/page/1").is_err());
    }

    #[test]
    fn preview_expression_has_fixed_tauri_command_and_exact_payload() {
        let payload = json!({"network":"mainnet","runtimeRole":null});
        let expression = preview_expression(&payload);
        assert!(expression.contains("kgw_runtime_settings_preview_v1"));
        assert!(expression.contains(r#"{"network":"mainnet","runtimeRole":null}"#));
        assert!(!expression.contains("eval("));
    }

    #[test]
    fn server_frame_decoder_accepts_text_and_rejects_masked_or_fragmented() {
        let text = br#"{"id":1}"#;
        let mut frame = vec![0x81, text.len() as u8];
        frame.extend_from_slice(text);
        assert_eq!(
            read_frame(Cursor::new(frame)).unwrap(),
            Frame::Text(String::from_utf8(text.to_vec()).unwrap())
        );
        assert!(read_frame(Cursor::new(vec![0x81, 0x80])).is_err());
        assert!(read_frame(Cursor::new(vec![0x01, 0x00])).is_err());
    }

    fn read_client_text(stream: &mut TcpStream) -> Value {
        let mut head = [0u8; 2];
        stream.read_exact(&mut head).unwrap();
        assert_eq!(head[0] & 0x0f, 0x1);
        assert_ne!(head[1] & 0x80, 0);
        let mut length = u64::from(head[1] & 0x7f);
        if length == 126 {
            let mut bytes = [0u8; 2];
            stream.read_exact(&mut bytes).unwrap();
            length = u64::from(u16::from_be_bytes(bytes));
        } else if length == 127 {
            let mut bytes = [0u8; 8];
            stream.read_exact(&mut bytes).unwrap();
            length = u64::from_be_bytes(bytes);
        }
        assert!(length <= LIMIT as u64);
        let mut mask = [0u8; 4];
        stream.read_exact(&mut mask).unwrap();
        let mut payload = vec![0u8; length as usize];
        stream.read_exact(&mut payload).unwrap();
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
        serde_json::from_slice(&payload).unwrap()
    }

    fn write_server_text(stream: &mut TcpStream, value: &Value) {
        let payload = value.to_string().into_bytes();
        let mut header = vec![0x81];
        if payload.len() <= 125 {
            header.push(payload.len() as u8);
        } else {
            header.push(126);
            header.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        }
        stream.write_all(&header).unwrap();
        stream.write_all(&payload).unwrap();
    }

    fn read_header(stream: &mut TcpStream) -> String {
        let mut bytes = Vec::new();
        while !bytes.ends_with(b"\r\n\r\n") {
            let mut byte = [0u8; 1];
            stream.read_exact(&mut byte).unwrap();
            bytes.push(byte[0]);
            assert!(bytes.len() < HEADER_LIMIT);
        }
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn full_gate_uses_loopback_cdp_and_fixed_tauri_preview_contract() {
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut http, _) = listener.accept().unwrap();
            let request = read_header(&mut http);
            assert!(request.starts_with("GET /json/list HTTP/1.0\r\n"));
            let body = json!([{
                "type":"page",
                "url":"http://tauri.localhost/",
                "webSocketDebuggerUrl":format!("ws://127.0.0.1:{port}/devtools/page/fixture")
            }])
            .to_string();
            write!(
                http,
                "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
            drop(http);

            let (mut ws, _) = listener.accept().unwrap();
            let handshake = read_header(&mut ws);
            assert!(handshake.starts_with("GET /devtools/page/fixture HTTP/1.1\r\n"));
            assert!(handshake.contains("Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ=="));
            write!(ws, "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {WS_ACCEPT}\r\n\r\n").unwrap();

            for index in 0..10 {
                let request = read_client_text(&mut ws);
                assert_eq!(request["method"], "Runtime.evaluate");
                let id = request["id"].as_u64().unwrap();
                let expression = request
                    .pointer("/params/expression")
                    .and_then(Value::as_str)
                    .unwrap();
                assert!(expression.contains("kgw_runtime_settings_preview_v1"));
                let value = if expression.contains("effectiveNodeSettings") {
                    json!({"ok":true,"value":{"runtimeRole":"node","effectiveNodeSettings":{"asyncThreads":2}}})
                } else if expression.contains(r#""runtimeRole":"invalid""#) {
                    json!({"ok":false,"error":"runtimeRole must be node or bridge"})
                } else if expression.contains(r#""nodeKind":"integrated-inproc""#) {
                    json!({"ok":false,"error":"effectiveNodeSettings is required"})
                } else {
                    json!({"ok":false,"error":"effectiveBridgeSettings is required"})
                };
                write_server_text(
                    &mut ws,
                    &json!({"id":id,"result":{"result":{"value":value}}}),
                );
                assert_eq!(id as usize, index + 1);
            }
        });

        let receipt = run_at_port(port).unwrap();
        assert_eq!(receipt["state"], "VERIFIED_SUCCESS");
        assert_eq!(receipt["positiveCases"], 2);
        assert_eq!(receipt["cases"].as_array().unwrap().len(), 8);
        server.join().unwrap();
    }
}
