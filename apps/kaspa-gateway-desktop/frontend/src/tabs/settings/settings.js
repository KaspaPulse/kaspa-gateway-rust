const SETTINGS_STORAGE_KEY = "kgw-settings-python-exact-state";

function kgwSettingsToWesternDigits(value) {
  return String(value ?? "")
    .replace(/[٠-٩]/g, (d) => "٠١٢٣٤٥٦٧٨٩".indexOf(d))
    .replace(/[۰-۹]/g, (d) => "۰۱۲۳۴۵۶۷۸۹".indexOf(d));
}

function kgwSettingsNormalizeNumericFields() {
  const ids = [
    "settingsApiTimeout",
    "settingsRetryAttempts",
    "settingsBackoffFactor",
    "settingsMaxWorkers",
    "settingsMaxPages",
    "settingsPageDelay",
    "settingsPriceCacheHours",
    "settingsNetworkCacheHours",
    "settingsRefreshInterval"
  ];

  ids.forEach((id) => {
    const node = q(`#${CSS.escape(id)}`);
    if (!node) return;

    node.type = "text";
    node.inputMode = "decimal";
    node.dir = "ltr";
    node.value = kgwSettingsToWesternDigits(node.value);

    if (node.dataset.westernDigitBound === "true") return;
    node.dataset.westernDigitBound = "true";

    node.addEventListener("input", () => {
      const normalized = kgwSettingsToWesternDigits(node.value);
      if (node.value !== normalized) {
        const pos = node.selectionStart;
        node.value = normalized;
        try {
          node.setSelectionRange(pos, pos);
        } catch (_) {}
      }
    });
  });
}


function settingsLogger() {
  if (typeof window.kgwCreateLogger === "function") {
    return window.kgwCreateLogger("settings");
  }

  return {
    log: () => {},
    warn: () => {},
    error: () => {}
  };
}

function root() {
  return document.getElementById("settings");
}

function q(selector) {
  return root()?.querySelector(selector) || null;
}

function qa(selector) {
  return Array.from(root()?.querySelectorAll(selector) || []);
}


// KGW_SETTINGS_UI_TRACE_PATCH_R48B3
function kgwSettingsUiTraceR48B3(action, phase, details) {
  try {
    const safeAction = String(action || "settings-ui");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "settings",
      net: "ui",
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_SETTINGS_UI_TRACE_PATCH_R48B3",
        owner: "settings-existing-ui-owners",
        action: safeAction,
        phase: safePhase,
        details: safeDetails
      })
    };
    const tauri = window.__TAURI__;
    const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
      ? tauri.core.invoke.bind(tauri.core)
      : tauri && typeof tauri.invoke === "function"
        ? tauri.invoke.bind(tauri)
        : window.__TAURI_INVOKE__;
    if (typeof invoke === "function") {
      invoke("kgw_frontend_button_trace_v1", args).catch(function () {});
    }
  } catch (_) {}
}
function setSaveEnabled(enabled) {
  const save = q("#settingsSaveSettings");
  if (save) save.disabled = !enabled;
}

function markDirty() {
  setSaveEnabled(true);
}

function activateOuter(tab) {
  qa("[data-settings-tab]").forEach((button) => {
    const active = button.dataset.settingsTab === tab;
    button.classList.toggle("active", active);
    button.setAttribute("aria-selected", String(active));
  });

  qa("[data-settings-panel]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.settingsPanel === tab);
  });
}

function activateInner(tab) {
  qa("[data-settings-inner-tab]").forEach((button) => {
    const active = button.dataset.settingsInnerTab === tab;
    button.classList.toggle("active", active);
    button.setAttribute("aria-selected", String(active));
  });

  qa("[data-settings-inner-panel]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.settingsInnerPanel === tab);
  });
}

function updateSelectAll(masterSelector, childSelector) {
  const master = q(masterSelector);
  const children = qa(childSelector);

  if (!master || !children.length) return;

  master.checked = children.every((node) => node.checked);
  master.indeterminate = !master.checked && children.some((node) => node.checked);
}

function bindSelectAll(masterSelector, childSelector) {
  const master = q(masterSelector);
  const children = qa(childSelector);

  if (!master) return;

  if (master.dataset.bound !== "true") {
    master.dataset.bound = "true";
    master.addEventListener("change", (event) => {
      children.forEach((node) => {
        node.checked = master.checked;
      });
      master.indeterminate = false;
      kgwSettingsUiTraceR48B3("settings-select-all", "r48b3-select-all-master-change", {
        trusted: Boolean(event && event.isTrusted),
        masterSelector: String(masterSelector || ""),
        childSelector: String(childSelector || ""),
        checked: Boolean(master.checked)
      });
      markDirty();
    });
  }

  children.forEach((node) => {
    if (node.dataset.bound === "true") return;
    node.dataset.bound = "true";
    node.addEventListener("change", (event) => {
      updateSelectAll(masterSelector, childSelector);
      kgwSettingsUiTraceR48B3("settings-select-all", "r48b3-select-all-child-change", {
        trusted: Boolean(event && event.isTrusted),
        masterSelector: String(masterSelector || ""),
        childSelector: String(childSelector || ""),
        targetId: String(node.id || ""),
        targetValue: String(node.dataset.settingsLanguage || node.dataset.settingsCurrency || node.dataset.settingsVisibleTab || "")
      });
      markDirty();
    });
  });

  updateSelectAll(masterSelector, childSelector);
}

function combineUrl() {
  const base = q("#settingsApiBase");
  const path = q("#settingsApiPath");
  const preview = q("#settingsApiPreview");

  if (!base || !path || !preview) return;

  const left = String(base.value || "").replace(/\/+$/, "");
  const right = String(path.value || "").replace(/^\/+/, "");

  preview.value = left && right ? `${left}/${right}` : left || right;
}

function selectEndpoint(row) {
  qa(".tree-row").forEach((node) => node.classList.remove("is-selected"));
  row.classList.add("is-selected");

  const key = q("#settingsApiKey");
  const desc = q("#settingsApiDescription");
  const base = q("#settingsApiBase");
  const path = q("#settingsApiPath");

  if (key) key.value = row.dataset.apiKey || "";
  if (desc) desc.value = row.dataset.apiDesc || "";
  if (base) base.value = row.dataset.apiBase || "";
  if (path) path.value = row.dataset.apiPath || "";

  combineUrl();
}


/* KGW_SETTINGS_DISPLAY_MINIMUM_SELECTION_OWNER_FIX_R1
 * settings.js owns Display Settings checkbox validity.
 * main.js remains the only owner of top-tab DOM visibility.
 * A display group must never have zero selected entries.
 */
/* KGW_SETTINGS_DISPLAY_SILENT_DEFAULT_FALLBACK_FIX_R3
 * Zero-selected Display Settings groups are repaired silently.
 * Fixed fallback: English / USD / Explorer.
 */
const KGW_DISPLAY_SELECTION_GROUPS_R1 = [
  {
    name: "languages",
    selector: "[data-settings-language]",
    attr: "data-settings-language",
    selectAll: "#settingsLangSelectAll",
    label: "Displayed Languages",
    fallbackValue: "en"
  },
  {
    name: "currencies",
    selector: "[data-settings-currency]",
    attr: "data-settings-currency",
    selectAll: "#settingsCurrencySelectAll",
    label: "Displayed Currencies",
    fallbackValue: "USD"
  },
  {
    name: "tabs",
    selector: "[data-settings-visible-tab]",
    attr: "data-settings-visible-tab",
    selectAll: "#settingsTabSelectAll",
    label: "Displayed Tabs",
    fallbackValue: "explorer"
  }
]

function kgwDisplaySelectionNodesR1(group) {
  return qa(group.selector).filter((node) => node && node.type === "checkbox");
}

function kgwDisplaySelectionCheckedCountR1(group) {
  return kgwDisplaySelectionNodesR1(group).filter((node) => node.checked).length;
}

function kgwDisplaySelectionSetAllR1(group, checked) {
  kgwDisplaySelectionNodesR1(group).forEach((node) => {
    node.checked = checked;
  });
  updateSelectAll(group.selectAll, group.selector);
}

function kgwDisplaySelectionSetFallbackR1(group) {
  const nodes = kgwDisplaySelectionNodesR1(group);
  let fallbackApplied = false;

  nodes.forEach((node) => {
    const value = node.getAttribute(group.attr) || node.value || "";
    const checked = value === group.fallbackValue;
    node.checked = checked;
    fallbackApplied = fallbackApplied || checked;
  });

  if (!fallbackApplied && nodes[0]) {
    nodes[0].checked = true;
  }

  updateSelectAll(group.selectAll, group.selector);
}

function kgwDisplaySelectionZeroGroupsR1() {
  return KGW_DISPLAY_SELECTION_GROUPS_R1.filter((group) => {
    const nodes = kgwDisplaySelectionNodesR1(group);
    return nodes.length > 0 && nodes.filter((node) => node.checked).length === 0;
  });
}

function kgwDisplaySelectionWarnR1(groups) {
  if (!Array.isArray(groups) || groups.length === 0) return;

  settingsLogger().warn("display selection repaired silently to fixed fallback", {
    groups: groups.map((group) => group.name),
    fallback: groups.reduce((acc, group) => {
      acc[group.name] = group.fallbackValue;
      return acc;
    }, {})
  });
}

function kgwDisplaySelectionEnsureDefaultsR1(reason = "default") {
  const repaired = [];

  KGW_DISPLAY_SELECTION_GROUPS_R1.forEach((group) => {
    const nodes = kgwDisplaySelectionNodesR1(group);
    if (nodes.length === 0) return;

    if (nodes.filter((node) => node.checked).length === 0) {
      kgwDisplaySelectionSetFallbackR1(group);
      repaired.push(group.name);
    }
  });

  if (repaired.length) {
    settingsLogger().warn("display selection repaired from zero-selected state", { reason, repaired });
  }

  return repaired;
}

function kgwDisplaySelectionValidateForSaveR1() {
  const zeroGroups = kgwDisplaySelectionZeroGroupsR1();
  if (zeroGroups.length === 0) return true;

  zeroGroups.forEach((group) => kgwDisplaySelectionSetFallbackR1(group));
  kgwDisplaySelectionWarnR1(zeroGroups);
  return true;
}

function kgwDisplaySelectionBindMinimumGuardsR1() {
  KGW_DISPLAY_SELECTION_GROUPS_R1.forEach((group) => {
    const master = q(group.selectAll);

    if (master && master.dataset.kgwMinimumSelectionBound !== "true") {
      master.dataset.kgwMinimumSelectionBound = "true";
      master.addEventListener("change", () => {
        window.setTimeout(() => {
          if (kgwDisplaySelectionCheckedCountR1(group) === 0) {
            kgwDisplaySelectionSetFallbackR1(group);
            kgwDisplaySelectionWarnR1([group]);
            markDirty();
          }
        }, 0);
      });
    }

    kgwDisplaySelectionNodesR1(group).forEach((node) => {
      if (node.dataset.kgwMinimumSelectionBound === "true") return;
      node.dataset.kgwMinimumSelectionBound = "true";
      node.addEventListener("change", () => {
        window.setTimeout(() => {
          if (kgwDisplaySelectionCheckedCountR1(group) === 0) {
            kgwDisplaySelectionSetFallbackR1(group);
            kgwDisplaySelectionWarnR1([group]);
            markDirty();
          }
        }, 0);
      });
    });
  });
}

function collectState() {
  const state = {
    inputs: {},
    checks: {},
    activeOuter: qa("[data-settings-tab].active")[0]?.dataset.settingsTab || "api-performance",
    activeInner: qa("[data-settings-inner-tab].active")[0]?.dataset.settingsInnerTab || "general"
  };

  qa("input, select").forEach((node) => {
    if (!node.id) return;

    if (node.type === "checkbox") {
      state.checks[node.id] = node.checked;
    } else {
      state.inputs[node.id] = node.value;
    }
  });

  qa("[data-settings-language], [data-settings-currency], [data-settings-visible-tab]").forEach((node, index) => {
    const key =
      node.dataset.settingsLanguage ? `language:${node.dataset.settingsLanguage}` :
      node.dataset.settingsCurrency ? `currency:${node.dataset.settingsCurrency}` :
      node.dataset.settingsVisibleTab ? `tab:${node.dataset.settingsVisibleTab}` :
      `check:${index}`;

    state.checks[key] = node.checked;
  });

  return state;
}

function applyState(state) {
  if (!state || typeof state !== "object") return;

  Object.entries(state.inputs || {}).forEach(([id, value]) => {
    const node = q(`#${CSS.escape(id)}`);
    if (node) node.value = value;
  });

  Object.entries(state.checks || {}).forEach(([id, value]) => {
    let node = q(`#${CSS.escape(id)}`);

    if (!node && id.startsWith("language:")) {
      node = q(`[data-settings-language="${CSS.escape(id.slice(9))}"]`);
    }

    if (!node && id.startsWith("currency:")) {
      node = q(`[data-settings-currency="${CSS.escape(id.slice(9))}"]`);
    }

    if (!node && id.startsWith("tab:")) {
      node = q(`[data-settings-visible-tab="${CSS.escape(id.slice(4))}"]`);
    }

    if (node && node.type === "checkbox") {
      node.checked = !!value;
    }
  });

  updateSelectAll("#settingsLangSelectAll", "[data-settings-language]");
  updateSelectAll("#settingsCurrencySelectAll", "[data-settings-currency]");
  updateSelectAll("#settingsTabSelectAll", "[data-settings-visible-tab]");

  if (state.activeOuter) activateOuter(state.activeOuter);
  if (state.activeInner) activateInner(state.activeInner);

  combineUrl();
}


/* KGW_SETTINGS_DISPLAY_SOURCE_BASED_FIX_R2
 * settings.js remains the canonical Settings state owner.
 * main.js remains the shell display application owner.
 */
function kgwSettingsDisplayPrefsFromCanonicalState(state) {
  const checks = state && typeof state === "object" ? state.checks : null;
  if (!checks || typeof checks !== "object") return null;

  const prefs = { languages: [], currencies: [], tabs: [] };

  Object.entries(checks).forEach(([key, checked]) => {
    if (!checked) return;

    if (key.startsWith("language:")) {
      prefs.languages.push(key.slice("language:".length));
      return;
    }

    if (key.startsWith("currency:")) {
      prefs.currencies.push(key.slice("currency:".length));
      return;
    }

    if (key.startsWith("tab:")) {
      prefs.tabs.push(key.slice("tab:".length));
    }
  });

  prefs.languages = Array.from(new Set(prefs.languages));
  prefs.currencies = Array.from(new Set(prefs.currencies));
  prefs.tabs = Array.from(new Set(prefs.tabs));

  return prefs.languages.length || prefs.currencies.length || prefs.tabs.length ? prefs : null;
}

function kgwSettingsApplyShellDisplayFromState(state, reason = "settings") {
  const prefs = kgwSettingsDisplayPrefsFromCanonicalState(state);
  const shell = window.kgwShellDisplayPreferencesR71;

  if (!prefs || !shell) return;

  try {
    if (typeof shell.save === "function") shell.save(prefs);
    if (typeof shell.apply === "function") shell.apply(prefs, reason);
  } catch (_) {}

  try {
    window.dispatchEvent(new CustomEvent("kgw:shell-display-preferences-changed", {
      detail: prefs
    }));
  } catch (_) {}
}


/* KGW_DYNAMIC_PATHS_BACKEND_OWNER_FIX_R4
 * Rust owns environment-specific default paths.
 * settings.js may display and save path values, but path field defaults must not hard-code a Windows user path.
 */
function kgwSettingsBackendInvokeR4(command, payload = {}) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_INVOKE__;

  if (typeof invoke !== "function") {
    return Promise.reject(new Error("Tauri invoke is not available"));
  }

  return invoke(command, payload);
}

function kgwSettingsIsStaleUserPathR4(value) {
  const text = String(value || "");
  return /^[A-Za-z]:[\\/]+Users[\\/]+[^\\/]+[\\/]+AppData[\\/]+Roaming[\\/]+KaspaGateway/i.test(text) || /AppData[\\/]+Roaming[\\/]+KaspaGateway/i.test(text);
}

function kgwSettingsApplyDynamicPathValuesR4(paths, options = {}) {
  const force = options.force === true;

  const mapping = {
    settingsDatabasePath: paths.database || paths.database_path || paths.data || "",
    settingsExportPath: paths.exports || paths.export_path || "",
    settingsLogPath: paths.logs || paths.log_path || "",
    settingsBackupPath: paths.backups || paths.backup_path || ""
  };

  Object.entries(mapping).forEach(([id, value]) => {
    if (!value) return;

    const node = q(`#${CSS.escape(id)}`);
    if (!node) return;

    const current = String(node.value || "");
    if (force || current.trim() === "" || kgwSettingsIsStaleUserPathR4(current)) {
      node.value = value;
    }
  });
}

async function kgwSettingsLoadDynamicPathDefaultsR4(reason = "settings") {
  try {
    const defaults = await kgwSettingsBackendInvokeR4("settings_defaults");
    if (defaults && defaults.paths) {
      kgwSettingsApplyDynamicPathValuesR4(defaults.paths, { force: reason === "reset-defaults" });
      settingsLogger().log("dynamic settings paths loaded from backend owner", { reason });
      return defaults.paths;
    }
  } catch (error) {
    settingsLogger().warn("dynamic settings paths backend load failed", { reason, error: String(error?.message || error) });
  }

  return null;
}

async function kgwSettingsRepairDynamicPathsBeforeSaveR4() {
  await kgwSettingsLoadDynamicPathDefaultsR4("save-repair");
}

async function saveState() {
  await kgwSettingsRepairDynamicPathsBeforeSaveR4();

  if (!kgwDisplaySelectionValidateForSaveR1()) {
    setSaveEnabled(true);
    return;
  }

  const state = collectState();

  try {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(state, null, 2));
  } catch (_) {}

  kgwSettingsApplyShellDisplayFromState(state, "settings-save");

  setSaveEnabled(false);
  settingsLogger().log("settings saved");
}

async function resetDefaults(options = {}) {
  const shouldClearStorage = options.clearStorage !== false;
  const shouldApplyShell = options.applyShell !== false;

  if (shouldClearStorage) {
    try {
      localStorage.removeItem(SETTINGS_STORAGE_KEY);
    } catch (_) {}
  }

  qa("input[type='checkbox']").forEach((node) => {
    node.checked = node.id !== "settingsStartWindows" && node.id !== "settingsEnableAutoRefresh";
  });

  const defaults = {
    settingsLoggingLevel: "INFO",
    settingsDatabasePath: "",
    settingsExportPath: "",
    settingsLogPath: "",
    settingsBackupPath: "",
    settingsApiProfile: "default",
    settingsApiTimeout: "30",
    settingsRetryAttempts: "5",
    settingsBackoffFactor: "4.0",
    settingsMaxWorkers: "10",
    settingsMaxPages: "10000",
    settingsPageDelay: "0.05",
    settingsPriceCacheHours: "0.25",
    settingsNetworkCacheHours: "0.25",
    settingsRefreshInterval: "60"
  };

  Object.entries(defaults).forEach(([id, value]) => {
    const node = q(`#${CSS.escape(id)}`);
    if (node) node.value = value;
  });

  const firstEndpoint = q(".tree-row");
  if (firstEndpoint) selectEndpoint(firstEndpoint);

  activateOuter("api-performance");
  activateInner("general");

  updateSelectAll("#settingsLangSelectAll", "[data-settings-language]");
  updateSelectAll("#settingsCurrencySelectAll", "[data-settings-currency]");
  updateSelectAll("#settingsTabSelectAll", "[data-settings-visible-tab]");

  kgwDisplaySelectionEnsureDefaultsR1("reset-defaults");

  if (shouldApplyShell) {
    kgwSettingsApplyShellDisplayFromState(collectState(), "settings-reset-defaults");
  }

  setSaveEnabled(false);

  /* KGW_RESTORE_DEFAULTS_DYNAMIC_PATHS_FULL_REWRITE_FIX_R2
   * Restore Defaults must finish by applying backend-owned dynamic file paths.
   */
  await kgwSettingsLoadDynamicPathDefaultsR4("reset-defaults");
}

function bindStaticActions() {
  qa("[data-settings-tab]").forEach((button) => {
    if (button.dataset.bound === "true") return;
    button.dataset.bound = "true";

    button.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-navigation", "r48b3-settings-tab-click", {
        trusted: Boolean(event && event.isTrusted),
        selected: String(button.dataset.settingsTab || ""),
        text: String(button.textContent || "").trim()
      });
      activateOuter(button.dataset.settingsTab);
      markDirty();
    });
  });

  qa("[data-settings-inner-tab]").forEach((button) => {
    if (button.dataset.bound === "true") return;
    button.dataset.bound = "true";

    button.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-navigation", "r48b3-settings-inner-tab-click", {
        trusted: Boolean(event && event.isTrusted),
        selected: String(button.dataset.settingsInnerTab || ""),
        text: String(button.textContent || "").trim()
      });
      activateInner(button.dataset.settingsInnerTab);
      markDirty();
    });
  });

  bindSelectAll("#settingsLangSelectAll", "[data-settings-language]");
  bindSelectAll("#settingsCurrencySelectAll", "[data-settings-currency]");
  bindSelectAll("#settingsTabSelectAll", "[data-settings-visible-tab]");
  kgwDisplaySelectionBindMinimumGuardsR1();

  qa("input, select").forEach((node) => {
    if (node.dataset.changeBound === "true") return;
    node.dataset.changeBound = "true";

    node.addEventListener("input", (event) => {
      kgwSettingsUiTraceR48B3("settings-choice", "r48b3-settings-input", {
        trusted: Boolean(event && event.isTrusted),
        targetId: String(node.id || ""),
        targetName: String(node.name || ""),
        targetTag: String(node.tagName || ""),
        value: String(node.type === "checkbox" ? node.checked : node.value || "")
      });
      combineUrl();
      markDirty();
    });

    node.addEventListener("change", (event) => {
      kgwSettingsUiTraceR48B3("settings-choice", "r48b3-settings-change", {
        trusted: Boolean(event && event.isTrusted),
        targetId: String(node.id || ""),
        targetName: String(node.name || ""),
        targetTag: String(node.tagName || ""),
        value: String(node.type === "checkbox" ? node.checked : node.value || "")
      });
      combineUrl();
      markDirty();
    });
  });

  qa(".tree-row").forEach((row) => {
    if (row.dataset.bound === "true") return;
    row.dataset.bound = "true";

    row.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-endpoint", "r48b3-endpoint-row-click", {
        trusted: Boolean(event && event.isTrusted),
        apiKey: String(row.dataset.apiKey || ""),
        text: String(row.textContent || "").trim()
      });
      selectEndpoint(row);
      markDirty();
    });
  });

  qa("[data-clear-for]").forEach((button) => {
    if (button.dataset.bound === "true") return;
    button.dataset.bound = "true";

    button.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-field-action", "r48b3-clear-for-click", {
        trusted: Boolean(event && event.isTrusted),
        target: String(button.dataset.clearFor || ""),
        text: String(button.textContent || "").trim()
      });
      const target = q(`#${CSS.escape(button.dataset.clearFor)}`);
      if (target) target.value = "";
      markDirty();
    });
  });

  qa("[data-browse-for]").forEach((button) => {
    if (button.dataset.bound === "true") return;
    button.dataset.bound = "true";

    button.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-field-action", "r48b3-browse-for-click", {
        trusted: Boolean(event && event.isTrusted),
        target: String(button.dataset.browseFor || ""),
        text: String(button.textContent || "").trim()
      });
      const target = q(`#${CSS.escape(button.dataset.browseFor)}`);
      if (target && !target.value) {
        void kgwSettingsLoadDynamicPathDefaultsR4("browse-defaults");
      }
      markDirty();
    });
  });

  const reset = q("#settingsResetDefaults");
  if (reset && reset.dataset.bound !== "true") {
    reset.dataset.bound = "true";
    reset.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-action", "r48b3-reset-defaults-click", {
        trusted: Boolean(event && event.isTrusted),
        targetId: "settingsResetDefaults"
      });
      resetDefaults();
    });
  }

  const save = q("#settingsSaveSettings");
  if (save && save.dataset.bound !== "true") {
    save.dataset.bound = "true";
    save.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-action", "r48b3-save-settings-click", {
        trusted: Boolean(event && event.isTrusted),
        targetId: "settingsSaveSettings"
      });
      saveState();
    });
  }

  const placeholderActions = [
    "settingsProfileAdd",
    "settingsProfileRename",
    "settingsProfileDelete",
    "settingsResetSelectedEndpoint",
    "settingsAddressAdd",
    "settingsAddressDelete",
    "settingsAddressClear",
    "settingsAddressRefresh",
    "settingsExportAddresses",
    "settingsImportAddresses",
    "settingsDbRefresh",
    "settingsDbCompact",
    "settingsDbClearCaches",
    "settingsDbBackup",
    "settingsDbRestore",
    "settingsDbDelete"
  ];

  placeholderActions.forEach((id) => {
    const button = q(`#${CSS.escape(id)}`);
    if (!button || button.dataset.bound === "true") return;

    button.dataset.bound = "true";
    button.addEventListener("click", (event) => {
      kgwSettingsUiTraceR48B3("settings-placeholder-action", "r48b3-placeholder-action-click", {
        trusted: Boolean(event && event.isTrusted),
        actionId: String(id || ""),
        text: String(button.textContent || "").trim()
      });

      if (id === "settingsAddressClear") {
        const name = q("#settingsAddressName");
        const address = q("#settingsAddressValue");
        if (name) name.value = "";
        if (address) address.value = "";
      }

      if (id === "settingsAddressRefresh") {
        const updated = q("#settingsAddressLastUpdated");
        if (updated) {
          updated.textContent = `Last Updated: ${new Date().toLocaleTimeString("en-US", { hour12: false })}`;
        }
      }

      markDirty();
      settingsLogger().log("settings action", { id });
    });
  });
}

function loadSavedState() {
  try {
    const saved = JSON.parse(localStorage.getItem(SETTINGS_STORAGE_KEY) || "null");
    if (saved) {
      applyState(saved);
      const repaired = kgwDisplaySelectionEnsureDefaultsR1("settings-load");
      const state = collectState();

      if (repaired.length) {
        try {
          localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(state, null, 2));
        } catch (_) {}
      }

      window.setTimeout(() => kgwSettingsApplyShellDisplayFromState(state, "settings-load"), 0);
      void kgwSettingsLoadDynamicPathDefaultsR4("settings-load");
    } else {
      kgwDisplaySelectionEnsureDefaultsR1("settings-load-defaults");
      void kgwSettingsLoadDynamicPathDefaultsR4("settings-load-defaults");
    }
  } catch (_) {
    kgwDisplaySelectionEnsureDefaultsR1("settings-load-error");
    void kgwSettingsLoadDynamicPathDefaultsR4("settings-load-error");
  }
}


function installLogDiagnosticsSettings() {
  if (q("#settingsClearLogOnStartup") || q("#settingsDeveloperOperationLogs")) return;

  const clearKey = "kgw.clearLogOnStartup";
  const devKey = "kgw.developerOperationLogs";

  function makeRow(id, text, key) {
    const label = document.createElement("label");
    label.style.display = "flex";
    label.style.alignItems = "center";
    label.style.gap = "10px";
    label.style.margin = "8px 0";
    label.style.cursor = "pointer";

    const input = document.createElement("input");
    input.id = id;
    input.type = "checkbox";
    input.checked = localStorage.getItem(key) === "1";

    input.addEventListener("change", function (event) {
      kgwSettingsUiTraceR48B3("settings-log-diagnostics", "r48b3-log-diagnostics-change", {
        trusted: Boolean(event && event.isTrusted),
        targetId: String(id || ""),
        key: String(key || ""),
        checked: Boolean(input.checked)
      });
      localStorage.setItem(key, input.checked ? "1" : "0");
      setSaveEnabled(true);
      settingsLogger().log(`${text}: ${input.checked ? "enabled" : "disabled"}`);
    });

    const span = document.createElement("span");
    span.textContent = text;

    label.appendChild(input);
    label.appendChild(span);
    return label;
  }

  const block = document.createElement("div");
  block.id = "settingsLogDiagnostics";
  block.style.marginTop = "12px";
  block.style.paddingTop = "10px";
  block.style.borderTop = "1px solid rgba(120,160,210,0.35)";

  const title = document.createElement("div");
  title.textContent = (window.kgwT ? window.kgwT("settings.logDiagnostics") : "Log & Diagnostics");
  title.style.fontWeight = "700";
  title.style.marginBottom = "8px";

  block.appendChild(title);
  block.appendChild(makeRow("settingsClearLogOnStartup", "Clear log on startup", clearKey));
  block.appendChild(makeRow("settingsDeveloperOperationLogs", "Developer operation logs", devKey));

  const settingsRoot = root();
  if (!settingsRoot) return;

  const loggingControl =
    q("#settingsLoggingLevel") ||
    Array.from(settingsRoot.querySelectorAll("select, input")).find((node) =>
      String(node.id || node.name || "").toLowerCase().includes("logging")
    );

  const advancedBox =
    loggingControl?.closest("fieldset") ||
    loggingControl?.closest(".settings-section") ||
    loggingControl?.closest("div") ||
    Array.from(settingsRoot.querySelectorAll("fieldset, section, div")).find((node) => {
      const text = String(node.textContent || "");
      return text.includes("Logging Level") ||
        text.includes("Check for updates on startup") ||
        text.includes("Start with Windows");
    }) ||
    settingsRoot;

  advancedBox.appendChild(block);
  advancedBox.style.overflow = "visible";
  advancedBox.style.minHeight = "250px";
}
export async function initSettingsTab() {
  const node = root();

  if (!node) {
    settingsLogger().warn("settings root missing");
    return;
  }

  if (node.dataset.settingsPythonInitialized === "true") {
    return;
  }

  node.dataset.settingsPythonInitialized = "true";

  bindStaticActions();
  resetDefaults({ clearStorage: false, applyShell: false });
  loadSavedState();
  kgwDisplaySelectionEnsureDefaultsR1("settings-init");
  kgwSettingsNormalizeNumericFields();
  void kgwSettingsLoadDynamicPathDefaultsR4("settings-init");

  const firstEndpoint = q(".tree-row");
  if (firstEndpoint && !q(".tree-row.is-selected")) {
    selectEndpoint(firstEndpoint);
  }

  installLogDiagnosticsSettings();

  setSaveEnabled(false);
  settingsLogger().log("settings python exact ui initialized");
}

/* KGW real database maintenance binding: DB status comes from Rust, not static HTML. */
function kgwSettingsDbInvoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke || window.__TAURI_INVOKE__;
}

function kgwFormatSettingsDbModified(value) {
  const raw = String(value || "");

  if (!raw || raw === "missing" || raw === "unknown") return raw || "--";

  const ms = Number(raw);
  if (!Number.isFinite(ms)) return raw;

  const date = new Date(ms);
  if (Number.isNaN(date.getTime())) return raw;

  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  const ss = String(date.getSeconds()).padStart(2, "0");

  return `${y}-${m}-${d} ${hh}:${mm}:${ss}`;
}

function kgwRenderSettingsDatabaseRows(rows) {
  const tbody = document.getElementById("settingsDatabaseRows");
  if (!tbody) return;

  tbody.innerHTML = "";

  if (!Array.isArray(rows) || rows.length === 0) {
    const tr = document.createElement("tr");
    const td = document.createElement("td");
    td.colSpan = 4;
    td.textContent = (window.kgwT ? window.kgwT("settings.noDatabaseFilesFound") : "No database files found.");
    tr.appendChild(td);
    tbody.appendChild(tr);
    return;
  }

  for (const row of rows) {
    const tr = document.createElement("tr");

    const cells = [
      row.file || "--",
      Number(row.size_kb || 0).toLocaleString(undefined, {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2
      }),
      kgwFormatSettingsDbModified(row.last_modified),
      row.details || (row.exists ? "available" : "missing")
    ];

    for (const value of cells) {
      const td = document.createElement("td");
      td.textContent = String(value);
      tr.appendChild(td);
    }

    tbody.appendChild(tr);
  }
}

async function kgwRefreshSettingsDatabaseStatus() {
  const tbody = document.getElementById("settingsDatabaseRows");
  const invoke = kgwSettingsDbInvoke();

  if (!tbody) return;

  if (!invoke) {
    tbody.innerHTML = "<tr><td colspan=\"4\">Tauri invoke API is not available.</td></tr>";
    return;
  }

  tbody.innerHTML = "<tr><td colspan=\"4\">Loading database status...</td></tr>";

  try {
    const rows = await invoke("kgw_settings_database_status");
    kgwRenderSettingsDatabaseRows(rows);
  } catch (error) {
    tbody.innerHTML = "";
    const tr = document.createElement("tr");
    const td = document.createElement("td");
    td.colSpan = 4;
    td.textContent = `Failed to load database status: ${error?.message || error}`;
    tr.appendChild(td);
    tbody.appendChild(tr);
  }
}

function kgwInstallSettingsDatabaseStatus() {
  if (window.__kgwSettingsDatabaseStatusInstalled) return;
  window.__kgwSettingsDatabaseStatusInstalled = true;

  document.addEventListener("click", (event) => {
    const button = event.target.closest && event.target.closest("#settingsDbRefresh");
    if (!button) return;

    event.preventDefault();
    kgwRefreshSettingsDatabaseStatus();
  }, true);

  document.addEventListener("click", (event) => {
    const tab = event.target.closest && event.target.closest('[data-settings-tab="database-maintenance"]');
    if (!tab) return;

    window.setTimeout(kgwRefreshSettingsDatabaseStatus, 100);
  }, true);

  window.setTimeout(kgwRefreshSettingsDatabaseStatus, 300);
}

window.kgwRefreshSettingsDatabaseStatus = kgwRefreshSettingsDatabaseStatus;
window.kgwInstallSettingsDatabaseStatus = kgwInstallSettingsDatabaseStatus;

kgwInstallSettingsDatabaseStatus();

/* KGW real settings manage addresses binding: DB-backed. */
const KGW_SETTINGS_ADDRESS_STATE = {
  knownNames: new Map(),
  priceUsd: 0,
  balances: new Map(),
  loading: false
};

function kgwSettingsAddressInvoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke || window.__TAURI_INVOKE__;
}

function kgwSettingsAddressShort(value) {
  const text = String(value || "");
  if (text.length <= 28) return text;
  return `${text.slice(0, 14)}…${text.slice(-12)}`;
}

function kgwSettingsAddressNow() {
  const date = new Date();
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  const ss = String(date.getSeconds()).padStart(2, "0");
  return `${y}-${m}-${d} ${hh}:${mm}:${ss}`;
}

function kgwSettingsIsKaspaAddress(value) {
  const text = String(value || "").trim();
  return text.startsWith("kaspa:") || text.startsWith("kaspatest:");
}

function kgwSettingsAddressElements() {
  return {
    name: document.getElementById("settingsAddressName"),
    address: document.getElementById("settingsAddressValue"),
    rows: document.getElementById("settingsAddressRows"),
    lastUpdated: document.getElementById("settingsAddressLastUpdated"),
    explorer: document.getElementById("settingsAddressExplorer")
  };
}

function kgwSettingsAddressSetStatus(message) {
  const { lastUpdated } = kgwSettingsAddressElements();
  if (lastUpdated) lastUpdated.textContent = message;
  console.log("[KGW Settings Addresses]", message);
}

function kgwSettingsAddressNormalize(record) {
  const address = String(record?.address || record?.Address || "").trim();
  const rawName = String(record?.name || record?.Name || record?.label || record?.Label || "").trim();
  const network = String(record?.network || record?.Network || "mainnet").trim();

  // Do not show generated fallback names like "Kaspa kaspa:...." as user names.
  const generatedPrefix = `Kaspa ${kgwSettingsAddressShort(address)}`;
  const name = rawName === generatedPrefix || rawName.startsWith("Kaspa kaspa:")
    ? ""
    : rawName;

  return {
    address,
    name,
    network
  };
}

function kgwSettingsAddressFormatNumber(value, digits = 2) {
  const number = Number(value);
  if (!Number.isFinite(number)) return "--";

  return number.toLocaleString(undefined, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits
  });
}

async function kgwLoadSettingsKnownNames() {
  const invoke = kgwSettingsAddressInvoke();
  if (!invoke) return;

  try {
    const json = await invoke("settings_fetch_address_names");
    const items = Array.isArray(json)
      ? json
      : Array.isArray(json?.addresses)
        ? json.addresses
        : Array.isArray(json?.items)
          ? json.items
          : [];

    const map = new Map();

    for (const item of items) {
      const address = String(item?.address || item?.Address || "").trim();
      const name = String(
        item?.name ||
        item?.Name ||
        item?.known_name ||
        item?.knownName ||
        item?.label ||
        item?.Label ||
        ""
      ).trim();

      if (kgwSettingsIsKaspaAddress(address) && name) {
        map.set(address, name);
      }
    }

    KGW_SETTINGS_ADDRESS_STATE.knownNames = map;
  } catch (error) {
    console.warn("[KGW Settings Addresses] known names load failed", error);
  }
}

async function kgwLoadSettingsKaspaPrice() {
  const invoke = kgwSettingsAddressInvoke();
  if (!invoke) return;

  try {
    const prices = await invoke("kgw_get_kaspa_prices");
    const usd = Number(
      prices?.prices?.usd ||
      prices?.usd ||
      prices?.values?.USD ||
      prices?.values?.usd ||
      0
    );

    KGW_SETTINGS_ADDRESS_STATE.priceUsd = Number.isFinite(usd) ? usd : 0;
  } catch (error) {
    console.warn("[KGW Settings Addresses] price load failed", error);
  }
}

async function kgwLoadSettingsAddressBalance(address) {
  if (!kgwSettingsIsKaspaAddress(address)) return null;

  const invoke = kgwSettingsAddressInvoke();
  if (!invoke) return null;

  if (KGW_SETTINGS_ADDRESS_STATE.balances.has(address)) {
    return KGW_SETTINGS_ADDRESS_STATE.balances.get(address);
  }

  try {
    const result = await invoke("explorer_fetch_balance", { address });

    const balanceKas = Number(
      result?.balance_kas ??
      result?.balanceKas ??
      result?.kas ??
      result?.balance ??
      0
    );

    const normalized = Number.isFinite(balanceKas) ? balanceKas : 0;
    KGW_SETTINGS_ADDRESS_STATE.balances.set(address, normalized);
    return normalized;
  } catch (error) {
    console.warn("[KGW Settings Addresses] balance fetch failed", address, error);
    KGW_SETTINGS_ADDRESS_STATE.balances.set(address, null);
    return null;
  }
}

async function kgwEnrichSettingsAddressRows(records) {
  await Promise.allSettled([
    kgwLoadSettingsKnownNames(),
    kgwLoadSettingsKaspaPrice()
  ]);

  const list = Array.isArray(records)
    ? records.map(kgwSettingsAddressNormalize).filter((item) => kgwSettingsIsKaspaAddress(item.address))
    : [];

  for (const item of list) {
    item.knownName = KGW_SETTINGS_ADDRESS_STATE.knownNames.get(item.address) || "";
    item.balanceKas = await kgwLoadSettingsAddressBalance(item.address);
    item.valueUsd = item.balanceKas == null
      ? null
      : item.balanceKas * KGW_SETTINGS_ADDRESS_STATE.priceUsd;
  }

  return list;
}

function kgwSettingsAddressSelect(row) {
  const { name, address, explorer } = kgwSettingsAddressElements();

  if (name) name.value = row.name || "";
  if (address) address.value = row.address || "";

  if (explorer) {
    explorer.disabled = !row.address;
    explorer.classList.toggle("disabled-btn", !row.address);
  }
}

async function kgwRenderSettingsAddressRows(records) {
  const { rows } = kgwSettingsAddressElements();
  if (!rows) return;

  rows.innerHTML = "";

  const list = await kgwEnrichSettingsAddressRows(records);

  if (!list.length) {
    const tr = document.createElement("tr");
    const td = document.createElement("td");
    td.colSpan = 5;
    td.textContent = (window.kgwT ? window.kgwT("settings.noSavedAddresses") : "No saved addresses.");
    tr.appendChild(td);
    rows.appendChild(tr);
    return;
  }

  for (const item of list) {
    const tr = document.createElement("tr");
    tr.dataset.address = item.address;
    tr.style.cursor = "pointer";

    const balanceText = item.balanceKas == null
      ? "--"
      : kgwSettingsAddressFormatNumber(item.balanceKas, 2);

    const valueText = item.valueUsd == null
      ? "--"
      : `${kgwSettingsAddressFormatNumber(item.valueUsd, 2)} USD`;

    const cells = [
      item.name || "",
      item.address,
      item.knownName || "",
      balanceText,
      valueText
    ];

    for (const value of cells) {
      const td = document.createElement("td");
      td.textContent = String(value);
      tr.appendChild(td);
    }

    tr.addEventListener("click", () => {
      Array.from(rows.querySelectorAll("tr")).forEach((node) => node.classList.remove("is-selected"));
      tr.classList.add("is-selected");
      kgwSettingsAddressSelect(item);
    });

    rows.appendChild(tr);
  }
}

async function kgwRefreshSettingsAddresses() {
  if (KGW_SETTINGS_ADDRESS_STATE.loading) return [];

  const invoke = kgwSettingsAddressInvoke();

  if (!invoke) {
    kgwSettingsAddressSetStatus("Last Updated: Tauri invoke API is not available.");
    return [];
  }

  KGW_SETTINGS_ADDRESS_STATE.loading = true;
  kgwSettingsAddressSetStatus("Last Updated: loading...");

  try {
    const records = await invoke("get_all_addresses");
    await kgwRenderSettingsAddressRows(records);
    kgwSettingsAddressSetStatus(`Last Updated: ${kgwSettingsAddressNow()}`);

    if (typeof window.kgwRefreshSavedAddresses === "function") {
      window.kgwRefreshSavedAddresses().catch(console.error);
    }

    return records;
  } catch (error) {
    await kgwRenderSettingsAddressRows([]);
    kgwSettingsAddressSetStatus(`Last Updated: failed - ${error?.message || error}`);
    return [];
  } finally {
    KGW_SETTINGS_ADDRESS_STATE.loading = false;
  }
}

async function kgwSaveSettingsAddress() {
  const invoke = kgwSettingsAddressInvoke();
  const { name, address } = kgwSettingsAddressElements();

  if (!invoke) {
    kgwSettingsAddressSetStatus("Last Updated: Tauri invoke API is not available.");
    return;
  }

  const cleanAddress = String(address?.value || "").trim();
  const cleanName = String(name?.value || "").trim();

  if (!kgwSettingsIsKaspaAddress(cleanAddress)) {
    kgwSettingsAddressSetStatus("Last Updated: invalid Kaspa address.");
    return;
  }

  kgwSettingsAddressSetStatus("Last Updated: saving...");

  try {
    await invoke("save_address", {
      address: cleanAddress,
      name: cleanName
    });

    KGW_SETTINGS_ADDRESS_STATE.balances.delete(cleanAddress);
    await kgwRefreshSettingsAddresses();
  } catch (error) {
    kgwSettingsAddressSetStatus(`Last Updated: save failed - ${error?.message || error}`);
  }
}

async function kgwDeleteSettingsAddress() {
  const invoke = kgwSettingsAddressInvoke();
  const { address } = kgwSettingsAddressElements();

  if (!invoke) {
    kgwSettingsAddressSetStatus("Last Updated: Tauri invoke API is not available.");
    return;
  }

  const cleanAddress = String(address?.value || "").trim();

  if (!kgwSettingsIsKaspaAddress(cleanAddress)) {
    kgwSettingsAddressSetStatus("Last Updated: select a saved Kaspa address first.");
    return;
  }

  kgwSettingsAddressSetStatus("Last Updated: deleting...");

  try {
    await invoke("delete_saved_address", {
      address: cleanAddress
    });

    KGW_SETTINGS_ADDRESS_STATE.balances.delete(cleanAddress);

    const elements = kgwSettingsAddressElements();
    if (elements.name) elements.name.value = "";
    if (elements.address) elements.address.value = "";

    await kgwRefreshSettingsAddresses();
  } catch (error) {
    kgwSettingsAddressSetStatus(`Last Updated: delete failed - ${error?.message || error}`);
  }
}

function kgwClearSettingsAddressFields() {
  const { name, address, explorer } = kgwSettingsAddressElements();

  if (name) name.value = "";
  if (address) address.value = "";
  if (explorer) {
    explorer.disabled = true;
    explorer.classList.add("disabled-btn");
  }
}

function kgwInstallSettingsManageAddresses() {
  if (window.__kgwSettingsManageAddressesInstalled) return;
  window.__kgwSettingsManageAddressesInstalled = true;

  document.addEventListener("click", (event) => {
    const add = event.target.closest && event.target.closest("#settingsAddressAdd");
    const del = event.target.closest && event.target.closest("#settingsAddressDelete");
    const clear = event.target.closest && event.target.closest("#settingsAddressClear");
    const refresh = event.target.closest && event.target.closest("#settingsAddressRefresh");

    if (!add && !del && !clear && !refresh) return;

    event.preventDefault();
    event.stopImmediatePropagation();

    kgwSettingsUiTraceR48B3("settings-addresses", "r48b3-address-action-click", {
      trusted: Boolean(event && event.isTrusted),
      action: add ? "add" : del ? "delete" : clear ? "clear" : refresh ? "refresh" : "unknown",
      text: String((add || del || clear || refresh)?.textContent || "").trim()
    });

    if (add) {
      kgwSaveSettingsAddress();
      return;
    }

    if (del) {
      kgwDeleteSettingsAddress();
      return;
    }

    if (clear) {
      kgwClearSettingsAddressFields();
      return;
    }

    if (refresh) {
      KGW_SETTINGS_ADDRESS_STATE.balances.clear();
      kgwRefreshSettingsAddresses();
    }
  }, true);

  document.addEventListener("click", (event) => {
    const tab = event.target.closest && event.target.closest('[data-settings-tab="manage-addresses"]');
    if (!tab) return;

    window.setTimeout(kgwRefreshSettingsAddresses, 100);
  }, true);

  window.setTimeout(kgwRefreshSettingsAddresses, 500);
}

window.kgwRefreshSettingsAddresses = kgwRefreshSettingsAddresses;
window.kgwInstallSettingsManageAddresses = kgwInstallSettingsManageAddresses;

kgwInstallSettingsManageAddresses();

/* KGW fixed Manage Addresses layout: clean, no horizontal scroll by default. */
function kgwInstallSettingsManageAddressesCleanLayout() {
  if (window.__kgwSettingsManageAddressesCleanLayoutInstalled) return;
  window.__kgwSettingsManageAddressesCleanLayoutInstalled = true;

  const oldStyle = document.getElementById("kgw-settings-manage-addresses-clean-layout-style");
  if (oldStyle) oldStyle.remove();

  const style = document.createElement("style");
  style.id = "kgw-settings-manage-addresses-clean-layout-style";
  style.textContent = `
    /* General panel */
    .manage-addresses-panel {
      display: flex !important;
      flex-direction: column !important;
      gap: 12px !important;
      width: 100% !important;
      min-width: 0 !important;
    }

    /* Inputs row */
    .manage-addresses-panel .address-form-row {
      display: grid !important;
      grid-template-columns: auto minmax(170px, 0.75fr) auto minmax(520px, 1.85fr) !important;
      gap: 10px !important;
      align-items: center !important;
      width: 100% !important;
    }

    .manage-addresses-panel .address-form-row label {
      white-space: nowrap !important;
      margin: 0 !important;
    }

    #settingsAddressName {
      width: 100% !important;
      min-width: 170px !important;
      max-width: 340px !important;
    }

    #settingsAddressValue {
      width: 100% !important;
      min-width: 520px !important;
      font-size: 14px !important;
      direction: ltr !important;
    }

    /* Action row */
    .manage-addresses-panel .address-action-row {
      display: flex !important;
      flex-wrap: wrap !important;
      align-items: center !important;
      gap: 10px !important;
      width: 100% !important;
    }

    #settingsAddressLastUpdated {
      margin-left: 8px !important;
      white-space: nowrap !important;
    }

    /* Table shell */
    .manage-addresses-panel .address-table-shell,
    .manage-addresses-panel .settings-table-shell {
      width: 100% !important;
      min-width: 0 !important;
      overflow-x: hidden !important;
      overflow-y: auto !important;
      flex: 1 1 auto !important;
    }

    .manage-addresses-panel table[aria-label="Managed addresses"],
    .manage-addresses-panel .python-table {
      table-layout: fixed !important;
      width: 100% !important;
      min-width: 0 !important;
    }

    .manage-addresses-panel th,
    .manage-addresses-panel td {
      vertical-align: middle !important;
      overflow: hidden !important;
      text-overflow: ellipsis !important;
    }

    /* Default column widths */
    .manage-addresses-panel table[aria-label="Managed addresses"] th:nth-child(1),
    .manage-addresses-panel table[aria-label="Managed addresses"] td:nth-child(1) {
      width: 11% !important;
    }

    .manage-addresses-panel table[aria-label="Managed addresses"] th:nth-child(2),
    .manage-addresses-panel table[aria-label="Managed addresses"] td:nth-child(2) {
      width: 45% !important;
    }

    .manage-addresses-panel table[aria-label="Managed addresses"] th:nth-child(3),
    .manage-addresses-panel table[aria-label="Managed addresses"] td:nth-child(3) {
      width: 12% !important;
    }

    .manage-addresses-panel table[aria-label="Managed addresses"] th:nth-child(4),
    .manage-addresses-panel table[aria-label="Managed addresses"] td:nth-child(4) {
      width: 14% !important;
      text-align: right !important;
      white-space: nowrap !important;
    }

    .manage-addresses-panel table[aria-label="Managed addresses"] th:nth-child(5),
    .manage-addresses-panel table[aria-label="Managed addresses"] td:nth-child(5) {
      width: 18% !important;
      text-align: right !important;
      white-space: nowrap !important;
    }

    /* Kaspa address column: wrap instead of forcing horizontal scroll */
    #settingsAddressRows td:nth-child(2) {
      direction: ltr !important;
      font-family: Consolas, "Courier New", monospace !important;
      font-size: 13px !important;
      white-space: normal !important;
      overflow-wrap: anywhere !important;
      word-break: break-word !important;
      line-height: 1.25 !important;
    }

    /* Name and Known Name compact */
    #settingsAddressRows td:nth-child(1),
    #settingsAddressRows td:nth-child(3) {
      white-space: nowrap !important;
    }

    /* Export buttons row */
    .manage-addresses-panel .address-export-row {
      display: flex !important;
      justify-content: flex-end !important;
      gap: 10px !important;
      width: 100% !important;
    }
  `;

  document.head.appendChild(style);

  window.setTimeout(() => {
    const rows = document.getElementById("settingsAddressRows");
    const table = rows?.closest("table");
    if (table) {
      table.style.width = "100%";
      table.style.tableLayout = "fixed";
      table.style.minWidth = "0";
    }

    const shell =
      rows?.closest(".address-table-shell") ||
      rows?.closest(".settings-table-shell");
    if (shell) {
      shell.style.overflowX = "hidden";
    }
  }, 80);
}

window.kgwInstallSettingsManageAddressesCleanLayout = kgwInstallSettingsManageAddressesCleanLayout;
kgwInstallSettingsManageAddressesCleanLayout();

/* KGW settings address explorer open binding */
function kgwSettingsAddressOpenValue() {
  const addressInput = document.getElementById("settingsAddressValue");
  const value = String(addressInput?.value || "").trim();
  return /^kaspa(test)?:[a-z0-9]{50,}$/i.test(value) ? value : "";
}

function kgwOpenKaspaAddressInBrowser(address) {
  const clean = String(address || "").trim();

  if (!/^kaspa(test)?:[a-z0-9]{50,}$/i.test(clean)) {
    console.warn("[KGW Settings Addresses] invalid address for explorer open", clean);
    return;
  }

  const url = `https://explorer.kaspa.org/addresses/${encodeURIComponent(clean)}`;
  window.open(url, "_blank", "noopener,noreferrer");
}

function kgwInstallSettingsAddressExplorerOpen() {
  if (window.__kgwSettingsAddressExplorerOpenInstalled) return;
  window.__kgwSettingsAddressExplorerOpenInstalled = true;

  document.addEventListener("click", (event) => {
    const button = event.target.closest && event.target.closest("#settingsAddressExplorer");
    if (!button) return;

    event.preventDefault();
    event.stopImmediatePropagation();

    const address = kgwSettingsAddressOpenValue();

    if (!address) return;

    kgwOpenKaspaAddressInBrowser(address);
  }, true);
}

window.kgwOpenKaspaAddressInBrowser = kgwOpenKaspaAddressInBrowser;
window.kgwInstallSettingsAddressExplorerOpen = kgwInstallSettingsAddressExplorerOpen;

kgwInstallSettingsAddressExplorerOpen();

async function kgwDeleteSettingsAddressTransactions() {
  const input =
    document.querySelector("#settingsAddressValue") ||
    document.querySelector("#settingsAddressAddress") ||
    document.querySelector("#settingsAddressInput");

  const address = String(input?.value || "").trim();

  if (!address) {
    if (typeof kgwSettingsAddressSetStatus === "function") {
      kgwSettingsAddressSetStatus("Last Updated: select an address first");
    } else {
      alert("Select an address first.");
    }
    return;
  }

  const ok = window.confirm(
    `Clear all cached transactions for this address?\n\n${address}\n\nThe saved address will remain.`
  );

  if (!ok) return;

  const button = document.querySelector("#settingsAddressDeleteTransactions");
  if (button) button.disabled = true;

  try {
    const invoke =
      (typeof kgwSettingsAddressInvoke === "function" && kgwSettingsAddressInvoke()) ||
      window.__TAURI__?.core?.invoke ||
      window.__TAURI__?.tauri?.invoke ||
      window.__TAURI_INVOKE__;

    if (!invoke) {
      throw new Error("Tauri invoke API is not available.");
    }

    const deleted = await invoke("explorer_delete_transactions_for_address", { address });

    if (typeof kgwSettingsAddressSetStatus === "function") {
      kgwSettingsAddressSetStatus(`Last Updated: cleared ${deleted} cached transactions`);
    } else {
      alert(`Cleared ${deleted} cached transactions.`);
    }

    const refresh = document.querySelector("#settingsAddressRefresh");
    if (refresh) refresh.click();
  } catch (error) {
    const message = error?.message || String(error);

    if (typeof kgwSettingsAddressSetStatus === "function") {
      kgwSettingsAddressSetStatus(`Last Updated: clear transactions failed - ${message}`);
    } else {
      alert(`Clear transactions failed: ${message}`);
    }

    console.error("[settings] clear transactions failed", error);
  } finally {
    if (button) button.disabled = false;
  }
}

document.addEventListener("click", (event) => {
  const button = event.target?.closest && event.target.closest("#settingsAddressDeleteTransactions");
  if (!button) return;

  event.preventDefault();
  kgwDeleteSettingsAddressTransactions();
});

/* KGW_R80_SETTINGS_SEMANTIC_I18N_BINDING_SAFE_V3 */
(function installKgwSettingsSemanticI18nBindingR80V3() {
  if (window.kgwSettingsSemanticI18nBindingR80V3) return;
  window.kgwSettingsSemanticI18nBindingR80V3 = true;

  const bindings = [
  {
    "text": "API Settings",
    "key": "settings.api.settings"
  },
  {
    "text": "Language",
    "key": "settings.language"
  },
  {
    "text": "Theme",
    "key": "settings.theme"
  },
  {
    "text": "Autostart can only be set for the installed application.",
    "key": "settings.autostart.can.only.be.set.for.the.installed.application"
  },
  {
    "text": "Autostart is only available on Windows.",
    "key": "settings.autostart.is.only.available.on.windows"
  }
];

  function normalized(value) {
    return String(value || "").replace(/\s+/g, " ").trim();
  }

  function applyBindings(root = document) {
    const candidates = Array.from(root.querySelectorAll("button, label, span, h1, h2, h3, h4, p, small, option"));

    for (const item of bindings) {
      for (const element of candidates) {
        if (element.dataset && element.dataset.i18n) continue;
        if (normalized(element.textContent) === item.text) {
          element.dataset.i18n = item.key;
        }
      }
    }

    if (typeof window.kgwApplyLanguageR73 === "function") {
      window.setTimeout(() => {
      if (typeof window.kgwReapplyLanguageSilentlyR89 === "function") {
        window.kgwReapplyLanguageSilentlyR89("binding-refresh");
      }
    }, 0);
    }
  }

  document.addEventListener("click", (event) => {
    if (event.target.closest("[data-tab='settings']")) {
      window.setTimeout(() => applyBindings(document), 60);
      window.setTimeout(() => applyBindings(document), 300);
    }
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(() => applyBindings(document), 0);
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => applyBindings(document), { once: true });
  } else {
    window.setTimeout(() => applyBindings(document), 0);
  }
})();

/* KGW_R83_SETTINGS_GENERAL_COMPREHENSIVE_I18N_BINDING */
(function installKgwSettingsGeneralComprehensiveI18nBindingR83() {
  if (window.kgwSettingsGeneralComprehensiveI18nBindingR83) return;
  window.kgwSettingsGeneralComprehensiveI18nBindingR83 = true;

  const bindings = [
  {
    "text": "General",
    "key": "settings.general"
  },
  {
    "text": "Language",
    "key": "settings.language"
  },
  {
    "text": "Currency",
    "key": "settings.currency"
  },
  {
    "text": "Theme",
    "key": "settings.theme"
  },
  {
    "text": "Displayed Currencies",
    "key": "settings.displayed.currencies"
  },
  {
    "text": "Displayed Tabs",
    "key": "settings.displayed.tabs"
  },
  {
    "text": "Displayed Languages",
    "key": "settings.displayed.languages"
  },
  {
    "text": "API Settings",
    "key": "settings.api.settings"
  },
  {
    "text": "Autostart can only be set for the installed application.",
    "key": "settings.autostart.can.only.be.set.for.the.installed.application"
  },
  {
    "text": "Autostart is only available on Windows.",
    "key": "settings.autostart.is.only.available.on.windows"
  }
];

  function normalized(value) {
    return String(value || "").replace(/\s+/g, " ").trim();
  }

  function applyBindings(root = document) {
    const candidates = Array.from(root.querySelectorAll("button, label, span, h1, h2, h3, h4, p, small, option, legend"));

    for (const item of bindings) {
      for (const element of candidates) {
        if (!element.dataset) continue;
        if (element.dataset.i18n) continue;

        if (normalized(element.textContent) === item.text) {
          element.dataset.i18n = item.key;
        }
      }
    }

    if (typeof window.kgwApplyLanguageR73 === "function") {
      window.setTimeout(() => {
      if (typeof window.kgwReapplyLanguageSilentlyR89 === "function") {
        window.kgwReapplyLanguageSilentlyR89("binding-refresh");
      }
    }, 0);
    }
  }

  document.addEventListener("click", (event) => {
    if (event.target.closest("[data-tab='settings']")) {
      window.setTimeout(() => applyBindings(document), 60);
      window.setTimeout(() => applyBindings(document), 300);
      window.setTimeout(() => applyBindings(document), 900);
    }
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(() => applyBindings(document), 0);
  });

  window.kgwApplySettingsGeneralI18nR83 = applyBindings;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => applyBindings(document), { once: true });
  } else {
    window.setTimeout(() => applyBindings(document), 0);
  }
})();


/* KGW_SETTINGS_DB_MAINTENANCE_ACTIONS_OWNER_V1 */
const KGW_SETTINGS_DB_ACTION_STATE = {
  selectedKind: ""
};

function kgwSettingsDbKindFromFileName(value) {
  const text = String(value || "").trim().toLowerCase();

  if (text.includes("address")) return "addresses";
  if (text.includes("transaction")) return "transactions";
  if (text.includes("appdata") || text.includes("app data") || text.includes("app_data")) return "app_data";

  return "";
}

function kgwSettingsDbStatus(message) {
  const row = document.querySelector("#settingsDatabaseRows");
  if (!row) return;
  row.dataset.kgwLastStatus = String(message || "");
  console.log("[KGW Settings DB]", message);
}

function kgwSettingsDbSelectedKind() {
  if (KGW_SETTINGS_DB_ACTION_STATE.selectedKind) return KGW_SETTINGS_DB_ACTION_STATE.selectedKind;

  const selected = document.querySelector("#settingsDatabaseRows tr.is-selected");
  if (!selected) return "";

  return selected.dataset.databaseKind || kgwSettingsDbKindFromFileName(selected.cells?.[0]?.textContent || "");
}

function kgwSettingsDbInstallRowSelection() {
  if (window.__kgwSettingsDbRowSelectionInstalled) return;
  window.__kgwSettingsDbRowSelectionInstalled = true;

  document.addEventListener("click", (event) => {
    const row = event.target.closest && event.target.closest("#settingsDatabaseRows tr");
    if (!row) return;

    const kind = kgwSettingsDbKindFromFileName(row.cells?.[0]?.textContent || "");
    if (!kind) return;

    Array.from(document.querySelectorAll("#settingsDatabaseRows tr")).forEach((node) => node.classList.remove("is-selected"));
    row.classList.add("is-selected");
    row.dataset.databaseKind = kind;
    KGW_SETTINGS_DB_ACTION_STATE.selectedKind = kind;
    kgwSettingsUiTraceR48B3("settings-database", "r48b3-database-row-select", {
      trusted: Boolean(event && event.isTrusted),
      database: String(kind || ""),
      text: String(row.textContent || "").trim()
    });
    kgwSettingsDbStatus("Selected database: " + kind);
  }, true);
}

async function kgwSettingsDbInvokeAction(command, args) {
  const invoke = kgwSettingsDbInvoke();
  if (!invoke) {
    kgwSettingsDbStatus("Tauri invoke API is not available.");
    return null;
  }

  const result = await invoke(command, args || {});

  if (result && Array.isArray(result.rows)) {
    kgwRenderSettingsDatabaseRows(result.rows);
  } else if (typeof window.kgwRefreshSettingsDatabaseStatus === "function") {
    await window.kgwRefreshSettingsDatabaseStatus();
  }

  if (result && result.message) {
    kgwSettingsDbStatus(result.message);
  }

  return result;
}

function kgwInstallSettingsDbMaintenanceActions() {
  if (window.__kgwSettingsDbMaintenanceActionsInstalled) return;
  window.__kgwSettingsDbMaintenanceActionsInstalled = true;

  kgwSettingsDbInstallRowSelection();

  document.addEventListener("click", (event) => {
    const button = event.target.closest && event.target.closest(
      "#settingsDbRefresh,#settingsDbCompact,#settingsDbClearCaches,#settingsDbBackup,#settingsDbRestore,#settingsDbDelete"
    );

    if (!button) return;

    event.preventDefault();
    event.stopImmediatePropagation();

    const id = button.id;

    kgwSettingsUiTraceR48B3("settings-database", "r48b3-database-action-click", {
      trusted: Boolean(event && event.isTrusted),
      actionId: String(id || ""),
      text: String(button.textContent || "").trim()
    });

    (async () => {
      if (id === "settingsDbRefresh") {
        if (typeof window.kgwRefreshSettingsDatabaseStatus === "function") {
          await window.kgwRefreshSettingsDatabaseStatus();
        }
        return;
      }

      if (id === "settingsDbCompact") {
        await kgwSettingsDbInvokeAction("kgw_settings_database_compact");
        return;
      }

      if (id === "settingsDbClearCaches") {
        if (!confirm("Clear application cache rows?")) return;
        await kgwSettingsDbInvokeAction("kgw_settings_database_clear_caches");
        return;
      }

      if (id === "settingsDbBackup") {
        await kgwSettingsDbInvokeAction("kgw_settings_database_backup");
        return;
      }

      if (id === "settingsDbRestore") {
        if (!confirm("Restore the latest database backup? A safety backup will be created first.")) return;
        await kgwSettingsDbInvokeAction("kgw_settings_database_restore_latest");
        return;
      }

      if (id === "settingsDbDelete") {
        const database = kgwSettingsDbSelectedKind();
        if (!database) {
          alert("Select a database row first.");
          return;
        }

        const typed = prompt("Type DELETE to delete and reinitialize: " + database);
        if (typed !== "DELETE") return;

        await kgwSettingsDbInvokeAction("kgw_settings_database_delete", { database });
      }
    })().catch((error) => {
      kgwSettingsDbStatus("Database action failed: " + (error?.message || error));
      console.error("[KGW Settings DB] action failed", error);
    });
  }, true);
}

window.kgwInstallSettingsDbMaintenanceActions = kgwInstallSettingsDbMaintenanceActions;
kgwInstallSettingsDbMaintenanceActions();
