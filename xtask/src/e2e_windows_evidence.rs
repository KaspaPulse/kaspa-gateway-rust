use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[cfg(windows)]
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_MODULE, MIB_TCP6TABLE_OWNER_MODULE,
    MIB_TCPROW_OWNER_MODULE, MIB_TCPTABLE_OWNER_MODULE, TCP_TABLE_OWNER_MODULE_ALL,
};

const AF_INET: u32 = 2;
const AF_INET6: u32 = 23;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Options {
    repository: PathBuf,
    output_directory: PathBuf,
    ports: Vec<u16>,
    desktop_pid: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ProcessSnapshot {
    process_id: u32,
    parent_process_id: u32,
    name: String,
    executable_path: String,
    command_line: String,
    creation_date: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct TcpSnapshot {
    local_address: String,
    local_port: u16,
    remote_address: String,
    remote_port: u16,
    state: u32,
    owning_process: u32,
    creation_time: Option<String>,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let options = parse_options(args)?;
    run(&options)
}

fn parse_options(args: &mut impl Iterator<Item = String>) -> Result<Options, String> {
    let mut repository = None;
    let mut output_directory = None;
    let mut ports = Vec::new();
    let mut desktop_pid = None;

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--repository" => repository = Some(PathBuf::from(value)),
            "--output-directory" => output_directory = Some(PathBuf::from(value)),
            "--ports" => ports = parse_ports(&value)?,
            "--desktop-pid" => {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    let parsed = trimmed
                        .parse::<u32>()
                        .map_err(|_| "--desktop-pid must be a positive integer".to_owned())?;
                    if parsed == 0 {
                        return Err("--desktop-pid must be a positive integer".to_owned());
                    }
                    desktop_pid = Some(parsed);
                }
            }
            _ => return Err(format!("unknown e2e-windows-evidence option: {flag}")),
        }
    }

    Ok(Options {
        repository: repository.ok_or_else(|| "--repository is required".to_owned())?,
        output_directory: output_directory
            .ok_or_else(|| "--output-directory is required".to_owned())?,
        ports,
        desktop_pid,
    })
}

fn parse_ports(value: &str) -> Result<Vec<u16>, String> {
    let mut ports = Vec::new();
    for raw in value.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let port = trimmed
            .parse::<u16>()
            .map_err(|_| format!("invalid TCP port: {trimmed}"))?;
        if port == 0 {
            return Err("TCP port must be in 1..=65535".to_owned());
        }
        if !ports.contains(&port) {
            ports.push(port);
        }
    }
    Ok(ports)
}

fn run(options: &Options) -> Result<String, String> {
    #[cfg(windows)]
    {
        run_windows(options)
    }

    #[cfg(not(windows))]
    {
        let _ = options;
        Err("e2e-windows-evidence requires Windows".to_owned())
    }
}

#[cfg(windows)]
fn run_windows(options: &Options) -> Result<String, String> {
    let repository = options.repository.canonicalize().map_err(|error| {
        format!(
            "failed to resolve repository {}: {error}",
            options.repository.display()
        )
    })?;
    fs::create_dir_all(&options.output_directory).map_err(|error| {
        format!(
            "failed to create output directory {}: {error}",
            options.output_directory.display()
        )
    })?;

    let process_tree = capture_process_tree(&repository, options.desktop_pid)?;
    let port_state = capture_port_state(&options.ports)?;

    let process_tree_path = options.output_directory.join("process-tree.json");
    let port_state_path = options.output_directory.join("port-state.json");

    write_powershell_shaped_json(
        &process_tree_path,
        process_tree.iter().map(process_json).collect(),
    )?;
    write_powershell_shaped_json(&port_state_path, port_state.iter().map(tcp_json).collect())?;

    serde_json::to_string(&json!({
        "process_tree_file": process_tree_path.to_string_lossy(),
        "port_state_file": port_state_path.to_string_lossy(),
        "process_count": process_tree.len(),
        "port_record_count": port_state.len(),
    }))
    .map_err(|error| format!("failed to serialize evidence summary: {error}"))
}

#[cfg(windows)]
fn capture_process_tree(
    repository: &Path,
    desktop_pid: Option<u32>,
) -> Result<Vec<ProcessSnapshot>, String> {
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything().without_tasks()),
    );

    let repository_text = repository.to_string_lossy().to_string();
    let repository_lower = repository_text.to_ascii_lowercase();

    let mut snapshots = HashMap::<u32, ProcessSnapshot>::new();
    let mut children = HashMap::<u32, Vec<u32>>::new();
    let mut roots = HashSet::<u32>::new();

    for (pid, process) in system.processes() {
        let process_id = pid.as_u32();
        let parent_process_id = process.parent().map(|pid| pid.as_u32()).unwrap_or(0);
        let executable_path = process
            .exe()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default();
        let command_line = render_command_line(process.cmd());
        let name = process.name().to_string_lossy().to_string();
        let creation_date = unix_seconds_rfc3339(process.start_time())
            .unwrap_or_else(|_| process.start_time().to_string());

        if parent_process_id != 0 {
            children
                .entry(parent_process_id)
                .or_default()
                .push(process_id);
        }

        if process_matches_repository(&executable_path, &command_line, &repository_lower) {
            roots.insert(process_id);
        }

        snapshots.insert(
            process_id,
            ProcessSnapshot {
                process_id,
                parent_process_id,
                name,
                executable_path,
                command_line,
                creation_date,
            },
        );
    }

    if let Some(pid) = desktop_pid {
        roots.insert(pid);
    }

    let mut wanted = HashSet::<u32>::new();
    let mut queue: VecDeque<u32> = roots.iter().copied().collect();
    while let Some(pid) = queue.pop_front() {
        if !wanted.insert(pid) {
            continue;
        }
        if let Some(child_ids) = children.get(&pid) {
            queue.extend(child_ids.iter().copied());
        }
    }

    let mut result: Vec<ProcessSnapshot> = wanted
        .into_iter()
        .filter_map(|pid| snapshots.remove(&pid))
        .collect();
    result.sort_by_key(|entry| (entry.parent_process_id, entry.process_id));
    Ok(result)
}

fn process_matches_repository(
    executable: &str,
    command_line: &str,
    repository_lower: &str,
) -> bool {
    let executable_lower = executable.to_ascii_lowercase();
    let command_lower = command_line.to_ascii_lowercase();
    (!executable_lower.is_empty() && executable_lower.starts_with(repository_lower))
        || (command_lower.contains(repository_lower) && command_lower.contains("kaspa-gateway"))
}

fn render_command_line(args: &[std::ffi::OsString]) -> String {
    args.iter()
        .map(|value| quote_windows_argument(&value.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_windows_argument(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_owned();
    }
    if !value.chars().any(|ch| ch.is_whitespace() || ch == '"') {
        return value.to_owned();
    }

    let mut rendered = String::from("\"");
    let mut backslashes = 0usize;
    for ch in value.chars() {
        match ch {
            '\\' => backslashes += 1,
            '"' => {
                rendered.push_str(&"\\".repeat(backslashes * 2 + 1));
                rendered.push('"');
                backslashes = 0;
            }
            _ => {
                rendered.push_str(&"\\".repeat(backslashes));
                backslashes = 0;
                rendered.push(ch);
            }
        }
    }
    rendered.push_str(&"\\".repeat(backslashes * 2));
    rendered.push('"');
    rendered
}

fn unix_seconds_rfc3339(seconds: u64) -> Result<String, String> {
    let seconds = i64::try_from(seconds).map_err(|_| "process start time overflow".to_owned())?;
    OffsetDateTime::from_unix_timestamp(seconds)
        .map_err(|error| format!("invalid process start time: {error}"))?
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format process start time: {error}"))
}

#[cfg(windows)]
fn capture_port_state(ports: &[u16]) -> Result<Vec<TcpSnapshot>, String> {
    if ports.is_empty() {
        return Ok(Vec::new());
    }
    let wanted: HashSet<u16> = ports.iter().copied().collect();
    let mut rows = Vec::new();
    rows.extend(tcp4_rows()?.into_iter().filter_map(|row| {
        let local_port = decode_mib_port(row.dwLocalPort);
        let remote_port = decode_mib_port(row.dwRemotePort);
        if !wanted.contains(&local_port) && !wanted.contains(&remote_port) {
            return None;
        }
        Some(TcpSnapshot {
            local_address: Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()).to_string(),
            local_port,
            remote_address: Ipv4Addr::from(row.dwRemoteAddr.to_ne_bytes()).to_string(),
            remote_port,
            state: row.dwState,
            owning_process: row.dwOwningPid,
            creation_time: filetime_ticks_rfc3339(row.liCreateTimestamp).ok(),
        })
    }));
    rows.extend(tcp6_rows()?.into_iter().filter_map(|row| {
        let local_port = decode_mib_port(row.dwLocalPort);
        let remote_port = decode_mib_port(row.dwRemotePort);
        if !wanted.contains(&local_port) && !wanted.contains(&remote_port) {
            return None;
        }
        Some(TcpSnapshot {
            local_address: ipv6_with_scope(row.ucLocalAddr, row.dwLocalScopeId),
            local_port,
            remote_address: ipv6_with_scope(row.ucRemoteAddr, row.dwRemoteScopeId),
            remote_port,
            state: row.dwState,
            owning_process: row.dwOwningPid,
            creation_time: filetime_ticks_rfc3339(row.liCreateTimestamp).ok(),
        })
    }));
    rows.sort_by(|a, b| {
        (
            &a.local_address,
            a.local_port,
            &a.remote_address,
            a.remote_port,
            a.owning_process,
        )
            .cmp(&(
                &b.local_address,
                b.local_port,
                &b.remote_address,
                b.remote_port,
                b.owning_process,
            ))
    });
    Ok(rows)
}

fn decode_mib_port(value: u32) -> u16 {
    u16::from_be(value as u16)
}

fn ipv6_with_scope(bytes: [u8; 16], scope_id: u32) -> String {
    let address = Ipv6Addr::from(bytes).to_string();
    if scope_id == 0 {
        address
    } else {
        format!("{address}%{scope_id}")
    }
}

fn filetime_ticks_rfc3339(ticks: i64) -> Result<String, String> {
    const WINDOWS_TO_UNIX_EPOCH_TICKS: i128 = 116_444_736_000_000_000;
    let ticks = i128::from(ticks);
    if ticks < WINDOWS_TO_UNIX_EPOCH_TICKS {
        return Err("TCP creation time predates Unix epoch".to_owned());
    }
    let unix_ticks = ticks - WINDOWS_TO_UNIX_EPOCH_TICKS;
    let nanos = unix_ticks
        .checked_mul(100)
        .ok_or_else(|| "TCP creation time overflow".to_owned())?;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|error| format!("invalid TCP creation time: {error}"))?
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format TCP creation time: {error}"))
}

#[cfg(windows)]
fn tcp4_rows() -> Result<Vec<MIB_TCPROW_OWNER_MODULE>, String> {
    tcp_table_rows::<MIB_TCPTABLE_OWNER_MODULE, MIB_TCPROW_OWNER_MODULE>(
        AF_INET,
        std::mem::offset_of!(MIB_TCPTABLE_OWNER_MODULE, table),
    )
}

#[cfg(windows)]
fn tcp6_rows() -> Result<Vec<MIB_TCP6ROW_OWNER_MODULE>, String> {
    tcp_table_rows::<MIB_TCP6TABLE_OWNER_MODULE, MIB_TCP6ROW_OWNER_MODULE>(
        AF_INET6,
        std::mem::offset_of!(MIB_TCP6TABLE_OWNER_MODULE, table),
    )
}

#[cfg(windows)]
fn tcp_table_rows<Table, Row>(address_family: u32, table_offset: usize) -> Result<Vec<Row>, String>
where
    Row: Copy,
{
    let _ = std::marker::PhantomData::<Table>;
    let mut size = 0_u32;
    let first = unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut size,
            1,
            address_family,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        )
    };
    if first != ERROR_INSUFFICIENT_BUFFER && first != 0 {
        return Err(format!(
            "GetExtendedTcpTable sizing failed for family {address_family}: {first}"
        ));
    }
    if size == 0 {
        return Ok(Vec::new());
    }

    let words = (size as usize).div_ceil(std::mem::size_of::<u64>());
    let mut buffer = vec![0_u64; words];
    let mut actual_size = size;
    let result = unsafe {
        GetExtendedTcpTable(
            buffer.as_mut_ptr().cast(),
            &mut actual_size,
            1,
            address_family,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        )
    };
    if result != 0 {
        return Err(format!(
            "GetExtendedTcpTable failed for family {address_family}: {result}"
        ));
    }
    if actual_size < 4 {
        return Err("TCP table was shorter than entry count".to_owned());
    }

    let base = buffer.as_ptr().cast::<u8>();
    let count = unsafe { *(base.cast::<u32>()) } as usize;
    let row_size = std::mem::size_of::<Row>();
    let required = table_offset
        .checked_add(
            count
                .checked_mul(row_size)
                .ok_or_else(|| "TCP table size overflow".to_owned())?,
        )
        .ok_or_else(|| "TCP table size overflow".to_owned())?;
    if required > actual_size as usize {
        return Err(format!(
            "TCP table truncated: required {required} bytes, received {actual_size}"
        ));
    }

    let rows = unsafe { std::slice::from_raw_parts(base.add(table_offset).cast::<Row>(), count) };
    Ok(rows.to_vec())
}

fn process_json(entry: &ProcessSnapshot) -> Value {
    json!({
        "ProcessId": entry.process_id,
        "ParentProcessId": entry.parent_process_id,
        "Name": entry.name,
        "ExecutablePath": entry.executable_path,
        "CommandLine": entry.command_line,
        "CreationDate": entry.creation_date,
    })
}

fn tcp_json(entry: &TcpSnapshot) -> Value {
    json!({
        "LocalAddress": entry.local_address,
        "LocalPort": entry.local_port,
        "RemoteAddress": entry.remote_address,
        "RemotePort": entry.remote_port,
        "State": entry.state,
        "OwningProcess": entry.owning_process,
        "CreationTime": entry.creation_time,
    })
}

fn write_powershell_shaped_json(path: &Path, values: Vec<Value>) -> Result<(), String> {
    let value = match values.len() {
        0 => Value::Array(Vec::new()),
        1 => values.into_iter().next().expect("one value"),
        _ => Value::Array(values),
    };
    let rendered = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(path, rendered.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ports_and_deduplicates() {
        assert_eq!(
            parse_ports("18781, 16110,18781").unwrap(),
            vec![18781, 16110]
        );
        assert!(parse_ports("0").is_err());
        assert!(parse_ports("70000").is_err());
    }

    #[test]
    fn repository_matching_preserves_legacy_rule() {
        let repo = r"c:\repo";
        assert!(process_matches_repository(
            r"C:\Repo\bin\worker.exe",
            "",
            repo
        ));
        assert!(process_matches_repository(
            r"C:\Python\python.exe",
            r#"python C:\Repo\tools\kaspa-gateway-fixture.py"#,
            repo
        ));
        assert!(!process_matches_repository(
            r"C:\Python\python.exe",
            r#"python C:\Repo\tools\fixture.py"#,
            repo
        ));
    }

    #[test]
    fn windows_argument_quoting_handles_spaces_and_quotes() {
        assert_eq!(quote_windows_argument("plain"), "plain");
        assert_eq!(quote_windows_argument("two words"), r#""two words""#);
        assert_eq!(quote_windows_argument(""), r#""""#);
    }

    #[test]
    fn mib_port_decoding_matches_network_byte_order() {
        assert_eq!(decode_mib_port(0x0000_5d49), 18781);
        assert_eq!(decode_mib_port(0x0000_ee3e), 16110);
    }

    #[test]
    fn powershell_json_shape_preserves_single_object_and_arrays() {
        let one = vec![json!({"a": 1})];
        let shaped = match one.len() {
            1 => one.into_iter().next().unwrap(),
            _ => unreachable!(),
        };
        assert!(shaped.is_object());

        let many = Value::Array(vec![json!({"a": 1}), json!({"a": 2})]);
        assert_eq!(many.as_array().unwrap().len(), 2);
    }
}
