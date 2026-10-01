use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

const DIFFICULTY_PRESETS: &[&str] = &[
    "1", "2", "4", "8", "16", "32", "64", "128", "256", "512", "1024", "2048", "4096", "8192",
    "16384", "32768", "65536",
];
const DIFFICULTY_DATALIST_ID: &str = "kgw-bridge-difficulty-presets-r16c";

fn escape_html_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            other => output.push(other),
        }
    }
    output
}

fn element_id(net: &str, name: &str) -> String {
    format!("bridge-{net}-{name}")
}
fn property(target: &JsValue, key: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

fn profile_string(profile: &JsValue, key: &str) -> String {
    crate::js_string_owned(&property(profile, key))
}

fn profile_bool(profile: &JsValue, key: &str) -> bool {
    crate::js_boolean(&property(profile, key))
}

fn span_suffix(span: &str) -> String {
    if span.is_empty() {
        String::new()
    } else {
        format!(" {span}")
    }
}

fn inline_toggle(net: &str, name: &str) -> String {
    crate::bridge_command_options::bridge_command_inline_toggle_r7(net.to_owned(), name.to_owned())
}
struct InputCard<'a> {
    net: &'a str,
    name: &'a str,
    label: &'a str,
    value: &'a str,
    placeholder: &'a str,
    span: &'a str,
    input_attrs: &'a str,
    toggle: &'a str,
}

fn card_input_html(card: InputCard<'_>) -> String {
    format!(
        "\n    <div class=\"bridge-v7-card{}\">\n      <span class=\"kgw-command-option-title-row-r8e\">\n        {}\n        <span class=\"kgw-command-option-title-text-r8e\">{}</span>\n      </span> <!-- KGW_BRIDGE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->\n      <input {} id=\"{}\" data-testid=\"kgw-bridge-field-{}-{}\" type=\"text\" value=\"{}\" placeholder=\"{}\">\n    </div>",
        span_suffix(card.span),
        card.toggle,
        escape_html_text(card.label),
        card.input_attrs,
        element_id(card.net, card.name),
        escape_html_text(card.net),
        escape_html_text(card.name),
        escape_html_text(card.value),
        escape_html_text(card.placeholder),
    )
}

fn card_select_html(
    net: &str,
    name: &str,
    label: &str,
    options: &[String],
    value: &str,
    span: &str,
    toggle: &str,
) -> String {
    let opts = options
        .iter()
        .map(|item| {
            let selected = if item == value { " selected" } else { "" };
            format!(
                "<option value=\"{}\"{}>{}</option>",
                escape_html_text(item),
                selected,
                escape_html_text(if item.is_empty() { "not set" } else { item })
            )
        })
        .collect::<String>();
    format!(
        "\n    <div class=\"bridge-v7-card{}\">\n      <span class=\"kgw-command-option-title-row-r8e\">\n        {}\n        <span class=\"kgw-command-option-title-text-r8e\">{}</span>\n      </span> <!-- KGW_BRIDGE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->\n      <select id=\"{}\" data-testid=\"kgw-bridge-field-{}-{}\">{}</select>\n    </div>",
        span_suffix(span),
        toggle,
        escape_html_text(label),
        element_id(net, name),
        escape_html_text(net),
        escape_html_text(name),
        opts,
    )
}

fn card_check_html(net: &str, name: &str, label: &str, checked: bool, span: &str) -> String {
    format!(
        "\n    <label class=\"bridge-v7-card check{}\">\n      <input id=\"{}\" data-testid=\"kgw-bridge-field-{}-{}\" type=\"checkbox\"{}>\n      <span>{}</span>\n    </label>",
        span_suffix(span),
        element_id(net, name),
        escape_html_text(net),
        escape_html_text(name),
        if checked { " checked" } else { "" },
        escape_html_text(label),
    )
}

fn card_input_owned(
    net: &str,
    name: &str,
    label: &str,
    value: &str,
    placeholder: &str,
    span: &str,
    input_attrs: &str,
) -> String {
    let toggle = inline_toggle(net, name);
    card_input_html(InputCard {
        net,
        name,
        label,
        value,
        placeholder,
        span,
        input_attrs,
        toggle: &toggle,
    })
}
fn card_select_owned(
    net: &str,
    name: &str,
    label: &str,
    options: &[&str],
    value: &str,
    span: &str,
) -> String {
    let options = options
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    card_select_html(
        net,
        name,
        label,
        &options,
        value,
        span,
        &inline_toggle(net, name),
    )
}

fn difficulty_input_attrs_text(name: &str) -> String {
    if ![
        "minShareDiff",
        "sharesPerMin",
        "instanceDiff",
        "instanceSharesPerMin",
    ]
    .contains(&name)
    {
        return String::new();
    }
    format!(
        "list=\"{DIFFICULTY_DATALIST_ID}\" inputmode=\"numeric\" autocomplete=\"off\" data-kgw-difficulty-preset-r16c=\"{}\"",
        escape_html_text(name)
    )
}

fn difficulty_datalist_html() -> String {
    let options = DIFFICULTY_PRESETS
        .iter()
        .map(|value| format!("<option value=\"{}\"></option>", escape_html_text(value)))
        .collect::<String>();
    format!("<datalist id=\"{DIFFICULTY_DATALIST_ID}\">{options}</datalist>")
}

#[wasm_bindgen(js_name = bridgeDifficultyPresetValuesR16C)]
pub fn bridge_difficulty_preset_values_r16c() -> Array {
    DIFFICULTY_PRESETS
        .iter()
        .map(|value| JsValue::from_str(value))
        .collect()
}

#[wasm_bindgen(js_name = bridgeDifficultyDatalistIdR16C)]
pub fn bridge_difficulty_datalist_id_r16c() -> String {
    DIFFICULTY_DATALIST_ID.to_owned()
}

fn inprocess_label(key: &str, fallback: &str) -> String {
    let value =
        crate::bridge_frontend_helpers::bridge_i18n_text_r41(key.to_owned(), fallback.to_owned());
    escape_html_text(&crate::js_string_owned(&value))
}

fn inprocess_toggle(net: &str, name: &str) -> String {
    crate::bridge_command_options::bridge_command_inline_toggle_r7(net.to_owned(), name.to_owned())
}

fn inprocess_toggle_input(
    net: &str,
    name: &str,
    i18n_key: &str,
    fallback: &str,
    input_type: &str,
    attrs: &str,
    value: &str,
) -> String {
    format!(
        r#"<div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        {}
        <span class="kgw-command-option-title-text-r8e" data-i18n="{}">{}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="{}" type="{}" {} value="{}">
    </div>"#,
        inprocess_toggle(net, name),
        escape_html_text(i18n_key),
        inprocess_label(i18n_key, fallback),
        element_id(net, name),
        escape_html_text(input_type),
        attrs,
        escape_html_text(value),
    )
}

fn inprocess_check(
    net: &str,
    name: &str,
    i18n_key: &str,
    fallback: &str,
    checked: bool,
    danger: bool,
) -> String {
    format!(
        r#"<label class="bridge-v7-card check{}">
            <input id="{}" type="checkbox"{}>
            <span data-i18n="{}">{}</span>
          </label>"#,
        if danger { " danger" } else { "" },
        element_id(net, name),
        if checked { " checked" } else { "" },
        escape_html_text(i18n_key),
        inprocess_label(i18n_key, fallback),
    )
}

#[wasm_bindgen(js_name = bridgeRenderInprocessNodeSettingsUi)]
pub fn bridge_render_inprocess_node_settings_ui(profile: JsValue) -> String {
    let net = profile_string(&profile, "key");
    if net.is_empty() {
        return String::new();
    }
    let testnet = profile_bool(&profile, "testnet");
    let netsuffix = profile_string(&profile, "netsuffix");
    let kaspad_port = profile_string(&profile, "kaspadPort");
    let testnet_args = if testnet {
        if netsuffix.is_empty() {
            "--testnet".to_owned()
        } else {
            format!("--testnet --netsuffix={}", escape_html_text(&netsuffix))
        }
    } else {
        "mainnet".to_owned()
    };

    let tabs = [
        ("basic", "bridge.inprocessNodeSettings.tab.basic", "Basic"),
        ("rpc", "bridge.inprocessNodeSettings.tab.rpc", "RPC"),
        (
            "storage",
            "bridge.inprocessNodeSettings.tab.storage",
            "Storage / Index",
        ),
        (
            "p2p",
            "bridge.inprocessNodeSettings.tab.p2p",
            "P2P / Network",
        ),
        (
            "perf",
            "bridge.inprocessNodeSettings.tab.performance",
            "Performance / Logs",
        ),
        (
            "advanced",
            "bridge.inprocessNodeSettings.tab.advanced",
            "Advanced",
        ),
        (
            "danger",
            "bridge.inprocessNodeSettings.tab.dangerous",
            "Dangerous",
        ),
    ];
    let tab_buttons = tabs
        .iter()
        .enumerate()
        .map(|(index, (key, i18n_key, fallback))| {
            format!(
                r#"<button type="button" class="bridge-v12d-node-tab{}" data-net="{}" data-bridge-inprocess-node-tab="{}">{}</button>"#,
                if index == 0 { " active" } else { "" },
                escape_html_text(&net),
                key,
                inprocess_label(i18n_key, fallback),
            )
        })
        .collect::<String>();

    let rpc_listen = inprocess_toggle_input(
        &net,
        "inprocessRpcListen",
        "bridge.inprocessNodeSettings.rpcListen",
        "--rpclisten",
        "text",
        "",
        &format!("127.0.0.1:{}", escape_html_text(&kaspad_port)),
    );
    let rpc_borsh = inprocess_toggle_input(
        &net,
        "inprocessRpcListenBorsh",
        "bridge.inprocessNodeSettings.rpcListenBorsh",
        "--rpclisten-borsh",
        "text",
        "",
        "",
    );
    let rpc_json = inprocess_toggle_input(
        &net,
        "inprocessRpcListenJson",
        "bridge.inprocessNodeSettings.rpcListenJson",
        "--rpclisten-json",
        "text",
        "",
        "",
    );
    let listen = inprocess_toggle_input(
        &net,
        "inprocessListen",
        "bridge.inprocessNodeSettings.listen",
        "--listen",
        "text",
        "",
        "",
    );
    let add_peer = inprocess_toggle_input(
        &net,
        "inprocessAddPeer",
        "bridge.inprocessNodeSettings.addPeer",
        "--addpeer",
        "text",
        "",
        "",
    );
    let connect = inprocess_toggle_input(
        &net,
        "inprocessConnect",
        "bridge.inprocessNodeSettings.connect",
        "--connect",
        "text",
        "",
        "",
    );
    let max_inpeers = inprocess_toggle_input(
        &net,
        "inprocessMaxInpeers",
        "bridge.inprocessNodeSettings.maxInpeers",
        "--maxinpeers",
        "number",
        r#"min="0" max="32" step="1""#,
        "32",
    );
    let outpeers = inprocess_toggle_input(
        &net,
        "inprocessOutpeers",
        "bridge.inprocessNodeSettings.outpeers",
        "--outpeers",
        "number",
        r#"min="0" max="8" step="1""#,
        "8",
    );
    let perf_interval = inprocess_toggle_input(
        &net,
        "inprocessPerfMetricsIntervalSec",
        "bridge.inprocessNodeSettings.perfMetricsIntervalSec",
        "--perf-metrics-interval-sec",
        "number",
        r#"min="1" step="1""#,
        "10",
    );
    let log_level = inprocess_toggle_input(
        &net,
        "inprocessLogLevel",
        "bridge.inprocessNodeSettings.logLevel",
        "--loglevel",
        "text",
        "",
        "info",
    );
    let ram_scale = inprocess_toggle_input(
        &net,
        "inprocessRamScale",
        "bridge.inprocessNodeSettings.ramScale",
        "--ram-scale",
        "number",
        r#"min="0.1" step="0.1""#,
        "1",
    );
    let configfile = inprocess_toggle_input(
        &net,
        "inprocessConfigfile",
        "bridge.inprocessNodeSettings.configfile",
        "--configfile",
        "text",
        r#"placeholder="unsupported: managed ownership""#,
        "",
    );
    let override_params = inprocess_toggle_input(
        &net,
        "inprocessOverrideParamsFile",
        "bridge.inprocessNodeSettings.overrideParamsFile",
        "--override-params-file",
        "text",
        "",
        "",
    );
    let async_threads = bridge_card_input(
        net.clone(),
        "inprocessAsyncThreads".to_owned(),
        "--async-threads".to_owned(),
        "16".to_owned(),
        String::new(),
        String::new(),
        String::new(),
    );

    format!(
        r#"
    <div class="bridge-v12d-inprocess-node-settings bridge-v12d-inprocess-inactive" data-net="{net}" data-bridge-inprocess-node-settings="{net}" data-kgw-owner="KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D">
      <div class="bridge-v12d-node-tabs">{tab_buttons}</div>

      <section class="bridge-v12d-node-panel active" data-net="{net}" data-bridge-inprocess-node-panel="basic">
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card span2">
            <span data-i18n="bridge.inprocessNodeSettings.appdir">{appdir_label}</span>
            <input id="{appdir_id}" type="text" value="" readonly>
          </div>
          <div class="bridge-v7-card span2">
            <span data-i18n="bridge.inprocessNodeSettings.testnet">{testnet_label}</span>
            <input id="{network_args_id}" type="text" value="{testnet_args}" readonly>
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="{net}" data-bridge-inprocess-node-panel="rpc" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          {rpc_listen}
          {rpc_borsh}
          {rpc_json}
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="{net}" data-bridge-inprocess-node-panel="storage" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          {utxo}
          {archival}
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="{net}" data-bridge-inprocess-node-panel="p2p" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          {listen}
          {add_peer}
          {connect}
          {disable_upnp}
          {max_inpeers}
          {outpeers}
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="{net}" data-bridge-inprocess-node-panel="perf" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          {async_threads}
          {perf_metrics}
          {perf_interval}
          {log_level}
          {ram_scale}
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="{net}" data-bridge-inprocess-node-panel="advanced" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          {configfile}
          {yes}
        </div>
      </section>

      <p class="kgw-danger-warning">Unsafe RPC exposes RPC beyond loopback. Unsynced mining bypasses synchronization. Enable only when you understand the risk.</p>
      <section class="bridge-v12d-node-panel" data-net="{net}" data-bridge-inprocess-node-panel="danger" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          {unsafe_rpc}
          {override_params}
          {devnet}
          {simnet}
          {unsynced}
        </div>
      </section>
    </div>"#,
        net = escape_html_text(&net),
        tab_buttons = tab_buttons,
        appdir_label = inprocess_label(
            "bridge.inprocessNodeSettings.appdir",
            "same --appdir / database path"
        ),
        appdir_id = element_id(&net, "inprocessAppdirMirror"),
        testnet_label = inprocess_label(
            "bridge.inprocessNodeSettings.testnet",
            "kaspad network args"
        ),
        network_args_id = element_id(&net, "inprocessNetworkArgs"),
        testnet_args = testnet_args,
        rpc_listen = rpc_listen,
        rpc_borsh = rpc_borsh,
        rpc_json = rpc_json,
        utxo = inprocess_check(
            &net,
            "inprocessUtxoIndex",
            "bridge.inprocessNodeSettings.utxoIndex",
            "--utxoindex",
            true,
            false
        ),
        archival = inprocess_check(
            &net,
            "inprocessArchival",
            "bridge.inprocessNodeSettings.archival",
            "--archival",
            false,
            false
        ),
        listen = listen,
        add_peer = add_peer,
        connect = connect,
        disable_upnp = inprocess_check(
            &net,
            "inprocessDisableUpnp",
            "bridge.inprocessNodeSettings.disableUpnp",
            "--disable-upnp",
            true,
            false
        ),
        max_inpeers = max_inpeers,
        outpeers = outpeers,
        async_threads = async_threads,
        perf_metrics = inprocess_check(
            &net,
            "inprocessPerfMetrics",
            "bridge.inprocessNodeSettings.perfMetrics",
            "--perf-metrics",
            true,
            false
        ),
        perf_interval = perf_interval,
        log_level = log_level,
        ram_scale = ram_scale,
        configfile = configfile,
        yes = inprocess_check(
            &net,
            "inprocessYes",
            "bridge.inprocessNodeSettings.yes",
            "--yes",
            true,
            false
        ),
        unsafe_rpc = inprocess_check(
            &net,
            "inprocessUnsafeRpc",
            "bridge.inprocessNodeSettings.unsafeRpc",
            "--unsaferpc",
            false,
            true
        ),
        override_params = override_params,
        devnet = inprocess_check(
            &net,
            "inprocessDevnet",
            "bridge.inprocessNodeSettings.devnet",
            "--devnet",
            false,
            true
        ),
        simnet = inprocess_check(
            &net,
            "inprocessSimnet",
            "bridge.inprocessNodeSettings.simnet",
            "--simnet",
            false,
            true
        ),
        unsynced = inprocess_check(
            &net,
            "inprocessEnableUnsyncedMining",
            "bridge.inprocessNodeSettings.enableUnsyncedMining",
            "--enable-unsynced-mining",
            false,
            true
        ),
    )
}

#[wasm_bindgen(js_name = bridgeDifficultyDatalistR16C)]
pub fn bridge_difficulty_datalist_r16c() -> String {
    difficulty_datalist_html()
}

#[wasm_bindgen(js_name = bridgeDifficultyInputAttrsR16C)]
pub fn bridge_difficulty_input_attrs_r16c(name: String) -> String {
    difficulty_input_attrs_text(&name)
}
#[wasm_bindgen(js_name = bridgeCardInput)]
pub fn bridge_card_input(
    net: String,
    name: String,
    label: String,
    value: String,
    placeholder: String,
    span: String,
    input_attrs: String,
) -> String {
    card_input_owned(
        &net,
        &name,
        &label,
        &value,
        &placeholder,
        &span,
        &input_attrs,
    )
}

#[wasm_bindgen(js_name = bridgeCardSelect)]
pub fn bridge_card_select(
    net: String,
    name: String,
    label: String,
    options: Array,
    value: String,
    span: String,
) -> String {
    let options = options
        .iter()
        .map(|item| crate::js_string_owned(&item))
        .collect::<Vec<_>>();
    card_select_html(
        &net,
        &name,
        &label,
        &options,
        &value,
        &span,
        &inline_toggle(&net, &name),
    )
}
#[wasm_bindgen(js_name = bridgeCardCheck)]
pub fn bridge_card_check(
    net: String,
    name: String,
    label: String,
    checked: bool,
    span: String,
) -> String {
    card_check_html(&net, &name, &label, checked, &span)
}

#[wasm_bindgen(js_name = bridgeRenderRuntime)]
pub fn bridge_render_runtime(profile: JsValue) -> String {
    let net = profile_string(&profile, "key");
    let testnet = profile_bool(&profile, "testnet");
    let kaspad_port = profile_string(&profile, "kaspadPort");
    let testnet_card = if net == "mainnet" {
        String::new()
    } else {
        card_check_html(&net, "testnet", "--testnet", testnet, "")
    };
    format!(
        "\n    <div class=\"bridge-v7-grid\">\n      {}\n      {}\n      {}\n      {}\n      {}\n      {}\n      {}\n      {}\n    </div>",
        card_select_owned(
            &net,
            "nodeMode",
            "--node-mode",
            &["external", "inprocess"],
            "external",
            ""
        ),
        testnet_card,
        card_input_owned(&net, "config", "--config", "", "config.yaml", "", ""),
        card_input_owned(&net, "appdir", "--appdir", "", "app dir", "", ""),
        card_input_owned(
            &net,
            "kaspadAddress",
            "--kaspad-address",
            &format!("127.0.0.1:{kaspad_port}"),
            "",
            "",
            ""
        ),
        card_input_owned(
            &net,
            "blockWaitTime",
            "--block-wait-time",
            "50ms",
            "",
            "",
            ""
        ),
        card_input_owned(
            &net,
            "healthCheckPort",
            "--health-check-port",
            "",
            "optional",
            "",
            ""
        ),
        card_input_owned(
            &net,
            "webDashboardPort",
            "--web-dashboard-port",
            "",
            ":3030",
            "",
            ""
        ),
    )
}

#[wasm_bindgen(js_name = bridgeRenderDifficulty)]
pub fn bridge_render_difficulty(profile: JsValue) -> String {
    let net = profile_string(&profile, "key");
    format!(
        "\n    <div class=\"bridge-v7-grid\">\n      {}\n      {}\n      {}\n      {}\n      {}\n      {}\n      {}\n    </div>",
        card_input_owned(
            &net,
            "minShareDiff",
            "--min-share-diff",
            "8192",
            "",
            "",
            &difficulty_input_attrs_text("minShareDiff")
        ),
        card_input_owned(
            &net,
            "sharesPerMin",
            "--shares-per-min",
            "30",
            "",
            "",
            &difficulty_input_attrs_text("sharesPerMin")
        ),
        card_select_owned(
            &net,
            "varDiff",
            "--var-diff",
            &["true", "false"],
            "true",
            ""
        ),
        card_select_owned(
            &net,
            "varDiffStats",
            "--var-diff-stats",
            &["true", "false"],
            "true",
            ""
        ),
        card_select_owned(
            &net,
            "pow2Clamp",
            "--pow2-clamp",
            &["true", "false"],
            "true",
            ""
        ),
        card_input_owned(&net, "extranonceSize", "--extranonce-size", "0", "", "", ""),
        card_input_owned(
            &net,
            "coinbaseTagSuffix",
            "--coinbase-tag-suffix",
            "",
            "optional",
            "span2",
            ""
        ),
    )
}
#[wasm_bindgen(js_name = bridgeRenderLogging)]
pub fn bridge_render_logging(profile: JsValue) -> String {
    let net = profile_string(&profile, "key");
    format!(
        "\n    <div class=\"bridge-v7-grid\">\n      {}\n      {}\n      {}\n    </div>",
        card_select_owned(
            &net,
            "printStats",
            "--print-stats",
            &["true", "false"],
            "true",
            ""
        ),
        card_select_owned(
            &net,
            "logToFile",
            "--log-to-file",
            &["true", "false"],
            "false",
            ""
        ),
        card_select_owned(
            &net,
            "approxGeoLookup",
            "--approximate-geo-lookup",
            &["not set", "true", "false"],
            "not set",
            "span2"
        ),
    )
}

#[wasm_bindgen(js_name = bridgeRenderPorts)]
pub fn bridge_render_ports(profile: JsValue) -> String {
    let net = profile_string(&profile, "key");
    format!(
        "\n    <div class=\"bridge-v7-grid\">\n      {}\n      {}\n    </div>",
        card_input_owned(
            &net,
            "stratumPort",
            "--stratum-port",
            &profile_string(&profile, "stratumPort"),
            "",
            "",
            ""
        ),
        card_input_owned(
            &net,
            "promPort",
            "--prom-port",
            &profile_string(&profile, "promPort"),
            "",
            "",
            ""
        ),
    )
}
#[wasm_bindgen(js_name = bridgeRenderCpuMiner)]
pub fn bridge_render_cpu_miner(profile: JsValue) -> String {
    let net = profile_string(&profile, "key");
    format!(
        "\n    <div class=\"bridge-v7-grid\">\n      {}\n      {}\n      {}\n      {}\n      {}\n    </div>",
        card_check_html(&net, "internalCpuMiner", "Enable CPU Mining", false, ""),
        card_input_owned(
            &net,
            "internalCpuMinerAddress",
            "Mining / Reward Address",
            "",
            "kaspatest:...",
            "span2",
            ""
        ),
        card_input_owned(
            &net,
            "internalCpuMinerThreads",
            "CPU Threads",
            "1",
            "threads",
            "",
            ""
        ),
        card_input_owned(
            &net,
            "internalCpuMinerThrottleMs",
            "Throttle (milliseconds)",
            "",
            "optional",
            "",
            ""
        ),
        card_input_owned(
            &net,
            "internalCpuMinerTemplatePollMs",
            "Template Poll Interval (milliseconds)",
            "",
            "optional",
            "span2",
            ""
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_contract_is_stable() {
        assert_eq!(DIFFICULTY_PRESETS.len(), 17);
        assert_eq!(DIFFICULTY_PRESETS[0], "1");
        assert_eq!(DIFFICULTY_PRESETS[16], "65536");
        assert_eq!(difficulty_input_attrs_text("other"), "");
        assert!(difficulty_input_attrs_text("minShareDiff").contains(DIFFICULTY_DATALIST_ID));
    }
    #[test]
    fn html_escape_and_cards_preserve_legacy_shape() {
        assert_eq!(
            escape_html_text("<a x=\"1\">&"),
            "&lt;a x=&quot;1&quot;&gt;&amp;"
        );
        let html = card_input_html(InputCard {
            net: "mainnet",
            name: "config",
            label: "--config",
            value: "",
            placeholder: "config.yaml",
            span: "",
            input_attrs: "",
            toggle: "<toggle>",
        });
        assert!(html.contains("<toggle>"));
        assert!(html.contains("id=\"bridge-mainnet-config\""));
        assert!(html.contains("data-testid=\"kgw-bridge-field-mainnet-config\""));
        assert!(html.contains("placeholder=\"config.yaml\""));
    }

    #[test]
    fn datalist_contains_all_presets_in_order() {
        let html = difficulty_datalist_html();
        let mut cursor = 0usize;
        for value in DIFFICULTY_PRESETS {
            let needle = format!("value=\"{value}\"");
            let found = html[cursor..].find(&needle).expect("preset missing");
            cursor += found + needle.len();
        }
    }
}
