use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const HELP_KEYS: &[(&str, &str)] = &[
    ("appDir", "common.tooltip.appdir"),
    ("appdir", "common.tooltip.appdir"),
    ("inprocessAppdirMirror", "common.tooltip.appdir"),
    ("archival", "common.tooltip.archival"),
    ("inprocessArchival", "common.tooltip.archival"),
    ("asyncThreads", "common.tooltip.async.threads"),
    ("inprocessAsyncThreads", "common.tooltip.async.threads"),
    ("configFile", "common.tooltip.configfile"),
    ("config", "common.tooltip.configfile"),
    ("inprocessConfigfile", "common.tooltip.configfile"),
    ("connectEnabled", "common.tooltip.connect"),
    ("connectHost", "common.tooltip.connect"),
    ("connectPort", "common.tooltip.connect"),
    ("inprocessConnect", "common.tooltip.connect"),
    ("disableUpnp", "common.tooltip.disable.upnp"),
    ("inprocessDisableUpnp", "common.tooltip.disable.upnp"),
    (
        "enableUnsyncedMining",
        "common.tooltip.enable.unsynced.mining",
    ),
    (
        "inprocessEnableUnsyncedMining",
        "common.tooltip.enable.unsynced.mining",
    ),
    ("externalIpEnabled", "common.tooltip.externalip"),
    ("externalIpHost", "common.tooltip.externalip"),
    ("externalIpPort", "common.tooltip.externalip"),
    ("healthCheckPort", "common.tooltip.hcp"),
    ("listenEnabled", "common.tooltip.listen"),
    ("listenHost", "common.tooltip.listen"),
    ("listenPort", "common.tooltip.listen"),
    ("inprocessListen", "common.tooltip.listen"),
    ("maxInPeers", "common.tooltip.maxinpeers"),
    ("inprocessMaxInpeers", "common.tooltip.maxinpeers"),
    ("netsuffix", "common.tooltip.netsuffix"),
    ("noDnsSeed", "common.tooltip.nodnsseed"),
    ("outPeers", "common.tooltip.outpeers"),
    ("inprocessOutpeers", "common.tooltip.outpeers"),
    ("ramScale", "common.tooltip.ram.scale"),
    ("inprocessRamScale", "common.tooltip.ram.scale"),
    ("resetDb", "common.tooltip.reset.db"),
    ("retentionDays", "common.tooltip.retention.period.days"),
    ("sanity", "common.tooltip.sanity"),
    ("uaComment", "common.tooltip.uacomment"),
    ("yes", "common.tooltip.yes"),
    ("inprocessYes", "common.tooltip.yes"),
    ("rpcListenEnabled", "node.tooltip.rpclisten"),
    ("rpcListenHost", "node.tooltip.rpclisten"),
    ("rpcListenPort", "node.tooltip.rpclisten"),
    ("inprocessRpcListen", "node.tooltip.rpclisten"),
    ("rpcMaxClients", "node.tooltip.rpcmaxclients"),
    ("unsafeRpc", "node.tooltip.unsaferpc"),
    ("inprocessUnsafeRpc", "node.tooltip.unsaferpc"),
    ("utxoIndex", "node.tooltip.utxoindex"),
    ("inprocessUtxoIndex", "node.tooltip.utxoindex"),
    ("noGrpc", "node.tooltip.nogrpc"),
    ("stratumPort", "common.tooltip.ks.stratum"),
    ("promPort", "common.tooltip.ks.prom"),
    ("minShareDiff", "common.tooltip.ks.mindiff"),
    ("sharesPerMin", "common.tooltip.ks.sharespermin"),
    ("varDiff", "common.tooltip.ks.vardiff"),
    ("varDiffStats", "common.tooltip.ks.vardiffstats"),
    ("pow2Clamp", "common.tooltip.ks.pow2clamp"),
    ("extranonceSize", "common.tooltip.ks.extranonce"),
    ("printStats", "common.tooltip.ks.stats"),
    ("instanceDiff", "common.tooltip.ks.mindiff"),
    ("instanceSharesPerMin", "common.tooltip.ks.sharespermin"),
    ("instanceVarDiff", "common.tooltip.ks.vardiff"),
    ("instanceVarDiffStats", "common.tooltip.ks.vardiffstats"),
    ("instancePow2Clamp", "common.tooltip.ks.pow2clamp"),
    ("settingsRetryAttempts", "common.tooltip.retry.attempts"),
    ("settingsBackoffFactor", "common.tooltip.backoff.factor"),
    ("settingsMaxWorkers", "common.tooltip.max.workers"),
    ("settingsMaxPages", "common.tooltip.max.pages"),
    ("settingsPageDelay", "common.tooltip.page.delay"),
    (
        "settingsNetworkCacheHours",
        "common.tooltip.network.cache.hours",
    ),
];

const GLOBAL_HELP_IDS: &[&str] = &[
    "settingsCheckUpdates",
    "settingsStartWindows",
    "settingsLoggingLevel",
    "settingsDatabasePath",
    "settingsExportPath",
    "settingsLogPath",
    "settingsBackupPath",
    "settingsApiProfile",
    "settingsApiKey",
    "settingsApiDescription",
    "settingsApiBase",
    "settingsApiPath",
    "settingsApiTimeout",
    "settingsRetryAttempts",
    "settingsBackoffFactor",
    "settingsMaxWorkers",
    "settingsMaxPages",
    "settingsPageDelay",
    "settingsPriceCacheHours",
    "settingsNetworkCacheHours",
    "settingsEnableAutoRefresh",
    "settingsRefreshInterval",
    "settingsAddressName",
    "settingsAddressValue",
];

const HELP_FALLBACKS: &[(&str, &str)] = &[
    (
        "nodeMode",
        "Chooses whether the Bridge uses an independent Node or owns an embedded in-process Node. Change it only when the ownership model is intentional.",
    ),
    (
        "blockWaitTime",
        "Controls how long the Bridge waits for block-related work before retrying. Keep the default unless the runtime environment requires a deliberate adjustment.",
    ),
    (
        "logLevel",
        "Controls runtime log verbosity. Higher verbosity is useful for diagnosis but produces more output.",
    ),
    (
        "inprocessLogLevel",
        "Controls the embedded Node log verbosity. Use higher verbosity only when extra diagnostics are needed.",
    ),
    (
        "noLogFiles",
        "Disables file logging while keeping the live application log available.",
    ),
    (
        "perfMetrics",
        "Enables periodic performance metrics collection for the Node.",
    ),
    (
        "inprocessPerfMetrics",
        "Enables periodic performance metrics for the embedded Node.",
    ),
    (
        "perfMetricsInterval",
        "Controls how often performance metrics are sampled.",
    ),
    (
        "inprocessPerfMetricsIntervalSec",
        "Controls how often embedded Node performance metrics are sampled.",
    ),
    (
        "rocksDbPreset",
        "Selects the RocksDB tuning preset. Use a non-default preset only when it matches the storage hardware.",
    ),
    (
        "rocksDbCacheSize",
        "Sets the RocksDB cache budget. Larger values trade memory for database cache capacity.",
    ),
    (
        "rocksDbWalDir",
        "Overrides the RocksDB write-ahead-log directory. Use an absolute path on storage you intentionally manage.",
    ),
    (
        "maxTrackedAddresses",
        "Limits how many addresses may be tracked for UTXO-change notifications.",
    ),
    (
        "overrideParamsFile",
        "Provides advanced network-parameter overrides. Managed networks may reject this setting; use only for controlled testing.",
    ),
    (
        "inprocessOverrideParamsFile",
        "Provides advanced network-parameter overrides for the embedded Node. Use only for controlled testing.",
    ),
    (
        "inprocessDevnet",
        "Requests devnet behavior. KaspaGateway managed network tabs do not support this mode.",
    ),
    (
        "inprocessSimnet",
        "Requests simnet behavior. KaspaGateway managed network tabs do not support this mode.",
    ),
    (
        "rpcBorshEnabled",
        "Enables the Borsh RPC listener when a separate Borsh endpoint is intentionally required.",
    ),
    (
        "rpcBorshHost",
        "Host/interface for the optional Borsh RPC listener.",
    ),
    ("rpcBorshPort", "Port for the optional Borsh RPC listener."),
    (
        "rpcJsonEnabled",
        "Enables the JSON RPC listener when a separate JSON endpoint is intentionally required.",
    ),
    (
        "rpcJsonHost",
        "Host/interface for the optional JSON RPC listener.",
    ),
    ("rpcJsonPort", "Port for the optional JSON RPC listener."),
    (
        "addPeerEnabled",
        "Enables an explicit peer to add while retaining normal peer discovery.",
    ),
    ("addPeerHost", "Host of the explicit peer to add."),
    ("addPeerPort", "P2P port of the explicit peer to add."),
    (
        "inprocessAddPeer",
        "Adds an explicit peer to the embedded Node while retaining its normal peer policy.",
    ),
    (
        "internalCpuMiner",
        "Enables the embedded CPU-only miner for supported test networks. It does not enable an ASIC listener.",
    ),
    (
        "internalCpuMinerAddress",
        "Reward address used by the embedded CPU miner. Use an address for the selected test network.",
    ),
    (
        "internalCpuMinerThreads",
        "Number of CPU threads assigned to the embedded testnet miner.",
    ),
    (
        "internalCpuMinerThrottleMs",
        "Delay between CPU-mining work loops. Increase it to reduce CPU pressure.",
    ),
    (
        "internalCpuMinerTemplatePollMs",
        "How often the CPU miner requests a fresh mining template.",
    ),
    (
        "kaspadAddress",
        "RPC endpoint used by the Bridge when connecting to an independent Node.",
    ),
    (
        "coinbaseTagSuffix",
        "Optional suffix added to the mining coinbase tag.",
    ),
    (
        "logToFile",
        "Controls file logging for this runtime. Managed mode may keep this disabled while live logs remain available.",
    ),
    (
        "approxGeoLookup",
        "Controls approximate geographic lookup when the embedded runtime supports it.",
    ),
    (
        "instance",
        "Enables this Bridge instance and its instance-specific overrides.",
    ),
    (
        "instancePort",
        "Stratum port override for this Bridge instance.",
    ),
    (
        "instanceProm",
        "Prometheus port override for this Bridge instance.",
    ),
    (
        "instanceLogToFile",
        "Instance file logging is managed by KaspaGateway and may be unavailable.",
    ),
    (
        "settingsCheckUpdates",
        "Controls whether KaspaGateway checks for available updates at startup.",
    ),
    (
        "settingsStartWindows",
        "Controls whether the installed KaspaGateway application starts with Windows.",
    ),
    (
        "settingsLoggingLevel",
        "Controls application log verbosity. Higher levels provide more diagnostics and more output.",
    ),
    (
        "settingsDatabasePath",
        "Directory used for application database files. Leave it empty to use the managed application-data location.",
    ),
    (
        "settingsExportPath",
        "Default directory offered when exporting data.",
    ),
    (
        "settingsLogPath",
        "Directory used for application log files when file logging is enabled.",
    ),
    (
        "settingsBackupPath",
        "Directory used for database backups and restore points.",
    ),
    (
        "settingsApiProfile",
        "Selects the saved API endpoint profile used by the application.",
    ),
    (
        "settingsApiKey",
        "Identifier of the selected API endpoint entry.",
    ),
    (
        "settingsApiDescription",
        "Human-readable description of the selected API endpoint.",
    ),
    (
        "settingsApiBase",
        "Base URL used to compose the selected API endpoint.",
    ),
    ("settingsApiPath", "Path appended to the endpoint base URL."),
    (
        "settingsApiTimeout",
        "Maximum time to wait for an API request before it is treated as timed out.",
    ),
    (
        "settingsPriceCacheHours",
        "Hours to keep price data before requesting a fresh value.",
    ),
    (
        "settingsEnableAutoRefresh",
        "Enables periodic refresh for supported application data.",
    ),
    (
        "settingsRefreshInterval",
        "Seconds between automatic refresh attempts.",
    ),
    (
        "settingsAddressName",
        "Friendly local name stored with a saved Kaspa address.",
    ),
    (
        "settingsAddressValue",
        "Kaspa address stored in the local managed address list.",
    ),
];

fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}
fn global() -> JsValue {
    js_sys::global().into()
}
fn window() -> JsValue {
    property(&global(), "window")
}
fn document() -> JsValue {
    property(&global(), "document")
}
fn property(target: &JsValue, name: &str) -> JsValue {
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}
fn set_property(target: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(name), value).map(|_| ())
}
fn function(target: &JsValue, name: &str) -> Result<Function, JsValue> {
    property(target, name)
        .dyn_into::<Function>()
        .map_err(|_| js_error(format!("missing JS method {name}")))
}
fn call1(target: &JsValue, name: &str, arg: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)?.call1(target, arg)
}
fn call2(target: &JsValue, name: &str, a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)?.call2(target, a, b)
}
fn optional_call0(target: &JsValue, name: &str) {
    if let Ok(callback) = property(target, name).dyn_into::<Function>() {
        let _ = callback.call0(target);
    }
}
fn optional_call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    property(target, name)
        .dyn_into::<Function>()
        .ok()
        .and_then(|callback| callback.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}
fn is_present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}
fn raw_string(value: &JsValue) -> String {
    if value.is_null() || value.is_undefined() {
        String::new()
    } else {
        crate::js_string_owned(value)
    }
}
fn boolean(value: &JsValue) -> bool {
    value.as_bool().unwrap_or_else(|| {
        if value.is_null() || value.is_undefined() {
            false
        } else {
            crate::js_boolean(value)
        }
    })
}
fn object_is(a: &JsValue, b: &JsValue) -> bool {
    Object::is(a, b)
}
fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}
fn dataset_text(target: &JsValue, name: &str) -> String {
    raw_string(&property(&dataset(target), name))
}
fn set_dataset(target: &JsValue, name: &str, value: &str) -> Result<(), JsValue> {
    set_property(&dataset(target), name, &JsValue::from_str(value))
}
fn set_attr(target: &JsValue, name: &str, value: &str) -> Result<(), JsValue> {
    call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    )
    .map(|_| ())
}
fn get_attr(target: &JsValue, name: &str) -> String {
    raw_string(
        &call1(target, "getAttribute", &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED),
    )
}
fn has_attr(target: &JsValue, name: &str) -> bool {
    call1(target, "hasAttribute", &JsValue::from_str(name))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}
fn indexed_values(collection: &JsValue) -> Vec<JsValue> {
    let length = property(collection, "length").as_f64().unwrap_or(0.0) as u32;
    (0..length)
        .map(|index| {
            Reflect::get(collection, &JsValue::from_f64(index as f64)).unwrap_or(JsValue::UNDEFINED)
        })
        .filter(is_present)
        .collect()
}
fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    indexed_values(
        &call1(target, "querySelectorAll", &JsValue::from_str(selector))
            .unwrap_or(JsValue::UNDEFINED),
    )
}
fn children(target: &JsValue) -> Vec<JsValue> {
    indexed_values(&property(target, "children"))
}
fn class_contains(target: &JsValue, name: &str) -> bool {
    call1(
        &property(target, "classList"),
        "contains",
        &JsValue::from_str(name),
    )
    .ok()
    .and_then(|v| v.as_bool())
    .unwrap_or(false)
}
fn class_add(target: &JsValue, name: &str) -> Result<(), JsValue> {
    call1(
        &property(target, "classList"),
        "add",
        &JsValue::from_str(name),
    )
    .map(|_| ())
}
fn tag_name(target: &JsValue) -> String {
    raw_string(&property(target, "tagName")).to_uppercase()
}
fn text_content(target: &JsValue) -> String {
    raw_string(&property(target, "textContent"))
}
fn set_text(target: &JsValue, text: &str) -> Result<(), JsValue> {
    set_property(target, "textContent", &JsValue::from_str(text))
}
fn create_element(name: &str) -> Result<JsValue, JsValue> {
    call1(&document(), "createElement", &JsValue::from_str(name))
}
fn append(parent: &JsValue, child: &JsValue) -> Result<(), JsValue> {
    call1(parent, "appendChild", child).map(|_| ())
}
fn prepend(parent: &JsValue, child: &JsValue) -> Result<(), JsValue> {
    call1(parent, "prepend", child).map(|_| ())
}
fn replace_with(target: &JsValue, replacement: &JsValue) -> Result<(), JsValue> {
    call1(target, "replaceWith", replacement).map(|_| ())
}
fn closest(target: &JsValue, selector: &str) -> JsValue {
    optional_call1(target, "closest", &JsValue::from_str(selector))
}
fn matches(target: &JsValue, selector: &str) -> bool {
    optional_call1(target, "matches", &JsValue::from_str(selector))
        .as_bool()
        .unwrap_or(false)
}
fn contains(root: &JsValue, target: &JsValue) -> bool {
    call1(root, "contains", target)
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
fn copy_attributes(from: &JsValue, to: &JsValue) -> Result<(), JsValue> {
    let attrs = property(from, "attributes");
    let length = property(&attrs, "length").as_f64().unwrap_or(0.0) as u32;
    for index in 0..length {
        let attr =
            Reflect::get(&attrs, &JsValue::from_f64(index as f64)).unwrap_or(JsValue::UNDEFINED);
        if !is_present(&attr) {
            continue;
        }
        let name = raw_string(&property(&attr, "name"));
        if !name.is_empty() {
            set_attr(to, &name, &raw_string(&property(&attr, "value")))?;
        }
    }
    Ok(())
}

fn normalize_name_text(id: &str, bridge_instance_field: &str) -> String {
    if !bridge_instance_field.is_empty() {
        return bridge_instance_field.to_owned();
    }
    let mut name = id.to_owned();
    for prefix in [
        "node-mainnet-",
        "node-testnet10-",
        "node-testnet13-",
        "bridge-mainnet-",
        "bridge-testnet10-",
        "bridge-testnet13-",
    ] {
        if let Some(stripped) = name.strip_prefix(prefix) {
            name = stripped.to_owned();
            break;
        }
    }
    if let Some((head, tail)) = name.rsplit_once('-')
        && !tail.is_empty()
        && tail.chars().all(|ch| ch.is_ascii_digit())
    {
        name = head.to_owned();
    }
    name
}
fn normalize_setting_name(field: &JsValue) -> String {
    if !is_present(field) {
        return String::new();
    }
    normalize_name_text(
        &raw_string(&property(field, "id")),
        &dataset_text(field, "bridgeInstanceField"),
    )
}
fn ends_any(lower: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| lower.ends_with(suffix))
}
fn setting_kind_from(name: &str, field_type: &str, tag: &str, preview: bool) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if field_type.eq_ignore_ascii_case("checkbox") {
        return "boolean";
    }
    if preview {
        return "preview";
    }
    if tag.eq_ignore_ascii_case("SELECT") {
        return if lower.contains("log") {
            "log-level"
        } else {
            "enum"
        };
    }
    if lower.ends_with("port") || lower.ends_with("instanceprom") {
        return "port";
    }
    if lower.ends_with("ramscale") {
        return "ram";
    }
    if ends_any(
        &lower,
        &[
            "threads",
            "peers",
            "netsuffix",
            "extranoncesize",
            "maxtrackedaddresses",
        ],
    ) {
        return "integer";
    }
    if lower.ends_with("host")
        || lower.ends_with("kaspadaddress")
        || ends_any(&lower, &["rpclisten", "rpclistenborsh", "rpclistenjson"])
    {
        return "host";
    }
    if ends_any(
        &lower,
        &["time", "ms", "interval", "intervalsec", "days", "hours"],
    ) {
        return "duration";
    }
    if ["dir", "file", "path", "config"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        return "path";
    }
    if ends_any(
        &lower,
        &[
            "address",
            "comment",
            "suffix",
            "networkargs",
            "addpeer",
            "connect",
        ],
    ) {
        return "long";
    }
    "number"
}
fn setting_field_kind_text(field: &JsValue) -> &'static str {
    setting_kind_from(
        &normalize_setting_name(field),
        &raw_string(&property(field, "type")),
        &tag_name(field),
        matches(field, "[data-bridge-instance-preview]"),
    )
}
fn recommended_span(kind: &str) -> &'static str {
    match kind {
        "preview" => "full",
        "path" => "3",
        "long" => "2",
        _ => "1",
    }
}
fn lookup(items: &[(&str, &'static str)], key: &str) -> Option<&'static str> {
    items
        .iter()
        .find_map(|(name, value)| (*name == key).then_some(*value))
}
fn collapse_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn generic_help(name: &str, label: &str, kind: &str) -> String {
    let clean = collapse_ws(if label.is_empty() { name } else { label });
    let subject = if clean.is_empty() {
        "This setting"
    } else {
        clean.as_str()
    };
    let suffix = match kind {
        "boolean" => {
            " toggles this behavior. Change it only when the feature is intentionally required."
        }
        "port" => {
            " sets a network port. Change it only to resolve a port conflict or to use an intentionally custom endpoint."
        }
        "host" => {
            " sets the host or interface used by this endpoint. Keep loopback/private binding unless broader access is intentionally required."
        }
        "path" => {
            " selects a filesystem location. Use an absolute path that the current user can access."
        }
        "duration" => {
            " controls a timing interval. Keep the default unless a measured workload requires a deliberate change."
        }
        "integer" | "number" | "ram" => {
            " controls a numeric runtime limit or tuning value. Change it only when the workload requires a deliberate override."
        }
        "enum" | "log-level" => {
            " selects one of the supported runtime modes. Use the default unless another mode is intentionally required."
        }
        _ => {
            " configures this runtime option. Change it only when a non-default value is intentionally required."
        }
    };
    format!("{subject}{suffix}")
}
fn translate_help(key: &str, fallback: &str) -> String {
    if key.is_empty() {
        return fallback.to_owned();
    }
    let win = window();
    for api_name in ["kgwI18n", "KGWI18n", "KGW_I18N", "i18n"] {
        let api = property(&win, api_name);
        if !is_present(&api) {
            continue;
        }
        for method_name in ["t", "translate", "get"] {
            let Ok(method) = property(&api, method_name).dyn_into::<Function>() else {
                continue;
            };
            let Ok(value) =
                method.call2(&api, &JsValue::from_str(key), &JsValue::from_str(fallback))
            else {
                continue;
            };
            let text = raw_string(&value).trim().to_owned();
            if !text.is_empty() && text != key {
                return text;
            }
        }
    }
    fallback.to_owned()
}
struct HelpDescriptor {
    key: String,
    text: String,
    kind: String,
}
fn help_descriptor(field: &JsValue, label: &str) -> HelpDescriptor {
    let name = normalize_setting_name(field);
    let kind = setting_field_kind_text(field).to_owned();
    let id = raw_string(&property(field, "id"));
    let key = lookup(HELP_KEYS, &name)
        .or_else(|| lookup(HELP_KEYS, &id))
        .unwrap_or("")
        .to_owned();
    let fallback = lookup(HELP_FALLBACKS, &name)
        .or_else(|| lookup(HELP_FALLBACKS, &id))
        .map(str::to_owned)
        .unwrap_or_else(|| generic_help(&name, label, &kind));
    HelpDescriptor {
        key: key.clone(),
        text: translate_help(&key, &fallback),
        kind,
    }
}
fn convert_to_label(node: JsValue, field: &JsValue) -> Result<JsValue, JsValue> {
    let id = raw_string(&property(field, "id"));
    if !is_present(&node) || id.is_empty() {
        return Ok(node);
    }
    if tag_name(&node) == "LABEL" {
        if raw_string(&property(&node, "htmlFor")).is_empty() {
            set_property(&node, "htmlFor", &JsValue::from_str(&id))?;
        }
        return Ok(node);
    }
    let label = create_element("label")?;
    copy_attributes(&node, &label)?;
    set_property(&label, "htmlFor", &JsValue::from_str(&id))?;
    set_property(&label, "innerHTML", &property(&node, "innerHTML"))?;
    replace_with(&node, &label)?;
    Ok(label)
}
fn replace_card_label(card: JsValue) -> Result<JsValue, JsValue> {
    if tag_name(&card) != "LABEL" {
        return Ok(card);
    }
    let replacement = create_element("div")?;
    copy_attributes(&card, &replacement)?;
    loop {
        let child = property(&card, "firstChild");
        if !is_present(&child) {
            break;
        }
        append(&replacement, &child)?;
    }
    replace_with(&card, &replacement)?;
    Ok(replacement)
}
struct AtomicParts {
    card: JsValue,
    header: JsValue,
    label: JsValue,
}
fn ensure_atomic_structure(initial_card: JsValue, field: &JsValue) -> Result<AtomicParts, JsValue> {
    let card = replace_card_label(initial_card)?;
    class_add(&card, "kgw-setting-field")?;
    let kind = setting_field_kind_text(field);
    set_dataset(&card, "settingKind", kind)?;
    set_dataset(&card, "settingSpan", recommended_span(kind))?;
    let layout = if raw_string(&property(field, "type")).eq_ignore_ascii_case("checkbox") {
        "check"
    } else if ["path", "long", "preview"].contains(&kind) {
        "wide"
    } else {
        "compact"
    };
    set_dataset(&card, "settingLayout", layout)?;

    if dataset_text(&card, "kgwAtomicSetting") != "v2" {
        let title_row = query(&card, ":scope > .kgw-command-option-title-row-r8e");
        let mut label = if is_present(&title_row) {
            query(&title_row, ".kgw-command-option-title-text-r8e")
        } else {
            JsValue::UNDEFINED
        };
        if !is_present(&label) {
            label = children(&card)
                .into_iter()
                .find(|node| {
                    !object_is(node, field)
                        && tag_name(node) == "SPAN"
                        && !class_contains(node, "kgw-setting-state")
                })
                .unwrap_or(JsValue::UNDEFINED);
        }
        label = convert_to_label(label, field)?;
        let header = create_element("div")?;
        set_property(
            &header,
            "className",
            &JsValue::from_str("kgw-setting-field-header"),
        )?;
        if is_present(&title_row) {
            append(&header, &title_row)?;
        } else if is_present(&label) {
            append(&header, &label)?;
        }
        let control = create_element("div")?;
        set_property(
            &control,
            "className",
            &JsValue::from_str("kgw-setting-control"),
        )?;
        append(&control, field)?;
        let state = create_element("div")?;
        set_property(&state, "className", &JsValue::from_str("kgw-setting-state"))?;
        set_attr(&state, "aria-live", "polite")?;
        set_text(&state, &dataset_text(&card, "kgwSettingState"))?;
        prepend(&card, &header)?;
        call2(
            &header,
            "insertAdjacentElement",
            &JsValue::from_str("afterend"),
            &control,
        )?;
        call2(
            &control,
            "insertAdjacentElement",
            &JsValue::from_str("afterend"),
            &state,
        )?;
        set_dataset(&card, "kgwAtomicSetting", "v2")?;
    }
    let header = query(&card, ":scope > .kgw-setting-field-header");
    let preferred = query(&header, "label[for]");
    let label = if is_present(&preferred) {
        preferred
    } else {
        query(&header, ".kgw-command-option-title-text-r8e")
    };
    let state = query(&card, ":scope > .kgw-setting-state");
    let wanted = dataset_text(&card, "kgwSettingState");
    if is_present(&state) && text_content(&state) != wanted {
        set_text(&state, &wanted)?;
    }
    Ok(AtomicParts {
        card,
        header,
        label,
    })
}
fn ensure_help(
    card: &JsValue,
    header: &JsValue,
    label: &JsValue,
    field: &JsValue,
) -> Result<(), JsValue> {
    let id = raw_string(&property(field, "id"));
    if !is_present(header) || id.is_empty() {
        return Ok(());
    }
    let raw_label = text_content(label);
    let label_text = collapse_ws(if raw_label.is_empty() {
        id.as_str()
    } else {
        raw_label.as_str()
    });
    let descriptor = help_descriptor(field, &label_text);
    let help_id = format!("{id}-help");
    let mut trigger = query(header, ":scope > .kgw-field-help-trigger");
    if !is_present(&trigger) {
        trigger = create_element("button")?;
        set_property(&trigger, "type", &JsValue::from_str("button"))?;
        set_property(
            &trigger,
            "className",
            &JsValue::from_str("kgw-field-help-trigger"),
        )?;
        set_text(&trigger, "?")?;
        set_attr(&trigger, "aria-expanded", "false")?;
        append(header, &trigger)?;
    }
    set_attr(&trigger, "aria-controls", &help_id)?;
    set_attr(&trigger, "aria-label", &format!("Help: {label_text}"))?;
    set_dataset(&trigger, "helpKey", &descriptor.key)?;
    let mut popover = query(card, ":scope > .kgw-field-help-popover");
    if !is_present(&popover) {
        popover = create_element("div")?;
        set_property(
            &popover,
            "className",
            &JsValue::from_str("kgw-field-help-popover"),
        )?;
        set_property(&popover, "hidden", &JsValue::TRUE)?;
        set_attr(&popover, "role", "tooltip")?;
        append(card, &popover)?;
    }
    set_property(&popover, "id", &JsValue::from_str(&help_id))?;
    set_dataset(&popover, "helpKey", &descriptor.key)?;
    set_text(&popover, &descriptor.text)?;
    let mut described = get_attr(field, "aria-describedby")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !described.iter().any(|value| value == &help_id) {
        described.push(help_id);
    }
    set_attr(field, "aria-describedby", &described.join(" "))?;
    if [
        "port", "host", "integer", "ram", "duration", "path", "long", "number", "preview",
    ]
    .contains(&descriptor.kind.as_str())
    {
        set_property(field, "dir", &JsValue::from_str("ltr"))?;
    }
    Ok(())
}
fn close_help(root: &JsValue, except: Option<&JsValue>) {
    for button in query_all(root, ".kgw-field-help-trigger[aria-expanded=\"true\"]") {
        if except.is_some_and(|candidate| object_is(&button, candidate)) {
            continue;
        }
        let _ = set_attr(&button, "aria-expanded", "false");
        let id = get_attr(&button, "aria-controls");
        if id.is_empty() {
            continue;
        }
        let panel = query(root, &format!("#{id}"));
        if is_present(&panel) {
            let _ = set_property(&panel, "hidden", &JsValue::TRUE);
        }
    }
}
fn open_help(root: &JsValue, button: &JsValue) {
    close_help(root, Some(button));
    let _ = set_attr(button, "aria-expanded", "true");
    let id = get_attr(button, "aria-controls");
    if id.is_empty() {
        return;
    }
    let panel = query(root, &format!("#{id}"));
    if is_present(&panel) {
        let _ = set_property(&panel, "hidden", &JsValue::FALSE);
    }
}
fn event_target(event: &JsValue) -> JsValue {
    property(event, "target")
}
fn prevent(event: &JsValue) {
    optional_call0(event, "preventDefault");
    optional_call0(event, "stopPropagation");
}
fn install_help_behavior(root: &JsValue) -> Result<(), JsValue> {
    if !is_present(root) || dataset_text(root, "kgwSettingsHelpInstalled") == "v2" {
        return Ok(());
    }
    set_dataset(root, "kgwSettingsHelpInstalled", "v2")?;
    let click_root = root.clone();
    let click = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let button = closest(&event_target(&event), ".kgw-field-help-trigger");
        if !is_present(&button) || !contains(&click_root, &button) {
            close_help(&click_root, None);
            return;
        }
        prevent(&event);
        open_help(&click_root, &button);
    });
    call2(
        root,
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref(),
    )?;
    click.forget();

    let focus_root = root.clone();
    let focus = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let button = closest(&event_target(&event), ".kgw-field-help-trigger");
        if is_present(&button) && contains(&focus_root, &button) {
            open_help(&focus_root, &button);
        }
    });
    call2(
        root,
        "addEventListener",
        &JsValue::from_str("focusin"),
        focus.as_ref(),
    )?;
    focus.forget();

    let key_root = root.clone();
    let keydown = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let trigger = closest(&event_target(&event), ".kgw-field-help-trigger");
        let key = raw_string(&property(&event, "key"));
        if is_present(&trigger)
            && contains(&key_root, &trigger)
            && matches!(key.as_str(), "Enter" | " ")
        {
            prevent(&event);
            open_help(&key_root, &trigger);
            return;
        }
        if key != "Escape" {
            return;
        }
        let button = query(&key_root, ".kgw-field-help-trigger[aria-expanded=\"true\"]");
        if !is_present(&button) {
            return;
        }
        optional_call0(&event, "preventDefault");
        close_help(&key_root, None);
        optional_call0(&button, "focus");
    });
    call2(
        root,
        "addEventListener",
        &JsValue::from_str("keydown"),
        keydown.as_ref(),
    )?;
    keydown.forget();
    Ok(())
}

fn escape_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
    }
    output
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct SettingsGroup {
    section: String,
    name: String,
    label: String,
    html: String,
}
fn groups_from_js(groups: JsValue) -> Vec<SettingsGroup> {
    Array::from(&groups)
        .iter()
        .filter_map(|value| {
            let row = Array::from(&value);
            (row.length() >= 4).then(|| SettingsGroup {
                section: raw_string(&row.get(0)),
                name: raw_string(&row.get(1)),
                label: raw_string(&row.get(2)),
                html: raw_string(&row.get(3)),
            })
        })
        .collect()
}
fn render_settings_tabs_from(scope: &str, net: &str, groups: &[SettingsGroup]) -> String {
    let prefix = if scope == "node" {
        "node-v6"
    } else {
        "bridge-v7"
    };
    let sections = ["general", "advanced"];
    let tabs = sections.iter().enumerate().map(|(index, section)| format!(
        "<button type=\"button\" role=\"tab\" class=\"{prefix}-section-tab{}\" data-net=\"{net}\" data-{scope}-section-tab=\"{section}\" aria-selected=\"{}\">{}</button>",
        if index == 0 { " active" } else { "" }, index == 0, if index == 0 { "General" } else { "Advanced" }
    )).collect::<String>();
    let panels = sections.iter().enumerate().map(|(section_index, section)| {
        let entries = groups.iter().filter(|group| group.section == *section).collect::<Vec<_>>();
        let key = format!("{scope}-{net}-{section}");
        let nav = entries.iter().enumerate().map(|(index, group)| format!(
            "<button type=\"button\" role=\"tab\" id=\"{key}-{}-tab\" data-settings-tab=\"{}\" aria-controls=\"{key}-{}-panel\" aria-selected=\"{}\" tabindex=\"{}\">{}</button>",
            group.name, group.name, group.name, index == 0, if index == 0 { "0" } else { "-1" }, escape_html(&group.label)
        )).collect::<String>();
        let body = entries.iter().enumerate().map(|(index, group)| format!(
            "<section role=\"tabpanel\" id=\"{key}-{}-panel\" aria-labelledby=\"{key}-{}-tab\" data-settings-panel=\"{}\"{}>{}</section>",
            group.name, group.name, group.name, if index == 0 { "" } else { " hidden" }, group.html
        )).collect::<String>();
        format!(
            "<section class=\"{prefix}-section{}\" data-net=\"{net}\" data-{scope}-section-panel=\"{section}\"{}><div data-settings-tab-group><div class=\"kgw-settings-subtabs\" role=\"tablist\" aria-label=\"{}\">{nav}</div>{body}</div></section>",
            if section_index == 0 { " active" } else { "" },
            if section_index == 0 { "" } else { " hidden" },
            escape_html(&format!("{section} settings"))
        )
    }).collect::<String>();
    format!(
        "<div class=\"{prefix}-section-tabs\" role=\"tablist\" aria-label=\"Settings level\">{tabs}</div><div class=\"{prefix}-sections kgw-settings-sections\">{panels}</div>"
    )
}
fn strip_managed_suffixes(text: &str) -> String {
    let mut current = text.to_owned();
    loop {
        let lower = current.to_ascii_lowercase();
        let managed = lower.find(" (managed");
        let unsupported = lower.find(" (unsupported");
        let start = match (managed, unsupported) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            _ => None,
        };
        let Some(start) = start else {
            break;
        };
        let Some(relative_end) = current[start..].find(')') else {
            break;
        };
        current.replace_range(start..=start + relative_end, "");
    }
    current
}
fn decorate_settings_fields(root: &JsValue) -> Result<(), JsValue> {
    if !is_present(root) {
        return Ok(());
    }
    for initial_card in query_all(root, ".node-v6-card, .bridge-v7-card") {
        let field = query(
            &initial_card,
            "input[id], select[id], textarea[id], input[data-bridge-instance-preview]",
        );
        if !is_present(&field) {
            continue;
        }
        let parts = ensure_atomic_structure(initial_card, &field)?;
        if matches(&field, "[data-bridge-instance-preview][readonly]") {
            set_dataset(&parts.card, "settingExempt", "generated-readonly-preview")?;
            set_property(&field, "dir", &JsValue::from_str("ltr"))?;
            continue;
        }
        ensure_help(&parts.card, &parts.header, &parts.label, &field)?;
        let clean_label = query(&parts.header, ".kgw-command-option-title-text-r8e");
        if is_present(&clean_label) && !has_attr(&clean_label, "data-i18n") {
            if raw_string(&property(&clean_label, "title")).is_empty() {
                set_property(
                    &clean_label,
                    "title",
                    &JsValue::from_str(&text_content(&clean_label)),
                )?;
            }
            set_text(
                &clean_label,
                &strip_managed_suffixes(&text_content(&clean_label)),
            )?;
        }
    }
    install_help_behavior(root)
}
fn select_internal_tab(button: &JsValue) {
    if !is_present(button) {
        return;
    }
    let group = closest(button, "[data-settings-tab-group]");
    if !is_present(&group) {
        return;
    }
    for tab in query_all(
        &group,
        ":scope > .kgw-settings-subtabs > [data-settings-tab]",
    ) {
        let selected = object_is(&tab, button);
        let _ = set_attr(&tab, "aria-selected", &selected.to_string());
        let _ = set_property(
            &tab,
            "tabIndex",
            &JsValue::from_f64(if selected { 0.0 } else { -1.0 }),
        );
    }
    let selected_name = dataset_text(button, "settingsTab");
    for panel in query_all(&group, ":scope > [data-settings-panel]") {
        let hidden = dataset_text(&panel, "settingsPanel") != selected_name;
        let _ = set_property(&panel, "hidden", &JsValue::from_bool(hidden));
    }
}
fn reveal_settings_field(field: &JsValue) {
    if !is_present(field) {
        return;
    }
    let internal = closest(field, "[data-settings-panel]");
    let group = property(&internal, "parentElement");
    if is_present(&internal) && is_present(&group) {
        let name = dataset_text(&internal, "settingsPanel");
        select_internal_tab(&query(&group, &format!("[data-settings-tab=\"{name}\"]")));
    }
    let nested = closest(field, "[data-bridge-inprocess-node-panel]");
    if is_present(&nested) {
        let owner = closest(&nested, "[data-bridge-inprocess-node-settings]");
        let name = dataset_text(&nested, "bridgeInprocessNodePanel");
        let button = query(
            &owner,
            &format!("[data-bridge-inprocess-node-tab=\"{name}\"]"),
        );
        if is_present(&button) {
            optional_call0(&button, "click");
        }
    }
}
fn install_settings_layout(root: &JsValue) -> Result<(), JsValue> {
    decorate_settings_fields(root)?;
    if !is_present(root) || !dataset_text(root, "settingsLayoutInstalled").is_empty() {
        return Ok(());
    }
    set_dataset(root, "settingsLayoutInstalled", "true")?;
    let click_root = root.clone();
    let click = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = event_target(&event);
        let tab = closest(&target, "[data-settings-tab]");
        if is_present(&tab) && contains(&click_root, &tab) {
            select_internal_tab(&tab);
        }
        let button = closest(&target, "[data-settings-preview-toggle]");
        if is_present(&button) {
            let preview = closest(&button, ".kgw-effective-preview");
            let body = query(&preview, ".kgw-preview-body");
            if is_present(&body) {
                let next_hidden = !boolean(&property(&body, "hidden"));
                let _ = set_property(&body, "hidden", &JsValue::from_bool(next_hidden));
                let _ = set_attr(&button, "aria-expanded", &(!next_hidden).to_string());
                let _ = set_text(&button, if next_hidden { "Expand" } else { "Collapse" });
            }
        }
    });
    call2(
        root,
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref(),
    )?;
    click.forget();
    let keydown = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = event_target(&event);
        let tab = closest(&target, "[data-settings-tab]");
        let key = raw_string(&property(&event, "key"));
        if !is_present(&tab) || !matches!(key.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End")
        {
            return;
        }
        let parent = property(&tab, "parentElement");
        let tabs = query_all(&parent, "[data-settings-tab]");
        if tabs.is_empty() {
            return;
        }
        let index = tabs
            .iter()
            .position(|candidate| object_is(candidate, &tab))
            .unwrap_or(0);
        let next = match key.as_str() {
            "Home" => 0,
            "End" => tabs.len() - 1,
            "ArrowRight" => (index + 1) % tabs.len(),
            _ => (index + tabs.len() - 1) % tabs.len(),
        };
        optional_call0(&event, "preventDefault");
        select_internal_tab(&tabs[next]);
        optional_call0(&tabs[next], "focus");
    });
    call2(
        root,
        "addEventListener",
        &JsValue::from_str("keydown"),
        keydown.as_ref(),
    )?;
    keydown.forget();
    let language_root = root.clone();
    let language = Closure::<dyn FnMut(JsValue)>::new(move |_| {
        let _ = decorate_settings_fields(&language_root);
    });
    call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("kgw:language-applied"),
        language.as_ref(),
    )?;
    language.forget();
    Ok(())
}
fn global_help_descriptor(field: &JsValue, label: &str) -> HelpDescriptor {
    let id = raw_string(&property(field, "id"));
    let key = lookup(HELP_KEYS, &id).unwrap_or("").to_owned();
    let kind = setting_field_kind_text(field).to_owned();
    let fallback = lookup(HELP_FALLBACKS, &id)
        .map(str::to_owned)
        .unwrap_or_else(|| generic_help(&id, label, &kind));
    HelpDescriptor {
        key: key.clone(),
        text: translate_help(&key, &fallback),
        kind,
    }
}
fn decorate_global_field(field: &JsValue, label: &JsValue) -> Result<(), JsValue> {
    let id = raw_string(&property(field, "id"));
    if !is_present(field)
        || !is_present(label)
        || boolean(&property(field, "readOnly"))
        || !GLOBAL_HELP_IDS.contains(&id.as_str())
    {
        return Ok(());
    }
    let label_text = if tag_name(label) == "LEGEND" {
        indexed_values(&property(label, "childNodes"))
            .into_iter()
            .filter(|node| property(node, "nodeType").as_f64() == Some(3.0))
            .map(|node| text_content(&node))
            .collect::<String>()
            .trim()
            .to_owned()
    } else {
        text_content(label).trim().to_owned()
    };
    let wrapper = if tag_name(label) == "LEGEND" {
        class_add(label, "kgw-global-field-header")?;
        if raw_string(&property(label, "id")).is_empty() {
            set_property(label, "id", &JsValue::from_str(&format!("{id}-label")))?;
        }
        if get_attr(field, "aria-labelledby").is_empty() {
            set_attr(
                field,
                "aria-labelledby",
                &raw_string(&property(label, "id")),
            )?;
        }
        label.clone()
    } else {
        let parent = property(label, "parentElement");
        if is_present(&parent) && class_contains(&parent, "kgw-global-field-header") {
            parent
        } else {
            let wrapper = create_element("span")?;
            set_property(
                &wrapper,
                "className",
                &JsValue::from_str("kgw-global-field-header"),
            )?;
            replace_with(label, &wrapper)?;
            append(&wrapper, label)?;
            wrapper
        }
    };
    let descriptor = global_help_descriptor(field, &label_text);
    let help_id = format!("{id}-help");
    let mut trigger = query(&wrapper, ":scope > .kgw-field-help-trigger");
    if !is_present(&trigger) {
        trigger = create_element("button")?;
        set_property(&trigger, "type", &JsValue::from_str("button"))?;
        set_property(
            &trigger,
            "className",
            &JsValue::from_str("kgw-field-help-trigger"),
        )?;
        set_text(&trigger, "?")?;
        set_attr(&trigger, "aria-expanded", "false")?;
        append(&wrapper, &trigger)?;
    }
    set_attr(&trigger, "aria-controls", &help_id)?;
    let trigger_label = if label_text.is_empty() {
        id.as_str()
    } else {
        label_text.as_str()
    };
    set_attr(&trigger, "aria-label", &format!("Help: {trigger_label}"))?;
    set_dataset(&trigger, "helpKey", &descriptor.key)?;
    let mut popover = query(&wrapper, ":scope > .kgw-field-help-popover");
    if !is_present(&popover) {
        popover = create_element("span")?;
        set_property(
            &popover,
            "className",
            &JsValue::from_str("kgw-field-help-popover"),
        )?;
        set_attr(&popover, "role", "tooltip")?;
        set_property(&popover, "hidden", &JsValue::TRUE)?;
        append(&wrapper, &popover)?;
    }
    set_property(&popover, "id", &JsValue::from_str(&help_id))?;
    set_dataset(&popover, "helpKey", &descriptor.key)?;
    set_text(&popover, &descriptor.text)?;
    let mut described = get_attr(field, "aria-describedby")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !described.iter().any(|value| value == &help_id) {
        described.push(help_id);
    }
    set_attr(field, "aria-describedby", &described.join(" "))?;
    Ok(())
}
fn decorate_global_settings(root: &JsValue) -> Result<(), JsValue> {
    if !is_present(root) {
        return Ok(());
    }
    for id in GLOBAL_HELP_IDS {
        let field = query(root, &format!("#{id}"));
        if !is_present(&field) || boolean(&property(&field, "readOnly")) {
            continue;
        }
        let mut label = query(root, &format!("label[for=\"{id}\"]"));
        if !is_present(&label) {
            let containing = closest(&field, "label");
            if is_present(&containing) {
                label = containing;
            }
        }
        if !is_present(&label) {
            let fieldset = closest(&field, "fieldset");
            label = query(&fieldset, ":scope > legend");
        }
        decorate_global_field(&field, &label)?;
    }
    for dot in query_all(root, ".info-dot") {
        optional_call0(&dot, "remove");
    }
    install_help_behavior(root)
}
fn install_global_settings_layout(root: &JsValue) -> Result<(), JsValue> {
    decorate_global_settings(root)?;
    if !is_present(root) || dataset_text(root, "kgwGlobalSettingsLayoutInstalled") == "v2" {
        return Ok(());
    }
    set_dataset(root, "kgwGlobalSettingsLayoutInstalled", "v2")?;
    let language_root = root.clone();
    let language = Closure::<dyn FnMut(JsValue)>::new(move |_| {
        let _ = decorate_global_settings(&language_root);
    });
    call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("kgw:language-applied"),
        language.as_ref(),
    )?;
    language.forget();
    Ok(())
}

#[wasm_bindgen(js_name = settingsLayoutHelpI18nKeys)]
pub fn help_i18n_keys() -> JsValue {
    let object = Object::new();
    for (name, value) in HELP_KEYS {
        let _ = Reflect::set(&object, &JsValue::from_str(name), &JsValue::from_str(value));
    }
    object.into()
}
#[wasm_bindgen(js_name = settingsLayoutGlobalHelpIds)]
pub fn global_help_ids() -> Array {
    GLOBAL_HELP_IDS
        .iter()
        .map(|value| JsValue::from_str(value))
        .collect()
}
#[wasm_bindgen(js_name = settingsLayoutFieldKind)]
pub fn field_kind(field: JsValue) -> String {
    setting_field_kind_text(&field).to_owned()
}
#[wasm_bindgen(js_name = settingsLayoutRenderTabs)]
pub fn render_tabs(scope: String, net: String, groups: JsValue) -> String {
    render_settings_tabs_from(&scope, &net, &groups_from_js(groups))
}
#[wasm_bindgen(js_name = settingsLayoutDecorateFields)]
pub fn decorate_fields(root: JsValue) -> Result<(), JsValue> {
    decorate_settings_fields(&root)
}
#[wasm_bindgen(js_name = settingsLayoutSetFieldState)]
pub fn set_field_state(field: JsValue, message: String) -> Result<(), JsValue> {
    let card = closest(&field, ".kgw-setting-field, .node-v6-card, .bridge-v7-card");
    if !is_present(&card) {
        return Ok(());
    }
    set_dataset(&card, "kgwSettingState", &message)?;
    let state = query(&card, ":scope > .kgw-setting-state");
    if is_present(&state) {
        set_text(&state, &dataset_text(&card, "kgwSettingState"))?;
    }
    Ok(())
}
#[wasm_bindgen(js_name = settingsLayoutRevealField)]
pub fn reveal_field(field: JsValue) {
    reveal_settings_field(&field);
}
#[wasm_bindgen(js_name = settingsLayoutInstall)]
pub fn install_layout(root: JsValue) -> Result<(), JsValue> {
    install_settings_layout(&root)
}
#[wasm_bindgen(js_name = settingsLayoutDecorateGlobal)]
pub fn decorate_global(root: JsValue) -> Result<(), JsValue> {
    decorate_global_settings(&root)
}
#[wasm_bindgen(js_name = settingsLayoutInstallGlobal)]
pub fn install_global(root: JsValue) -> Result<(), JsValue> {
    install_global_settings_layout(&root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_kind_rules_match_legacy_contract() {
        assert_eq!(
            setting_kind_from("foo", "checkbox", "INPUT", false),
            "boolean"
        );
        assert_eq!(
            setting_kind_from("logLevel", "", "SELECT", false),
            "log-level"
        );
        assert_eq!(
            setting_kind_from("networkMode", "", "SELECT", false),
            "enum"
        );
        assert_eq!(
            setting_kind_from("rpcListenPort", "", "INPUT", false),
            "port"
        );
        assert_eq!(
            setting_kind_from("rocksDbWalDir", "", "INPUT", false),
            "path"
        );
        assert_eq!(
            setting_kind_from("internalCpuMinerThreads", "", "INPUT", false),
            "integer"
        );
        assert_eq!(setting_kind_from("connectHost", "", "INPUT", false), "host");
        assert_eq!(
            setting_kind_from("refreshIntervalSec", "", "INPUT", false),
            "duration"
        );
        assert_eq!(
            setting_kind_from("coinbaseTagSuffix", "", "INPUT", false),
            "long"
        );
        assert_eq!(
            setting_kind_from("difficulty", "", "INPUT", false),
            "number"
        );
        assert_eq!(setting_kind_from("anything", "", "INPUT", true), "preview");
    }
    #[test]
    fn setting_name_normalization_matches_network_and_instance_rules() {
        assert_eq!(
            normalize_name_text("node-mainnet-rpcListenPort", ""),
            "rpcListenPort"
        );
        assert_eq!(
            normalize_name_text("bridge-testnet10-instancePort-7", ""),
            "instancePort"
        );
        assert_eq!(
            normalize_name_text("ignored", "instanceProm"),
            "instanceProm"
        );
    }
    #[test]
    fn render_tabs_escapes_labels_but_preserves_owned_html() {
        let groups = vec![
            SettingsGroup {
                section: "general".into(),
                name: "basic".into(),
                label: "<Basic & safe>".into(),
                html: "<input id=\"x\">".into(),
            },
            SettingsGroup {
                section: "advanced".into(),
                name: "expert".into(),
                label: "Advanced".into(),
                html: "<select id=\"y\"></select>".into(),
            },
        ];
        let html = render_settings_tabs_from("node", "mainnet", &groups);
        assert!(html.contains("node-v6-section-tab active"));
        assert!(html.contains("&lt;Basic &amp; safe&gt;"));
        assert!(html.contains("<input id=\"x\">"));
        assert!(html.contains("node-mainnet-advanced-expert-panel"));
        assert!(html.contains(" hidden"));
    }
    #[test]
    fn help_metadata_and_suffix_cleanup_are_owned_by_rust() {
        assert_eq!(
            lookup(HELP_KEYS, "rpcListenPort"),
            Some("node.tooltip.rpclisten")
        );
        assert!(GLOBAL_HELP_IDS.contains(&"settingsAddressValue"));
        assert_eq!(
            strip_managed_suffixes("RPC (managed by owner) label (unsupported here)"),
            "RPC label"
        );
    }
}
