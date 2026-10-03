use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use time::{Date, Month, OffsetDateTime};

const REVIEW_MARKER: &str = "Last automated review: **";

pub fn check(root: &Path, max_age_days: i64) -> Result<String, String> {
    if max_age_days < 1 {
        return Err("security-advisory-check: --max-age-days must be positive".to_owned());
    }

    let audit_path = root.join(".cargo/audit.toml");
    let deny_path = root.join("deny.toml");
    let report_path = root.join("SECURITY_ADVISORIES.md");
    for path in [&audit_path, &deny_path, &report_path] {
        if !path.is_file() {
            return Err(format!(
                "security-advisory-check: required policy file is missing: {}",
                display_relative(root, path)
            ));
        }
    }

    let audit_text = fs::read_to_string(&audit_path)
        .map_err(|error| format!("security-advisory-check: {error}"))?;
    let deny_text = fs::read_to_string(&deny_path)
        .map_err(|error| format!("security-advisory-check: {error}"))?;
    let report = fs::read_to_string(&report_path)
        .map_err(|error| format!("security-advisory-check: {error}"))?;

    let audit_ids = rustsec_ids(&audit_text);
    let deny_ids = rustsec_ids(&deny_text);
    let report_ids = rustsec_ids(&report);
    if audit_ids != deny_ids {
        return Err(format!(
            "security-advisory-check: cargo-audit and cargo-deny ignore sets differ: audit={audit_ids:?} deny={deny_ids:?}"
        ));
    }

    let undocumented: Vec<_> = audit_ids.difference(&report_ids).cloned().collect();
    if !undocumented.is_empty() {
        return Err(format!(
            "security-advisory-check: undocumented ignored RustSec IDs: {}",
            undocumented.join(", ")
        ));
    }

    let reviewed = review_date(&report)?;
    let today = OffsetDateTime::now_utc().date();
    let age_days = (today - reviewed).whole_days();
    if age_days < 0 {
        return Err(format!(
            "security-advisory-check: review date {reviewed} is in the future"
        ));
    }
    if age_days > max_age_days {
        return Err(format!(
            "security-advisory-check: security advisory review is {age_days} days old; maximum allowed age is {max_age_days} days"
        ));
    }

    Ok(format!(
        "security-advisory-check: PASS ({} managed RustSec IDs; review age {age_days} days)",
        audit_ids.len()
    ))
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn rustsec_ids(text: &str) -> BTreeSet<String> {
    let bytes = text.as_bytes();
    let mut ids = BTreeSet::new();
    const WIDTH: usize = 17;
    if bytes.len() < WIDTH {
        return ids;
    }
    for index in 0..=bytes.len() - WIDTH {
        let slice = &bytes[index..index + WIDTH];
        let Ok(candidate) = std::str::from_utf8(slice) else {
            continue;
        };
        if is_rustsec_id(candidate) {
            ids.insert(candidate.to_owned());
        }
    }
    ids
}

fn is_rustsec_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 17
        && &bytes[..8] == b"RUSTSEC-"
        && bytes[8..12].iter().all(u8::is_ascii_digit)
        && bytes[12] == b'-'
        && bytes[13..17].iter().all(u8::is_ascii_digit)
}

fn review_date(report: &str) -> Result<Date, String> {
    let start = report.find(REVIEW_MARKER).ok_or_else(|| {
        "security-advisory-check: SECURITY_ADVISORIES.md is missing 'Last automated review: **YYYY-MM-DD**'".to_owned()
    })? + REVIEW_MARKER.len();
    let tail = report.get(start..).unwrap_or_default();
    let raw = tail.get(..10).ok_or_else(|| {
        "security-advisory-check: invalid review date: incomplete YYYY-MM-DD".to_owned()
    })?;
    if !tail.get(10..).is_some_and(|rest| rest.starts_with("**")) {
        return Err("security-advisory-check: invalid review date marker".to_owned());
    }
    parse_date(raw)
}

fn parse_date(raw: &str) -> Result<Date, String> {
    let mut parts = raw.split('-');
    let year: i32 = parse_part(parts.next(), "year")?;
    let month: u8 = parse_part(parts.next(), "month")?;
    let day: u8 = parse_part(parts.next(), "day")?;
    if parts.next().is_some() {
        return Err("security-advisory-check: invalid review date: too many components".to_owned());
    }
    let month = Month::try_from(month)
        .map_err(|error| format!("security-advisory-check: invalid review date: {error}"))?;
    Date::from_calendar_date(year, month, day)
        .map_err(|error| format!("security-advisory-check: invalid review date: {error}"))
}

fn parse_part<T>(value: Option<&str>, label: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .ok_or_else(|| format!("security-advisory-check: invalid review date: missing {label}"))?
        .parse()
        .map_err(|error| format!("security-advisory-check: invalid review date: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_rustsec_ids() {
        let ids = rustsec_ids(
            "ignore = [\"RUSTSEC-2024-0001\", \"RUSTSEC-2024-0001\", \"RUSTSEC-2025-0123\"]",
        );
        assert_eq!(
            ids,
            BTreeSet::from([
                "RUSTSEC-2024-0001".to_owned(),
                "RUSTSEC-2025-0123".to_owned(),
            ])
        );
    }

    #[test]
    fn rejects_malformed_rustsec_ids() {
        assert!(!is_rustsec_id("RUSTSEC-2024-001"));
        assert!(!is_rustsec_id("RUSTSEC-20X4-0001"));
        assert!(!is_rustsec_id("RUSTSEC_2024-0001"));
    }

    #[test]
    fn parses_review_marker() {
        let date = review_date("Last automated review: **2026-09-23**").unwrap();
        assert_eq!(date.year(), 2026);
        assert_eq!(u8::from(date.month()), 9);
        assert_eq!(date.day(), 23);
    }

    #[test]
    fn invalid_review_marker_fails_closed() {
        assert!(review_date("Last automated review: 2026-09-23").is_err());
        assert!(review_date("Last automated review: **2026-99-99**").is_err());
    }
}
