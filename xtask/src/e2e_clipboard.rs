use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use std::mem;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{GlobalFree, HGLOBAL};
#[cfg(windows)]
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    GMEM_MOVEABLE, GMEM_ZEROINIT, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};

const CF_TEXT: u32 = 1;
const CF_OEMTEXT: u32 = 7;
const CF_UNICODETEXT: u32 = 13;
const CF_LOCALE: u32 = 16;

fn is_text_compatible_clipboard_format(format: u32) -> bool {
    matches!(format, CF_TEXT | CF_OEMTEXT | CF_UNICODETEXT | CF_LOCALE)
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Options {
    action: String,
    value: String,
    output_path: Option<PathBuf>,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let options = parse_options(args)?;
    run(&options)
}

fn parse_options(args: &mut impl Iterator<Item = String>) -> Result<Options, String> {
    let action = args
        .next()
        .ok_or_else(|| "e2e-clipboard requires read, write, or preflight".to_owned())?;
    if !matches!(action.as_str(), "read" | "write" | "preflight") {
        return Err(format!("unknown e2e-clipboard action: {action}"));
    }

    let mut value = String::new();
    let mut output_path = None;
    while let Some(flag) = args.next() {
        let argument = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--value" => value = argument,
            "--output-path" => output_path = Some(PathBuf::from(argument)),
            _ => return Err(format!("unknown e2e-clipboard option: {flag}")),
        }
    }

    Ok(Options {
        action,
        value,
        output_path,
    })
}

fn run(options: &Options) -> Result<String, String> {
    #[cfg(windows)]
    {
        let result = match options.action.as_str() {
            "write" => {
                write_text_after_preflight(&options.value)?;
                write_metadata(&options.value)
            }
            "read" => {
                let text = read_clipboard_text()?;
                if let Some(path) = &options.output_path {
                    write_utf8_no_bom(path, &text)?;
                }
                read_metadata(&text, options.output_path.as_deref())
            }
            "preflight" => clipboard_text_write_preflight()?,
            _ => unreachable!(),
        };
        serde_json::to_string(&result)
            .map_err(|error| format!("clipboard metadata serialization failed: {error}"))
    }

    #[cfg(not(windows))]
    {
        let _ = options;
        Err("e2e-clipboard requires Windows".to_owned())
    }
}

#[cfg(windows)]
pub(crate) fn read_snapshot_metadata() -> Result<Value, String> {
    let text = read_clipboard_text()?;
    Ok(read_metadata(&text, None))
}

fn write_metadata(text: &str) -> Value {
    json!({
        "mode": "write",
        "character_count": utf16_len(text),
        "line_count": line_count(text),
        "sha256": sha256_hex(text),
    })
}

fn read_metadata(text: &str, output_path: Option<&Path>) -> Value {
    json!({
        "mode": "read",
        "output_path": output_path.map_or_else(String::new, |path| path.to_string_lossy().into_owned()),
        "character_count": utf16_len(text),
        "line_count": line_count(text),
        "sha256": sha256_hex(text),
    })
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

fn line_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .count()
}

fn sha256_hex(text: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(text.as_bytes());
    format!("{:x}", digest.finalize())
}

fn write_utf8_no_bom(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(path, text.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}
#[cfg(windows)]
struct ClipboardGuard;

#[cfg(windows)]
impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

#[cfg(windows)]
struct OwnedGlobal(HGLOBAL);

#[cfg(windows)]
impl Drop for OwnedGlobal {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = GlobalFree(self.0);
            }
        }
    }
}

#[cfg(windows)]
fn open_clipboard_with_retry(label: &str) -> Result<ClipboardGuard, String> {
    let mut last_error = String::new();
    for attempt in 1_u64..=20 {
        if unsafe { OpenClipboard(ptr::null_mut()) } != 0 {
            return Ok(ClipboardGuard);
        }
        last_error = std::io::Error::last_os_error().to_string();
        thread::sleep(Duration::from_millis(50 * attempt));
    }
    Err(format!("{label} failed after retries: {last_error}"))
}

#[cfg(windows)]
pub(crate) fn clipboard_format_ids() -> Result<Vec<u32>, String> {
    let _clipboard = open_clipboard_with_retry("Clipboard format preflight")?;
    let mut formats = Vec::new();
    let mut current = 0_u32;
    loop {
        current = unsafe { EnumClipboardFormats(current) };
        if current == 0 {
            break;
        }
        formats.push(current);
    }
    formats.sort_unstable();
    formats.dedup();
    Ok(formats)
}

#[cfg(windows)]
pub(crate) fn clipboard_text_write_preflight() -> Result<Value, String> {
    let formats = clipboard_format_ids()?;
    let non_text: Vec<u32> = formats
        .iter()
        .copied()
        .filter(|format| !is_text_compatible_clipboard_format(*format))
        .collect();
    let safe = non_text.is_empty();
    Ok(json!({
        "safe_for_text_only_write": safe,
        "formats": formats,
        "non_text_formats": non_text,
        "reason": if safe {
            "clipboard contains only text-compatible formats"
        } else {
            "text-only EmptyClipboard/SetClipboardData would destroy non-text formats"
        }
    }))
}

#[cfg(windows)]
pub(crate) fn ensure_text_only_write_safe() -> Result<Value, String> {
    let result = clipboard_text_write_preflight()?;
    if result["safe_for_text_only_write"].as_bool() == Some(true) {
        Ok(result)
    } else {
        Err(format!(
            "clipboard mutation blocked: {} non-text formats={}",
            result["reason"]
                .as_str()
                .unwrap_or("unsafe clipboard formats"),
            result["non_text_formats"]
        ))
    }
}

#[cfg(windows)]
pub(crate) fn write_text_after_preflight(text: &str) -> Result<(), String> {
    ensure_text_only_write_safe()?;
    write_clipboard_text(text)
}

#[cfg(windows)]
pub(crate) fn read_text() -> Result<String, String> {
    read_clipboard_text()
}

#[cfg(windows)]
fn read_clipboard_text() -> Result<String, String> {
    let _clipboard = open_clipboard_with_retry("Get-Clipboard")?;
    if unsafe { IsClipboardFormatAvailable(CF_UNICODETEXT) } == 0 {
        return Ok(String::new());
    }

    let handle = unsafe { GetClipboardData(CF_UNICODETEXT) };
    if handle.is_null() {
        return Err(format!(
            "Get-Clipboard failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let global = handle as HGLOBAL;
    let size = unsafe { GlobalSize(global) };
    if size < 2 {
        return Err("Get-Clipboard returned an invalid CF_UNICODETEXT allocation".to_owned());
    }
    let pointer = unsafe { GlobalLock(global) };
    if pointer.is_null() {
        return Err(format!(
            "Get-Clipboard GlobalLock failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    let units = unsafe { std::slice::from_raw_parts(pointer.cast::<u16>(), size / 2) };
    let length = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(units.len());
    let text = String::from_utf16_lossy(&units[..length]);
    unsafe {
        let _ = GlobalUnlock(global);
    }
    Ok(text)
}

#[cfg(windows)]
fn write_clipboard_text(text: &str) -> Result<(), String> {
    let mut utf16: Vec<u16> = text.encode_utf16().collect();
    utf16.push(0);
    let bytes = utf16.len() * std::mem::size_of::<u16>();

    let global = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes) };
    if global.is_null() {
        return Err(format!(
            "Set-Clipboard GlobalAlloc failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let owned = OwnedGlobal(global);

    let pointer = unsafe { GlobalLock(owned.0) };
    if pointer.is_null() {
        return Err(format!(
            "Set-Clipboard GlobalLock failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    unsafe {
        ptr::copy_nonoverlapping(utf16.as_ptr(), pointer.cast::<u16>(), utf16.len());
        let _ = GlobalUnlock(owned.0);
    }

    let _clipboard = open_clipboard_with_retry("Set-Clipboard")?;
    if unsafe { EmptyClipboard() } == 0 {
        return Err(format!(
            "Set-Clipboard EmptyClipboard failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    let transferred = unsafe { SetClipboardData(CF_UNICODETEXT, owned.0) };
    if transferred.is_null() {
        return Err(format!(
            "Set-Clipboard SetClipboardData failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    mem::forget(owned);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_compatible_formats_include_windows_locale_metadata() {
        for format in [CF_TEXT, CF_OEMTEXT, CF_UNICODETEXT, CF_LOCALE] {
            assert!(is_text_compatible_clipboard_format(format));
        }
        assert!(!is_text_compatible_clipboard_format(2));
        assert!(!is_text_compatible_clipboard_format(15));
        assert!(!is_text_compatible_clipboard_format(49_301));
    }

    #[test]
    fn metadata_matches_powershell_utf16_and_line_contract() {
        let text = "alpha\r\nbeta\rgamma\n🦀";
        let write = write_metadata(text);
        assert_eq!(write["mode"], "write");
        assert_eq!(write["character_count"], 20);
        assert_eq!(write["line_count"], 4);
        assert_eq!(
            write["sha256"],
            "0e094fe6a4dee49e14649ed3f23717cc0ef15acb2079c42b35893b4eda5f91da"
        );
    }

    #[test]
    fn empty_metadata_matches_legacy_contract() {
        let write = write_metadata("");
        assert_eq!(write["character_count"], 0);
        assert_eq!(write["line_count"], 0);
        assert_eq!(
            write["sha256"],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let read = read_metadata("", None);
        assert_eq!(read["mode"], "read");
        assert_eq!(read["output_path"], "");
        assert_eq!(read["line_count"], 0);
    }

    #[test]
    fn read_metadata_preserves_requested_output_path() {
        let path = Path::new(r"C:\kgw evidence\clipboard.raw.txt");
        let read = read_metadata("one\ntwo", Some(path));
        assert_eq!(read["output_path"], path.to_string_lossy().as_ref());
        assert_eq!(read["character_count"], 7);
        assert_eq!(read["line_count"], 2);
    }

    #[test]
    fn parser_accepts_legacy_surface_and_rejects_unknowns() {
        let mut args = vec![
            "write".to_owned(),
            "--value".to_owned(),
            "sentinel".to_owned(),
        ]
        .into_iter();
        assert_eq!(
            parse_options(&mut args).unwrap(),
            Options {
                action: "write".to_owned(),
                value: "sentinel".to_owned(),
                output_path: None,
            }
        );

        let mut bad = vec!["read".to_owned(), "--unknown".to_owned(), "x".to_owned()].into_iter();
        assert!(parse_options(&mut bad).is_err());
    }

    #[test]
    fn utf8_writer_never_adds_a_bom() {
        let path =
            std::env::temp_dir().join(format!("kgw-e2e-clipboard-{}.txt", std::process::id()));
        let _ = fs::remove_file(&path);
        write_utf8_no_bom(&path, "مرحبا\nKaspa").unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert_eq!(bytes, "مرحبا\nKaspa".as_bytes());
        let _ = fs::remove_file(path);
    }
}
