import { applyStatusTone } from "../../status.js";
import {
  confirmUserAction,
  settingsDisplayChecksWithDefaults,
  settingsDisplayPreferences,
  settingsAddressesInstallAll,
  settingsAddressesOpenExplorer,
  settingsAddressesRefresh,
  settingsAddressesRenderRows,
  settingsAddressesSetStatus,
  settingsDisplayStateMissingContract,
  settingsDatabaseInstall,
  settingsDatabaseInstallMaintenance,
  settingsDatabaseRefresh,
  settingsSelectedDisplayKeys,
  settingsToWesternDigits
} from "../../settings-contract.js";
import { installGlobalSettingsLayout, installManageAddressesCleanLayout, installSettingsI18nBindings } from "../../settings-layout.js";
const SETTINGS_STORAGE_KEY = "kgw-settings-python-exact-state";

function kgwSettingsToWesternDigits(value) {
  return settingsToWesternDigits(value);
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
        } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
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
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
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
  kgwSettingsUpdateEndpointResetAvailability(row);
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

/* KGW_SHELL_DISPLAY_AND_ACTIVE_TAB_OWNER_R59C
 * Canonical settings adapter.
 * Settings prepares state and delegates shell decisions to the canonical owner in main.js.
 */
function kgwSettingsDisplayKeyForNodeR65(node) {
  if (!node || !node.dataset) return "";

  if (node.dataset.settingsLanguage) {
    return "language:" + node.dataset.settingsLanguage;
  }

  if (node.dataset.settingsCurrency) {
    return "currency:" + node.dataset.settingsCurrency;
  }

  if (node.dataset.settingsVisibleTab) {
    return "tab:" + node.dataset.settingsVisibleTab;
  }

  return "";
}

function kgwSettingsDisplayNodesR65() {
  return qa("[data-settings-language], [data-settings-currency], [data-settings-visible-tab]")
    .filter((node) => node && node.type === "checkbox");
}

function kgwSettingsKnownDisplayEntriesR65() {
  return kgwSettingsDisplayNodesR65().map((node) => ({
    key: kgwSettingsDisplayKeyForNodeR65(node),
    id: node.id || ""
  }));
}

function kgwSettingsDisplayChecksWithDefaultsR65(checks) {
  return settingsDisplayChecksWithDefaults(checks, kgwSettingsKnownDisplayEntriesR65());
}

function kgwSettingsSelectedDisplayKeysR65(checks, prefix) {
  return settingsSelectedDisplayKeys(checks, prefix);
}

function kgwSettingsDisplayStateLooksLegacyAllSelectedR65(state) {
  const checks = state && typeof state === "object" ? state.checks : null;
  return settingsDisplayStateMissingContract(checks);
}

function kgwSettingsApplyDisplayChecksR65(checks, reason = "display-checks") {
  const source = checks && typeof checks === "object" ? checks : {};

  kgwSettingsDisplayNodesR65().forEach((node) => {
    const key = kgwSettingsDisplayKeyForNodeR65(node);
    if (!key) return;

    const checked = source[key] === true;
    node.checked = checked;

    if (node.id) {
      source[node.id] = checked;
    }
  });

  updateSelectAll("#settingsLangSelectAll", "[data-settings-language]");
  updateSelectAll("#settingsCurrencySelectAll", "[data-settings-currency]");
  updateSelectAll("#settingsTabSelectAll", "[data-settings-visible-tab]");

  try {
    settingsLogger().log("display checkboxes applied", {
      patch: "R69",
      reason
    });
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }

  try {
    kgwSettingsUiTraceR48B3("settings-display", "r69-display-checks-applied", {
      reason: String(reason || ""),
      languages: kgwSettingsSelectedDisplayKeysR65(source, "language:").join(","),
      currencies: kgwSettingsSelectedDisplayKeysR65(source, "currency:").join(","),
      tabs: kgwSettingsSelectedDisplayKeysR65(source, "tab:").join(",")
    });
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
}

function kgwSettingsBuildCanonicalDefaultStateR65(reason = "display-defaults-r65", persist = false) {
  const baseState = collectState();
  baseState.checks = kgwSettingsDisplayChecksWithDefaultsR65(baseState.checks);

  kgwSettingsApplyDisplayChecksR65(baseState.checks, reason);

  const state = collectState();
  state.checks = kgwSettingsDisplayChecksWithDefaultsR65(state.checks);
  kgwSettingsApplyDisplayChecksR65(state.checks, reason + "-verified");

  const finalState = collectState();
  finalState.checks = kgwSettingsDisplayChecksWithDefaultsR65(finalState.checks);

  if (persist) {
    try {
      localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(finalState, null, 2));
    } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
  }

  kgwSettingsApplyShellDisplayFromState(finalState, reason);
  return finalState;
}

function kgwSettingsReapplyDisplayStateR65(state, reason = "display-state-r65") {
  const checks = state && typeof state === "object" && state.checks && typeof state.checks === "object"
    ? state.checks
    : null;

  if (!checks) {
    return kgwSettingsBuildCanonicalDefaultStateR65(reason + "-fallback", true);
  }

  kgwSettingsApplyDisplayChecksR65(checks, reason);
  return collectState();
}


function kgwSettingsApplyDisplayDefaultsR59C(reason = "startup-defaults", persist = false) {
  return kgwSettingsBuildCanonicalDefaultStateR65(reason, persist);
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
  return settingsDisplayPreferences(checks);
}

function kgwSettingsApplyShellDisplayFromState(state, reason = "settings") {
  /* KGW_SETTINGS_DIRECT_VISUAL_ONLY_R76B */
  const prefs = kgwSettingsDisplayPrefsFromCanonicalState(state);
  if (!prefs) {
    kgwSettingsUiTraceR48B3("settings-display-apply", "r76b-apply-no-prefs", {
      reason: String(reason || "")
    });
    return null;
  }

  const applied = [];
  const errors = [];

  try {
    if (window.kgwShellApplyDisplayPreferencesDirectR73 && typeof window.kgwShellApplyDisplayPreferencesDirectR73 === "function") {
      window.kgwShellApplyDisplayPreferencesDirectR73(prefs, reason);
      applied.push("direct-r76b");
    } else {
      errors.push("direct-r76b:missing-window-owner");
    }
  } catch (error) {
    errors.push("direct-r76b:" + String(error && error.message || error));
  }

  kgwSettingsUiTraceR48B3("settings-display-apply", "r76b-direct-shell-apply", {
    reason: String(reason || ""),
    languages: Array.isArray(prefs.languages) ? prefs.languages.join(",") : "",
    currencies: Array.isArray(prefs.currencies) ? prefs.currencies.join(",") : "",
    tabs: Array.isArray(prefs.tabs) ? prefs.tabs.join(",") : "",
    applied: applied.join(","),
    errors: errors.join(" | ")
  });

  return prefs;
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

const KGW_SETTINGS_WORKFLOW_STATE = {
  backendSettings: null,
  busy: false
};

function kgwSettingsDialogApi() {
  return window.__TAURI__?.dialog || null;
}

function kgwSettingsProfileSetStatus(message, kind = "info") {
  let status = q("#settingsProfileStatus");
  const profile = q("#settingsApiProfile");
  if (!status && profile) {
    status = document.createElement("div");
    status.id = "settingsProfileStatus";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    profile.parentElement?.appendChild(status);
  }

  if (status) {
    status.textContent = String(message || "");
    status.dataset.kind = kind;
    applyStatusTone(status, kind);
  }
}

function kgwSettingsActiveBackendProfile(settings) {
  const list = Array.isArray(settings?.api_profiles) ? settings.api_profiles : [];
  return list.find((profile) => profile?.name === settings?.active_api_profile) || list[0] || null;
}

function kgwSettingsUpdateEndpointResetAvailability(row) {
  const button = q("#settingsResetSelectedEndpoint");
  if (!button) return;
  const key = String(row?.dataset?.apiKey || "");
  const supported = key === "base_url" || row?.dataset?.kgwPersistedEndpoint === "true";
  button.disabled = !supported;
  button.setAttribute("aria-disabled", String(!supported));
  button.title = supported ? "Reset the selected persisted endpoint to its canonical default." : "This endpoint is not stored by the current Settings contract.";
}

function kgwSettingsApplyBackendProfiles(settings) {
  if (!settings || typeof settings !== "object") return;
  KGW_SETTINGS_WORKFLOW_STATE.backendSettings = settings;
  const select = q("#settingsApiProfile");
  const profiles = Array.isArray(settings.api_profiles) ? settings.api_profiles : [];
  if (select) {
    select.innerHTML = "";
    profiles.forEach((profile) => {
      const option = document.createElement("option");
      option.value = profile.name;
      option.textContent = profile.name;
      select.appendChild(option);
    });
    select.value = settings.active_api_profile || profiles[0]?.name || "";
  }

  const active = kgwSettingsActiveBackendProfile(settings);
  const endpointMap = new Map((active?.endpoints || []).map((endpoint) => [endpoint.name, endpoint]));
  qa(".tree-row[data-api-key]").forEach((row) => {
    const key = String(row.dataset.apiKey || "");
    row.dataset.kgwPersistedEndpoint = "false";
    if (key === "base_url" && active) {
      row.dataset.apiBase = active.base_url || "";
      row.dataset.apiPath = "";
      row.dataset.kgwPersistedEndpoint = "true";
      return;
    }
    const endpoint = endpointMap.get(key);
    if (endpoint && active) {
      row.dataset.apiBase = active.base_url || "";
      row.dataset.apiPath = endpoint.path || "";
      row.dataset.kgwPersistedEndpoint = "true";
    }
  });
  const selected = q(".tree-row.is-selected") || q(".tree-row");
  if (selected) selectEndpoint(selected);
}

async function kgwSettingsRefreshBackendProfiles(reason = "refresh") {
  try {
    const settings = await kgwSettingsBackendInvokeR4("settings_load");
    kgwSettingsApplyBackendProfiles(settings);
    settingsLogger().log("settings backend profiles refreshed", { reason });
    return settings;
  } catch (error) {
    kgwSettingsProfileSetStatus(`Profile load failed: ${error?.message || error}`, "error");
    throw error;
  }
}

async function kgwSettingsRunProfileMutation(command, payload, successMessage) {
  if (KGW_SETTINGS_WORKFLOW_STATE.busy) return null;
  KGW_SETTINGS_WORKFLOW_STATE.busy = true;
  kgwSettingsProfileSetStatus("Working...", "loading");
  try {
    const settings = await kgwSettingsBackendInvokeR4(command, payload);
    kgwSettingsApplyBackendProfiles(settings);
    kgwSettingsProfileSetStatus(successMessage, "success");
    return settings;
  } catch (error) {
    kgwSettingsProfileSetStatus(`Action failed: ${error?.message || error}`, "error");
    return null;
  } finally {
    KGW_SETTINGS_WORKFLOW_STATE.busy = false;
  }
}

async function kgwSettingsProfileAddAction() {
  const name = window.prompt("New API profile name (letters, numbers, dot, dash or underscore):", "");
  if (name == null) return;
  await kgwSettingsRunProfileMutation("settings_profile_add", { name }, "API profile added and saved.");
}

async function kgwSettingsProfileRenameAction() {
  const select = q("#settingsApiProfile");
  const current = String(select?.value || "");
  if (!current) return kgwSettingsProfileSetStatus("Select an API profile first.", "error");
  const newName = window.prompt("New API profile name:", current);
  if (newName == null || newName === current) return;
  await kgwSettingsRunProfileMutation("settings_profile_rename", { name: current, newName }, "API profile renamed and saved.");
}

async function kgwSettingsProfileDeleteAction() {
  const select = q("#settingsApiProfile");
  const name = String(select?.value || "");
  if (!name) return kgwSettingsProfileSetStatus("Select an API profile first.", "error");
  if (!await confirmUserAction(`Delete API profile "${name}"?`)) return;
  await kgwSettingsRunProfileMutation("settings_profile_delete", { name }, "API profile deleted and saved.");
}

async function kgwSettingsProfileSelectAction() {
  const select = q("#settingsApiProfile");
  const name = String(select?.value || "");
  if (!name) return;
  await kgwSettingsRunProfileMutation("settings_profile_select", { name }, "Active API profile saved.");
}

async function kgwSettingsResetSelectedEndpointAction() {
  const row = q(".tree-row.is-selected");
  const profileName = String(q("#settingsApiProfile")?.value || "");
  const endpointName = String(row?.dataset?.apiKey || "");
  if (!row || !profileName || !endpointName) {
    kgwSettingsProfileSetStatus("Select a persisted endpoint first.", "error");
    return;
  }
  await kgwSettingsRunProfileMutation(
    "settings_reset_selected_endpoint",
    { profileName, endpointName },
    "Selected endpoint reset to its canonical default and saved."
  );
}

async function kgwSettingsRefreshAddressesLocalOnly() {
  const invoke = kgwSettingsAddressInvoke();
  if (!invoke) throw new Error("Tauri invoke API is not available.");
  const records = await invoke("get_all_addresses");
  await kgwRenderSettingsAddressRows(records, { localOnly: true });
  if (typeof window.kgwRefreshSavedAddresses === "function") {
    await window.kgwRefreshSavedAddresses();
  }
  try {
    window.dispatchEvent(new CustomEvent("kgw:saved-addresses-changed"));
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
  return records;
}

function kgwSettingsAddressIoStatus(report, action) {
  const warnings = Array.isArray(report?.warnings) ? report.warnings : [];
  const imported = Number(report?.imported || 0);
  const exported = Number(report?.exported || 0);
  const skipped = Number(report?.skipped || 0);
  const count = action === "import" ? `${imported} imported, ${skipped} skipped` : `${exported} exported`;
  const warningText = warnings.length ? `; ${warnings.join(" | ")}` : "";
  kgwSettingsAddressSetStatus(`Last Updated: ${count}${warningText}`, warnings.length ? "warning" : "success");
}

async function kgwSettingsExportAddressesAction() {
  const dialog = kgwSettingsDialogApi();
  const invoke = kgwSettingsAddressInvoke();
  if (!dialog || typeof dialog.save !== "function" || !invoke) {
    kgwSettingsAddressSetStatus("Last Updated: native save dialog is unavailable.", "error");
    return;
  }

  try {
    const stats = await invoke("address_book_stats");
    const selected = await dialog.save({
      title: "Export saved Kaspa addresses",
      defaultPath: stats?.default_export_json_path || "kaspa_gateway_addresses.json",
      filters: [
        { name: "JSON", extensions: ["json"] },
        { name: "CSV", extensions: ["csv"] }
      ]
    });
    if (!selected) {
      kgwSettingsAddressSetStatus("Last Updated: export cancelled.");
      return;
    }
    const isCsv = String(selected).toLowerCase().endsWith(".csv");
    const command = isCsv ? "address_book_export_csv" : "address_book_export_json";
    const report = await invoke(command, { request: { path: String(selected), network: "mainnet" } });
    kgwSettingsAddressIoStatus(report, "export");
  } catch (error) {
    kgwSettingsAddressSetStatus(`Last Updated: export failed - ${error?.message || error}`, "error");
  }
}

async function kgwSettingsImportAddressesAction() {
  const dialog = kgwSettingsDialogApi();
  const invoke = kgwSettingsAddressInvoke();
  if (!dialog || typeof dialog.open !== "function" || !invoke) {
    kgwSettingsAddressSetStatus("Last Updated: native open dialog is unavailable.", "error");
    return;
  }

  try {
    const selected = await dialog.open({
      title: "Import saved Kaspa addresses",
      multiple: false,
      directory: false,
      filters: [
        { name: "Address files", extensions: ["json", "csv"] }
      ]
    });
    const path = Array.isArray(selected) ? selected[0] : selected;
    if (!path) {
      kgwSettingsAddressSetStatus("Last Updated: import cancelled.");
      return;
    }
    const isCsv = String(path).toLowerCase().endsWith(".csv");
    const command = isCsv ? "address_book_import_csv" : "address_book_import_json";
    const report = await invoke(command, { request: { path: String(path), network: "mainnet" } });
    await kgwSettingsRefreshAddressesLocalOnly();
    kgwSettingsAddressIoStatus(report, "import");
  } catch (error) {
    kgwSettingsAddressSetStatus(`Last Updated: import failed - ${error?.message || error}`, "error");
  }
}

function kgwInstallSettingsRealWorkflowActions() {
  if (window.__kgwSettingsRealWorkflowActionsInstalled) return;
  window.__kgwSettingsRealWorkflowActionsInstalled = true;

  const bindings = [
    ["settingsProfileAdd", kgwSettingsProfileAddAction],
    ["settingsProfileRename", kgwSettingsProfileRenameAction],
    ["settingsProfileDelete", kgwSettingsProfileDeleteAction],
    ["settingsResetSelectedEndpoint", kgwSettingsResetSelectedEndpointAction],
    ["settingsExportAddresses", kgwSettingsExportAddressesAction],
    ["settingsImportAddresses", kgwSettingsImportAddressesAction]
  ];
  bindings.forEach(([id, handler]) => {
    const button = q(`#${CSS.escape(id)}`);
    if (!button || button.dataset.kgwRealWorkflowBound === "true") return;
    button.dataset.kgwRealWorkflowBound = "true";
    button.addEventListener("click", (event) => {
      event.preventDefault();
      void handler();
    });
  });

  const select = q("#settingsApiProfile");
  if (select && select.dataset.kgwProfileSelectBound !== "true") {
    select.dataset.kgwProfileSelectBound = "true";
    select.addEventListener("change", () => void kgwSettingsProfileSelectAction());
  }
  const addressStatus = q("#settingsAddressLastUpdated");
  if (addressStatus) {
    addressStatus.setAttribute("role", "status");
    addressStatus.setAttribute("aria-live", "polite");
  }
  void kgwSettingsRefreshBackendProfiles("workflow-install");
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

/* KGW_SETTINGS_SINGLE_SAVE_FLIGHT_R75 */
let kgwSettingsSaveInFlightR75 = false;

async function saveState() {
  if (kgwSettingsSaveInFlightR75) {
    kgwSettingsUiTraceR48B3("settings-action", "r75-save-skip-busy", {
      targetId: "settingsSaveSettings"
    });
    return false;
  }

  kgwSettingsSaveInFlightR75 = true;

  kgwSettingsUiTraceR48B3("settings-action", "r75-save-begin", {
    targetId: "settingsSaveSettings"
  });

  try {
    await kgwSettingsRepairDynamicPathsBeforeSaveR4();

    if (!kgwDisplaySelectionValidateForSaveR1()) {
      kgwSettingsUiTraceR48B3("settings-action", "r75-save-validation-failed", {
        targetId: "settingsSaveSettings"
      });
      setSaveEnabled(true);
      return false;
    }

    const collected = collectState();
    const state = kgwSettingsReapplyDisplayStateR65(collected, "settings-save-r75");

    try {
      localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(state, null, 2));
    } catch (error) {
      kgwSettingsUiTraceR48B3("settings-action", "r75-save-storage-error", {
        message: String(error && error.message || error)
      });
    }

    const prefs = kgwSettingsApplyShellDisplayFromState(state, "settings-save-r75");

    setSaveEnabled(false);
    settingsLogger().log("settings saved", {
      patch: "R75",
      owner: "settings-single-save-direct-shell"
    });

    kgwSettingsUiTraceR48B3("settings-action", "r75-save-complete", {
      targetId: "settingsSaveSettings",
      hasPrefs: Boolean(prefs),
      languages: prefs && Array.isArray(prefs.languages) ? prefs.languages.join(",") : "",
      currencies: prefs && Array.isArray(prefs.currencies) ? prefs.currencies.join(",") : "",
      tabs: prefs && Array.isArray(prefs.tabs) ? prefs.tabs.join(",") : ""
    });

    return true;
  } catch (error) {
    kgwSettingsUiTraceR48B3("settings-action", "r75-save-error", {
      targetId: "settingsSaveSettings",
      message: String(error && error.message || error)
    });
    try {
      console.error("[settings] save failed", error);
    } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
    setSaveEnabled(true);
    return false;
  } finally {
    kgwSettingsSaveInFlightR75 = false;
  }
}

async function resetDefaults(options = {}) {
  const shouldClearStorage = options.clearStorage !== false;
  const shouldApplyShell = options.applyShell !== false;

  if (shouldClearStorage) {
    try {
      localStorage.removeItem(SETTINGS_STORAGE_KEY);
    } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
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
    settingsApiRetries: "3",
    settingsApiRateLimit: "60",
    settingsCacheTtl: "300",
    settingsMaxConnections: "10",
    settingsRequestTimeout: "30"
  };

  Object.entries(defaults).forEach(([id, value]) => {
    const node = q(`#${CSS.escape(id)}`);
    if (node) node.value = value;
  });

  activateOuter("api-performance");
  activateInner("general");

  const state = kgwSettingsBuildCanonicalDefaultStateR65("settings-reset-defaults-r69", shouldClearStorage);
  kgwDisplaySelectionEnsureDefaultsR1("reset-defaults-r69-guard");

  if (shouldApplyShell) {
    kgwSettingsApplyShellDisplayFromState(state, "settings-reset-defaults-r69");
  }

  setSaveEnabled(false);

  await kgwSettingsLoadDynamicPathDefaultsR4("reset-defaults");
}

function kgwSettingsPathStatus(targetId, message, state = "info") {
  const target = q(`#${CSS.escape(targetId)}`);
  if (!target) return;
  let status = target.parentElement?.querySelector(`[data-settings-path-status="${targetId}"]`);
  if (!status) {
    status = document.createElement("span");
    status.dataset.settingsPathStatus = targetId;
    status.setAttribute("role", state === "error" ? "alert" : "status");
    status.setAttribute("aria-live", state === "error" ? "assertive" : "polite");
    target.parentElement?.appendChild(status);
  }
  status.dataset.state = state;
  status.textContent = String(message || "");
  applyStatusTone(status, state);
}

async function kgwSettingsBrowsePath(targetId) {
  const target = q(`#${CSS.escape(targetId)}`);
  const dialog = kgwSettingsDialogApi();
  if (!target || !dialog || typeof dialog.open !== "function") {
    kgwSettingsPathStatus(targetId, "Native directory chooser is unavailable.", "error");
    return false;
  }
  const before = String(target.value || "");
  try {
    const selected = await dialog.open({ title: "Choose directory", directory: true, multiple: false });
    const path = Array.isArray(selected) ? selected[0] : selected;
    if (!path) {
      kgwSettingsPathStatus(targetId, "Browse cancelled; path unchanged.", "info");
      return false;
    }
    const report = await kgwSettingsBackendInvokeR4("settings_validate_custom_path", {
      key: targetId,
      path: String(path),
      createIfMissing: false
    });
    target.value = String(report?.path || path);
    kgwSettingsPathStatus(targetId, "Directory selected.", "success");
    markDirty();
    return true;
  } catch (error) {
    target.value = before;
    kgwSettingsPathStatus(targetId, `Browse failed: ${error?.message || error}`, "error");
    return false;
  }
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
      event.preventDefault();
      kgwSettingsUiTraceR48B3("settings-field-action", "r48b3-browse-for-click", {
        trusted: Boolean(event && event.isTrusted),
        target: String(button.dataset.browseFor || ""),
        text: String(button.textContent || "").trim()
      });
      void kgwSettingsBrowsePath(String(button.dataset.browseFor || ""));
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
    save.dataset.kgwSettingsSaveOwner = "R75";
    save.addEventListener("click", async (event) => {
      if (event && typeof event.preventDefault === "function") event.preventDefault();

      kgwSettingsUiTraceR48B3("settings-action", "r75-save-press", {
        trusted: Boolean(event && event.isTrusted),
        targetId: "settingsSaveSettings",
        disabled: Boolean(save.disabled)
      });

      if (save.disabled) {
        kgwSettingsUiTraceR48B3("settings-action", "r75-save-skip-disabled", {
          targetId: "settingsSaveSettings"
        });
        return;
      }

      await saveState();
    });
  }

  const placeholderActions = [
    "settingsAddressAdd",
    "settingsAddressDelete",
    "settingsAddressClear",
    "settingsAddressRefresh",
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

/* KGW_SETTINGS_STALE_DISPLAY_STATE_NORMALIZER_R63F
 * Normalizes stale saved display selections that predate the canonical R59C shell owner.
 */
function loadSavedState() {
  /* KGW_SETTINGS_FULL_PERSISTENCE_CONTRACT_FIX_R104
   * Valid user-saved state is authoritative.
   * Do not collapse all-selected choices into defaults.
   */
  try {
    const saved = JSON.parse(localStorage.getItem(SETTINGS_STORAGE_KEY) || "null");
    if (saved) {
      if (kgwSettingsDisplayStateLooksLegacyAllSelectedR65(saved)) {
        const state = kgwSettingsBuildCanonicalDefaultStateR65("settings-load-missing-display-contract-r104", true);
        window.setTimeout(() => kgwSettingsApplyShellDisplayFromState(state, "settings-load-missing-display-contract-r104"), 0);
        void kgwSettingsLoadDynamicPathDefaultsR4("settings-load-missing-display-contract-r104");
        return;
      }

      applyState(saved);
      const state = kgwSettingsReapplyDisplayStateR65(saved, "settings-load-r104");

      window.setTimeout(() => kgwSettingsApplyShellDisplayFromState(state, "settings-load-r104"), 0);
      void kgwSettingsLoadDynamicPathDefaultsR4("settings-load-r104");
      return;
    }

    kgwSettingsBuildCanonicalDefaultStateR65("settings-load-defaults-r104", true);
    void kgwSettingsLoadDynamicPathDefaultsR4("settings-load-defaults-r104");
  } catch (_) {
    kgwSettingsBuildCanonicalDefaultStateR65("settings-load-error-defaults-r104", false);
    void kgwSettingsLoadDynamicPathDefaultsR4("settings-load-error-r104");
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
  kgwInstallSettingsRealWorkflowActions();
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
async function kgwRefreshSettingsDatabaseStatus() {
  return await settingsDatabaseRefresh();
}

function kgwInstallSettingsDatabaseStatus() {
  return settingsDatabaseInstall();
}

window.kgwRefreshSettingsDatabaseStatus = kgwRefreshSettingsDatabaseStatus;
window.kgwInstallSettingsDatabaseStatus = kgwInstallSettingsDatabaseStatus;

kgwInstallSettingsDatabaseStatus();

/* KGW settings managed-address compatibility surface: implementation is Rust/WASM-owned. */
function kgwSettingsAddressInvoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke || window.__TAURI_INVOKE__;
}

function kgwSettingsAddressSetStatus(message, state = "info") {
  return settingsAddressesSetStatus(message, state);
}

async function kgwRenderSettingsAddressRows(records, options = {}) {
  return await settingsAddressesRenderRows(records, options);
}

async function kgwRefreshSettingsAddresses() {
  return await settingsAddressesRefresh();
}

function kgwOpenKaspaAddressInBrowser(address) {
  return settingsAddressesOpenExplorer(address);
}

window.kgwRefreshSettingsAddresses = kgwRefreshSettingsAddresses;
window.kgwOpenKaspaAddressInBrowser = kgwOpenKaspaAddressInBrowser;

settingsAddressesInstallAll();

/* KGW fixed Manage Addresses layout: clean, no horizontal scroll by default. */
function kgwInstallSettingsManageAddressesCleanLayout() {
  return installManageAddressesCleanLayout();
}

window.kgwInstallSettingsManageAddressesCleanLayout = kgwInstallSettingsManageAddressesCleanLayout;
kgwInstallSettingsManageAddressesCleanLayout();
installSettingsI18nBindings();

/* KGW Settings database maintenance compatibility surface: implementation is Rust/WASM-owned. */
function kgwInstallSettingsDbMaintenanceActions() {
  return settingsDatabaseInstallMaintenance();
}

window.kgwInstallSettingsDbMaintenanceActions = kgwInstallSettingsDbMaintenanceActions;
kgwInstallSettingsDbMaintenanceActions();

/* KGW_SETTING_FIELD_V2: presentation-only contextual help for Global Settings. */
const kgwGlobalSettingsRootV2 = root();
if (kgwGlobalSettingsRootV2) installGlobalSettingsLayout(kgwGlobalSettingsRootV2);
else document.addEventListener("DOMContentLoaded", () => installGlobalSettingsLayout(root()), { once: true });
