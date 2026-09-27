import {
  settingsAddressesInstallAll,
  settingsAddressesInstallIo,
  settingsAddressesOpenExplorer,
  settingsAddressesRefresh,
  settingsDatabaseInstall,
  settingsDatabaseInstallMaintenance,
  settingsDatabaseRefresh,
  settingsDiagnosticsInstall,
  settingsDisplayApplyShellFromState,
  settingsDisplayBindMinimumGuards,
  settingsDisplayBuildCanonicalDefaultState,
  settingsDisplayEnsureDefaults,
  settingsDisplayReapplyState,
  settingsDisplayStateLooksLegacyAllSelected,
  settingsDisplayValidateForSave,
  settingsStateActivateInner,
  settingsStateActivateOuter,
  settingsStateApply,
  settingsStateCollect,
  settingsStateCombineUrl,
  settingsPathsBrowse,
  settingsPathsLoadDefaults,
  settingsPathsRepairBeforeSave,
  settingsProfilesInstall,
  settingsProfilesSelectEndpoint,
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
  return settingsStateActivateOuter(tab);
}

function activateInner(tab) {
  return settingsStateActivateInner(tab);
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
  return settingsStateCombineUrl();
}
function selectEndpoint(row) {
  return settingsProfilesSelectEndpoint(row);
}


/* Settings display/state DOM ownership is Rust/WASM-owned. */
function kgwDisplaySelectionEnsureDefaultsR1(reason = "default") {
  return settingsDisplayEnsureDefaults(reason);
}

function kgwDisplaySelectionValidateForSaveR1() {
  return settingsDisplayValidateForSave();
}

function kgwDisplaySelectionBindMinimumGuardsR1() {
  return settingsDisplayBindMinimumGuards();
}

function kgwSettingsDisplayStateLooksLegacyAllSelectedR65(state) {
  return settingsDisplayStateLooksLegacyAllSelected(state);
}

function kgwSettingsBuildCanonicalDefaultStateR65(reason = "display-defaults-r65", persist = false) {
  return settingsDisplayBuildCanonicalDefaultState(reason, persist);
}

function kgwSettingsReapplyDisplayStateR65(state, reason = "display-state-r65") {
  return settingsDisplayReapplyState(state, reason);
}

function collectState() {
  return settingsStateCollect();
}

function applyState(state) {
  return settingsStateApply(state);
}

function kgwSettingsApplyShellDisplayFromState(state, reason = "settings") {
  return settingsDisplayApplyShellFromState(state, reason);
}
function kgwInstallSettingsRealWorkflowActions() {
  settingsAddressesInstallIo();
  settingsProfilesInstall();
}

async function kgwSettingsLoadDynamicPathDefaultsR4(reason = "settings") {
  return await settingsPathsLoadDefaults(reason);
}

async function kgwSettingsRepairDynamicPathsBeforeSaveR4() {
  return await settingsPathsRepairBeforeSave();
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
      void settingsPathsBrowse(String(button.dataset.browseFor || ""));
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

  settingsDiagnosticsInstall();

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
