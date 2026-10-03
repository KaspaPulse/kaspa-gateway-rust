use super::*;
use regex::Regex;
use xml::reader::{EventReader, XmlEvent};

const EXPECTED_TESTS: [&str; 5] = [
    "Mainnet Node regression copies only real child raw lines",
    "Testnet10 Node copies isolated Testnet10 child raw lines",
    "Mainnet Bridge copies only bridge child raw lines",
    "Testnet10 Bridge copies isolated Testnet10 bridge child raw lines",
    "Testnet13 stays disabled by default and policy blocks zero-touch launch",
];
fn report_files(directory: &Path, extension: &str) -> EvidenceResult<Vec<PathBuf>> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("Cannot list {}: {error}", directory.display()))?
        .map(|entry| entry.map_err(|error| error.to_string()))
        .collect::<EvidenceResult<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|value| value.to_string_lossy().eq_ignore_ascii_case(extension))
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    Ok(paths)
}
fn null_sequence(value: &Value) -> Vec<&Value> {
    if value.is_null() {
        vec![value]
    } else {
        sequence(value)
    }
}
fn json_counter(state: &Value, name: &str) -> EvidenceResult<i64> {
    Ok(integer(property(state, &[name]))?.unwrap_or(0))
}
fn parse_junit(content: &str, counters: &mut [i64; 4]) -> EvidenceResult<()> {
    // Saved reports do not require DTDs. Reject declarations instead of enabling
    // entity expansion or resolving local/network entities from untrusted input.
    if content.contains("<!DOCTYPE") {
        return Err("DTD declarations are not permitted in evidence XML".to_owned());
    }
    // The legacy DOM parser rejects malformed trailing XML before it counts
    // any suites. Validate first without retaining a large event tree.
    for event in EventReader::from_str(content) {
        event.map_err(|error| error.to_string())?;
    }
    for event in EventReader::from_str(content) {
        match event.map_err(|error| error.to_string())? {
            XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == "testsuite" && name.namespace.is_none() => {
                for (index, key) in ["tests", "failures", "errors", "skipped"]
                    .into_iter()
                    .enumerate()
                {
                    let raw = attributes
                        .iter()
                        .find(|attr| attr.name.local_name == key && attr.name.namespace.is_none())
                        .ok_or_else(|| format!("testsuite attribute '{key}' is missing"))?
                        .value
                        .trim();
                    let count = raw
                        .parse::<i32>()
                        .map_err(|_| format!("testsuite '{key}' is not a signed 32-bit integer"))?;
                    counters[index] = counters[index]
                        .checked_add(i64::from(count))
                        .ok_or_else(|| "JUnit counter overflow".to_owned())?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
pub fn wdio(artifact: &Path) -> EvidenceResult<Value> {
    let json_files = report_files(&artifact.join("json"), "json")?;
    let junit_files = report_files(&artifact.join("junit"), "xml")?;
    let mut errors = Vec::<String>::new();
    if json_files.is_empty() {
        errors.push("Missing WebdriverIO JSON report.".to_owned());
    }
    if junit_files.is_empty() {
        errors.push("Missing WebdriverIO JUnit report.".to_owned());
    }
    let mut names = Vec::<String>::new();
    let mut specs = Vec::<String>::new();
    let mut counts = [0i64; 3];
    for path in &json_files {
        let report = read_json(path)?;
        for spec in sequence(property(&report, &["specs"])) {
            let spec = text(spec);
            if !spec.trim().is_empty() {
                specs.push(spec);
            }
        }
        let state = property(&report, &["state"]);
        for (index, key) in ["passed", "failed", "skipped"].into_iter().enumerate() {
            counts[index] = counts[index]
                .checked_add(json_counter(state, key)?)
                .ok_or_else(|| "JSON counter overflow".to_owned())?;
        }
        for suite in null_sequence(property(&report, &["suites"])) {
            for test in null_sequence(property(suite, &["tests"])) {
                let name = field(test, &["name"]);
                let state = field(test, &["state"]);
                if !name.trim().is_empty() {
                    names.push(name.clone());
                }
                if !state.eq_ignore_ascii_case("passed") {
                    errors.push(format!(
                        "WebdriverIO JSON test '{name}' state was '{state}'."
                    ));
                }
            }
        }
    }
    let mut junit = [0i64; 4];
    for path in &junit_files {
        match read_text(path).and_then(|content| parse_junit(&content, &mut junit)) {
            Ok(()) => {}
            Err(error) => errors.push(format!(
                "Invalid WebdriverIO JUnit report '{}': {error}",
                path.display()
            )),
        }
    }
    if specs.len() != 1 {
        errors.push(format!(
            "Expected exactly one WebdriverIO spec, found {}.",
            specs.len()
        ));
    }
    let [passed, failed, skipped] = counts;
    if counts != [5, 0, 0] {
        errors.push(format!("WebdriverIO JSON state expected passed=5 failed=0 skipped=0, found passed={passed} failed={failed} skipped={skipped}."));
    }
    let [tests, failures, junit_errors, junit_skipped] = junit;
    if junit != [5, 0, 0, 0] {
        errors.push(format!("WebdriverIO JUnit expected tests=5 failures=0 errors=0 skipped=0, found tests={tests} failures={failures} errors={junit_errors} skipped={junit_skipped}."));
    }
    for expected in EXPECTED_TESTS {
        if !names.iter().any(|name| name.eq_ignore_ascii_case(expected)) {
            errors.push(format!(
                "WebdriverIO JSON report did not contain expected test '{expected}'."
            ));
        }
    }
    Ok(
        json!({"passed":errors.is_empty(),"errors":errors,"json_files":json_files,"junit_files":junit_files,"spec_count":specs.len(),
        "tests_passed":passed,"tests_failed":failed,"tests_skipped":skipped,"junit_tests":tests,"junit_failures":failures,
        "junit_errors":junit_errors,"junit_skipped":junit_skipped,"test_names":names}),
    )
}
pub fn policy(artifact: &Path) -> EvidenceResult<Value> {
    let case = artifact.join("cases").join("testnet13-policy");
    let policy_path = case.join("testnet13-policy.json");
    let paths = [
        policy_path.clone(),
        case.join("process-tree.json"),
        case.join("current-screenshot.png"),
        case.join("current-dom-state.json"),
        case.join("current-page-source.html"),
    ];
    let policy = read_json(&policy_path)?;
    let mut errors = Vec::<String>::new();
    if policy.is_null() {
        errors.push(format!(
            "Missing Testnet13 policy evidence: {}",
            policy_path.display()
        ));
    } else {
        if truthy(property(&policy, &["nodeEnabled"])) {
            errors.push(
                "Testnet13 node policy was enabled; expected disabled by default.".to_owned(),
            );
        }
        if truthy(property(&policy, &["bridgeEnabled"])) {
            errors.push(
                "Testnet13 bridge policy was enabled; expected disabled by default.".to_owned(),
            );
        }
        let blocked = property(&policy, &["startBlocked"]);
        if truthy(property(blocked, &["ok"])) {
            errors.push("Testnet13 start was not blocked.".to_owned());
        }
        let reason = field(blocked, &["error"]).to_lowercase();
        if !["experimental", "opt-in", "disabled", "policy"]
            .iter()
            .any(|word| reason.contains(word))
        {
            errors.push(
                "Testnet13 block reason did not mention experimental opt-in policy.".to_owned(),
            );
        }
        let status = field(&policy, &["status"]);
        if Regex::new(r"(?i)pid=\d+|running=true")
            .map_err(|error| error.to_string())?
            .is_match(&status)
        {
            errors.push("Testnet13 policy status indicates a runtime started.".to_owned());
        }
    }
    for path in &paths[1..] {
        if !path.is_file() {
            errors.push(format!(
                "Missing Testnet13 policy evidence file: {}",
                path.display()
            ));
        }
    }
    Ok(json!({"passed":errors.is_empty(),"errors":errors,"evidence_files":paths}))
}
pub fn recovery_files(artifact: &Path) -> EvidenceResult<Value> {
    let mut paths = Vec::new();
    for relative in [
        "zero-touch-process.stdout.log",
        "zero-touch-process.stderr.log",
        "wdio-run.log",
        "json/wdio-0-0.json",
        "junit/wdio-0-0.xml",
        "zero-touch-observations.json",
        "zero-touch-report.md",
        "zero-touch-script-summary.json",
        "zero-touch-result.json",
        "cases/testnet13-policy/testnet13-policy.json",
        "cases/testnet13-policy/process-tree.json",
        "cases/testnet13-policy/current-screenshot.png",
    ] {
        let path = relative
            .split('/')
            .fold(artifact.to_path_buf(), |path, part| path.join(part));
        if path.is_file() {
            paths.push(path);
        }
    }
    for stage in stage::required_stages()? {
        let case = artifact.join("cases").join(stage.slug);
        for name in [
            stage.status_file,
            "runtime-logs.json",
            "clipboard-capture.json",
            "clipboard.raw.txt",
            "process-tree.json",
            "port-state.json",
            "current-screenshot.png",
        ] {
            let path = case.join(name);
            if path.is_file() {
                paths.push(path);
            }
        }
    }
    Ok(json!(paths))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_xml_tail_never_contributes_partial_suite_counters() {
        let mut counters = [1, 2, 3, 4];
        assert!(
            parse_junit(
                r#"<testsuites><testsuite tests="5" failures="0" errors="0" skipped="0"/></wrong>"#,
                &mut counters
            )
            .is_err()
        );
        assert_eq!(counters, [1, 2, 3, 4]);
    }

    #[test]
    fn valid_junit_uses_real_xml_attributes_and_ignores_comments() {
        let mut counters = [0; 4];
        parse_junit(r#"<testsuites><!-- <testsuite tests="999"/> --><testsuite tests="5" failures="0" errors="0" skipped="0"><testcase name="A &amp; B"/></testsuite></testsuites>"#,&mut counters).unwrap();
        assert_eq!(counters, [5, 0, 0, 0]);
    }
    #[test]
    fn malformed_xml_dtd_and_invalid_counters_reject() {
        for input in [
            "<testsuite>",
            "<!DOCTYPE x><x/>",
            r#"<testsuite tests="NaN" failures="0" errors="0" skipped="0"/>"#,
        ] {
            assert!(parse_junit(input, &mut [0; 4]).is_err());
        }
    }
    #[test]
    fn namespace_does_not_impersonate_unqualified_testsuite() {
        let mut counters = [0; 4];
        parse_junit(
            r#"<testsuite xmlns="urn:other" tests="5" failures="0" errors="0" skipped="0"/>"#,
            &mut counters,
        )
        .unwrap();
        assert_eq!(counters, [0; 4]);
    }
}
