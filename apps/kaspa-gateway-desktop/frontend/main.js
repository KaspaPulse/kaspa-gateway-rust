import { KGW_TABS } from "./src/tabs/tab-registry.js";
import initShellWasm, {
  shellAuxInstall as wasmShellAuxInstall,
  shellRuntimeActivateTab as wasmShellActivateTab,
  shellRuntimeApplyTheme as wasmShellApplyTheme,
  shellRuntimeInstall as wasmShellInstall,
  shellRuntimeOpenTab as wasmShellOpenTab,
  shellRuntimeSavedMainTab as wasmShellSavedMainTab,
  shellRuntimeScheduleSavedRestore as wasmShellScheduleSavedRestore,
  shellRuntimeTraceTab as wasmShellTraceTab
} from "./generated/kgw_frontend_wasm/kgw_frontend_wasm.js";

await initShellWasm();
wasmShellInstall(KGW_TABS);
wasmShellAuxInstall();

async function openTab(tabId, options = {}) {
  return await wasmShellOpenTab(String(tabId || ""), options || {});
}

function activateTab(tabId) {
  return wasmShellActivateTab(String(tabId || ""));
}

function applyTheme(theme) {
  return wasmShellApplyTheme(String(theme || ""));
}

function kgwShellScheduleSavedMainTabRestoreR102C(reason = "schedule") {
  return wasmShellScheduleSavedRestore(String(reason || "schedule"));
}

function kgwShellSavedMainTabR102C() {
  return wasmShellSavedMainTab();
}

function kgwMainTabTraceR35C(tabId, phase, details = {}) {
  return wasmShellTraceTab(String(tabId || ""), String(phase || ""), details || {});
}

window.kgwOpenTab = openTab;
window.kgwActivateTab = activateTab;
window.kgwApplyTheme = applyTheme;

import("./src/core/header-live-metrics.js");


/* KGW_R71_SHELL_DISPLAY_PREFERENCES_OWNER_SAFE */
(function installKgwShellDisplayPreferencesOwnerR71() {
  if (window.kgwShellDisplayPreferencesR71) return;

  const storageKey = "kgw.shell.display.preferences.v71";
  const canonicalSettingsKey = "kgw-settings-python-exact-state";

  /* KGW_SETTINGS_DISPLAY_SOURCE_BASED_FIX_R2
   * main.js remains the shell display application owner.
   * It reads canonical Settings state first, then falls back to the legacy shell display key.
   */

  const languageOptions = [
    ["en", "English"], ["ar", "Arabic"], ["de", "German"], ["es", "Spanish"],
    ["fr", "French"], ["hi", "Hindi"], ["id", "Indonesian"], ["ja", "Japanese"],
    ["ko", "Korean"], ["ru", "Russian"], ["tr", "Turkish"], ["zh-CN", "Chinese (Simplified)"]
  ];

  const currencyOptions = [
    ["USD", "USD"], ["SAR", "SAR"], ["EUR", "EUR"], ["GBP", "GBP"],
    ["CAD", "CAD"], ["AUD", "AUD"], ["CHF", "CHF"], ["JPY", "JPY"],
    ["KRW", "KRW"], ["CNY", "CNY"], ["TRY", "TRY"], ["RUB", "RUB"],
    ["INR", "INR"], ["IDR", "IDR"], ["SGD", "SGD"], ["BRL", "BRL"], ["HKD", "HKD"]
  ];

  const tabOptions = [
    ["explorer", "Explorer"],
    ["kaspa-node", "Kaspa Node"],
    ["kaspa-bridge", "Kaspa Bridge"],
    ["analysis", "Analysis"],
    ["top-addresses", "Top Addresses"],
    ["log", "Log"]
  ];

  function keys(options) {
    return options.map((item) => item[0]);
  }

  function defaults() {
    /* KGW_SHELL_DISPLAY_AND_ACTIVE_TAB_OWNER_R59C
     * Canonical defaults owned by the single shell display and active-tab owner.
     */
    return {
      languages: ["en"],
      currencies: ["USD"],
      tabs: ["kaspa-node", "kaspa-bridge", "settings"],
      activeTab: "kaspa-node"
    };
  }

  function uniqueKnown(values, knownValues, fallbackValues) {
    const known = new Set(knownValues);
    const clean = Array.from(new Set(Array.isArray(values) ? values : []))
      .filter((value) => known.has(value));

    return clean.length > 0 ? clean : [fallbackValues[0]];
  }

  function normalize(input) {
    /* KGW_SETTINGS_FULL_PERSISTENCE_CONTRACT_FIX_R104
     * R71 previously validated saved values against defaults only.
     * That dropped user-saved languages/currencies/tabs at boot.
     * Validate against all known options and use defaults only as fallback.
     */
    const base = defaults();
    const prefs = input && typeof input === "object" ? input : base;
    const knownLanguages = keys(languageOptions);
    const knownCurrencies = keys(currencyOptions);
    const knownTabs = Array.from(new Set(keys(tabOptions).concat(["settings"])));

    const normalized = {
      languages: uniqueKnown(prefs.languages, knownLanguages, base.languages),
      currencies: uniqueKnown(prefs.currencies, knownCurrencies, base.currencies),
      tabs: uniqueKnown(prefs.tabs, knownTabs, base.tabs)
    };

    if (!normalized.tabs.includes("settings")) {
      normalized.tabs.push("settings");
    }

    return normalized;
  }

  function readCanonicalSettingsState() {
    /* KGW_SETTINGS_FULL_PERSISTENCE_CONTRACT_FIX_R104
     * Canonical Settings state is authoritative when it contains any valid display keys.
     * A user intentionally selecting all options is valid and must not be reduced to defaults.
     */
    try {
      const saved = JSON.parse(localStorage.getItem(canonicalSettingsKey) || "null");
      const checks = saved && typeof saved === "object" ? saved.checks : null;
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

      const normalized = normalize(prefs);
      const hasDisplayContract =
        prefs.languages.length > 0 &&
        prefs.currencies.length > 0 &&
        prefs.tabs.length > 0;

      return hasDisplayContract ? normalized : null;
    } catch {
      return null;
    }
  }

  function readLegacyDisplayPreferences() {
    try {
      return normalize(JSON.parse(localStorage.getItem(storageKey) || "null"));
    } catch {
      return defaults();
    }
  }

  function read() {
    return readCanonicalSettingsState() || readLegacyDisplayPreferences();
  }

  function save(prefs) {
    const normalized = normalize(prefs);
    localStorage.setItem(storageKey, JSON.stringify(normalized));
    return normalized;
  }

  function rebuildSelect(selectId, options, selectedValues) {
    const select = document.getElementById(selectId);
    if (!select) return;

    const previous = select.value;
    const allowed = selectedValues.length > 0 ? selectedValues : [options[0][0]];
    const currentValues = Array.from(select.options).map((option) => option.value);
    const desiredValues = options.filter(([value]) => allowed.includes(value)).map(([value]) => value);

    if (currentValues.join("|") !== desiredValues.join("|")) {
      const fragment = document.createDocumentFragment();

      for (const [value, label] of options) {
        if (!allowed.includes(value)) continue;

        const option = document.createElement("option");
        option.value = value;
        option.textContent = label;
        fragment.appendChild(option);
      }

      select.replaceChildren(fragment);
    }

    if (allowed.includes(previous)) {
      select.value = previous;
    } else {
      select.value = allowed[0];
      select.dispatchEvent(new Event("change", { bubbles: true }));
    }
  }

  /* KGW_SHELL_DISPLAY_AND_ACTIVE_TAB_OWNER_R59C
 * Canonical shell display and active-tab owner.
 * This is the single owner for visible tabs and active tab normalization.
 */
function kgwShellDisplayOwnerTabButtonsR59C() {
  return Array.from(document.querySelectorAll("[data-tab]"));
}

function kgwShellDisplayOwnerTabVisibleR59C(button) {
  if (!button) return false;
  if (button.hidden) return false;
  if (button.dataset && button.dataset.kgwDisplayVisible === "false") return false;
  if (button.getAttribute("aria-hidden") === "true") return false;
  if (button.style && button.style.display === "none") return false;
  return true;
}

function kgwShellDisplayOwnerTabIdVisibleR59C(tabId) {
  const requested = String(tabId || "");
  if (!requested) return false;

  const buttons = kgwShellDisplayOwnerTabButtonsR59C();
  if (!buttons.length) return true;

  const matching = buttons.filter((button) => button && button.dataset && button.dataset.tab === requested);
  if (!matching.length) return true;

  return matching.some((button) => kgwShellDisplayOwnerTabVisibleR59C(button));
}

function kgwShellDisplayOwnerPreferredTabIdR59C() {
  const preferredOrder = ["kaspa-node", "kaspa-bridge", "settings"];
  const buttons = kgwShellDisplayOwnerTabButtonsR59C();

  for (const tabId of preferredOrder) {
    const button = buttons.find((item) => item && item.dataset && item.dataset.tab === tabId && kgwShellDisplayOwnerTabVisibleR59C(item));
    if (button) return tabId;
  }

  const firstVisible = buttons.find((item) => item && item.dataset && item.dataset.tab && kgwShellDisplayOwnerTabVisibleR59C(item));
  return firstVisible && firstVisible.dataset ? firstVisible.dataset.tab : defaults().activeTab;
}

function kgwShellDisplayOwnerResolveTabIdR59C(tabId, reason = "resolve") {
  const requested = String(tabId || "").trim();

  if (requested && kgwShellDisplayOwnerTabIdVisibleR59C(requested)) {
    return requested;
  }

  const fallback = kgwShellDisplayOwnerPreferredTabIdR59C();

  try {
    console.info("[KGW][shell][canonical-tab-resolved]", {
      patch: "R59C",
      reason,
      requestedTab: requested || null,
      fallbackTab: fallback || null
    });
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }

  return fallback || requested || defaults().activeTab;
}

function kgwShellDisplayOwnerActiveButtonR59C() {
  return kgwShellDisplayOwnerTabButtonsR59C().find((button) => (
    button &&
    (
      button.classList.contains("active") ||
      button.classList.contains("is-active") ||
      button.getAttribute("aria-selected") === "true" ||
      button.getAttribute("aria-current") === "page"
    )
  )) || null;
}

function kgwShellDisplayOwnerEnsureActiveTabR59C(reason = "ensure-active") {
  const activeButton = kgwShellDisplayOwnerActiveButtonR59C();
  const activeTabId = activeButton && activeButton.dataset ? activeButton.dataset.tab : "";
  const savedTabId = kgwShellSavedMainTabR102C();
  let resolvedTabId = kgwShellDisplayOwnerResolveTabIdR59C(activeTabId, reason);

  /* R101Y_SAFE_USER_LOCATION_PERSISTENCE_FIX */
  /* KGW_SHELL_ANY_SAVED_MAIN_TAB_RESTORE_R102C
   * Do not force kaspa-node while a saved known tab is temporarily hidden during boot/settings display load.
   */
  if ((!activeTabId || resolvedTabId !== activeTabId) && savedTabId) {
    resolvedTabId = savedTabId;
  }

  if (!resolvedTabId || resolvedTabId === activeTabId) {
    kgwShellScheduleSavedMainTabRestoreR102C("ensure-active-no-change");
    return false;
  }

  window.setTimeout(() => {
    try {
      void openTab(resolvedTabId, {
        persist: false,
        allowHiddenSavedTab: Boolean(savedTabId && savedTabId === resolvedTabId),
        reason: savedTabId && savedTabId === resolvedTabId ? "display-owner-restore-saved-tab" : "display-owner-ensure-active"
      });
    } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
  }, 0);

  kgwShellScheduleSavedMainTabRestoreR102C("ensure-active-after-schedule");
  return true;
}

function kgwShellDisplayOwnerPublishR59C() {
  try {
    window.kgwShellDisplayAndActiveTabOwnerR59C = {
      marker: "KGW_SHELL_DISPLAY_AND_ACTIVE_TAB_OWNER_R59C",
      defaults,
      read,
      save,
      applyPreferences: apply,
      resolveTabId: kgwShellDisplayOwnerResolveTabIdR59C,
      ensureActiveTab: kgwShellDisplayOwnerEnsureActiveTabR59C
    };
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
}

/* KGW_SHELL_DIRECT_DISPLAY_APPLY_OWNER_R73
 * main.js remains the runtime owner for visible top tabs and shell dropdown filtering.
 * Settings may request this owner, but main.js performs the actual DOM update.
 */
/* KGW_SHELL_ACTUAL_DOM_OWNER_R75
 * Canonical runtime DOM application for display preferences.
 * Strengthens the existing main.js shell owner instead of adding a parallel UI layer.
 */
function kgwShellAsArrayR75(value, fallback) {
  const list = Array.isArray(value) ? value : fallback;
  return Array.from(new Set(list.map((item) => String(item || "").trim()).filter(Boolean)));
}

function kgwShellKnownLanguagesR75() {
  return ["en", "ar", "de", "es", "fr", "hi", "id", "ja", "ko", "ru", "tr", "zh-CN"];
}

function kgwShellKnownCurrenciesR75() {
  return ["USD", "SAR", "EUR", "GBP", "CHF", "AUD", "CAD", "JPY", "KRW", "RUB", "CNY", "TRY", "INR", "IDR", "HKD", "SGD", "BRL"];
}

function kgwShellOptionValueR75(option) {
  return String(
    option?.value ||
    option?.dataset?.value ||
    option?.dataset?.language ||
    option?.dataset?.currency ||
    option?.getAttribute?.("data-lang") ||
    option?.getAttribute?.("data-currency") ||
    option?.textContent ||
    ""
  ).trim();
}

function kgwShellSelectLooksLikeSetR75(select, universe, kind) {
  if (!select || !select.options) return false;

  const attr = [
    select.id,
    select.name,
    select.className,
    select.getAttribute("aria-label"),
    select.getAttribute("data-testid"),
    select.dataset?.role,
    select.dataset?.kind
  ].map((item) => String(item || "").toLowerCase()).join(" ");

  if (attr.includes(kind)) return true;

  const values = Array.from(select.options).map((option) => kgwShellOptionValueR75(option)).filter(Boolean);
  if (!values.length) return false;

  const known = new Set(universe);
  const hits = values.filter((value) => known.has(value)).length;

  return hits >= 2 && hits >= Math.ceil(values.length * 0.5);
}

function kgwShellCollectSelectsForKindR75(selectors, universe, kind) {
  const nodes = [];

  selectors.forEach((selector) => {
    try {
      document.querySelectorAll(selector).forEach((node) => {
        if (node && node.tagName === "SELECT" && !nodes.includes(node)) nodes.push(node);
      });
    } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
  });

  try {
    document.querySelectorAll("select").forEach((node) => {
      if (kgwShellSelectLooksLikeSetR75(node, universe, kind) && !nodes.includes(node)) {
        nodes.push(node);
      }
    });
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }

  return nodes;
}

function kgwShellApplySelectVisibilityR75(select, allowedValues, reason) {
  const allowed = new Set((Array.isArray(allowedValues) ? allowedValues : []).map((item) => String(item || "")));
  let shown = 0;
  let hidden = 0;
  let firstAllowed = "";

  Array.from(select.options || []).forEach((option) => {
    const value = kgwShellOptionValueR75(option);
    const visible = allowed.size === 0 || allowed.has(value);

    if (visible && !firstAllowed) firstAllowed = value;

    option.hidden = !visible;
    option.disabled = !visible;

    if (visible) {
      option.style.removeProperty("display");
      shown += 1;
    } else {
      option.style.setProperty("display", "none", "important");
      hidden += 1;
    }
  });

  if (firstAllowed && !allowed.has(String(select.value || ""))) {
    select.value = firstAllowed;
    try {
      select.dispatchEvent(new Event("change", { bubbles: true }));
    } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
  }

  select.dataset.kgwDisplayReason = String(reason || "");
  select.dataset.kgwDisplayShown = String(shown);
  select.dataset.kgwDisplayHidden = String(hidden);

  return { shown, hidden };
}

function kgwShellApplyLooseMenuVisibilityR75(kind, allowedValues, reason) {
  const universe = kind === "language" ? kgwShellKnownLanguagesR75() : kgwShellKnownCurrenciesR75();
  const allowed = new Set((Array.isArray(allowedValues) ? allowedValues : []).map((item) => String(item || "")));
  const universeSet = new Set(universe);

  const selectors = kind === "language"
    ? [
        "[data-language-option]",
        "[data-lang-option]",
        "[data-lang]",
        "[data-language]",
        "[data-value]"
      ]
    : [
        "[data-currency-option]",
        "[data-currency]",
        "[data-value]"
      ];

  const nodes = [];
  selectors.forEach((selector) => {
    try {
      document.querySelectorAll(selector).forEach((node) => {
        if (node && !nodes.includes(node)) nodes.push(node);
      });
    } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
  });

  let shown = 0;
  let hidden = 0;

  nodes.forEach((node) => {
    const value = String(
      node.dataset?.value ||
      node.dataset?.language ||
      node.dataset?.currency ||
      node.getAttribute?.("data-lang") ||
      node.getAttribute?.("data-currency") ||
      ""
    ).trim();

    if (!value || !universeSet.has(value)) return;

    const visible = allowed.size === 0 || allowed.has(value);
    node.hidden = !visible;
    node.setAttribute("aria-hidden", visible ? "false" : "true");

    if (visible) {
      node.style.removeProperty("display");
      shown += 1;
    } else {
      node.style.setProperty("display", "none", "important");
      hidden += 1;
    }

    node.dataset.kgwDisplayReason = String(reason || "");
  });

  return { shown, hidden };
}

function kgwShellNormalizeDisplayPrefsR73(prefs) {
  const source = prefs && typeof prefs === "object" ? prefs : {};

  const unique = (value, fallback) => {
    const list = Array.isArray(value) ? value : fallback;
    return Array.from(new Set(list.map((item) => String(item || "").trim()).filter(Boolean)));
  };

  const tabs = unique(source.tabs, ["kaspa-node", "kaspa-bridge", "settings"]);
  if (!tabs.includes("settings")) tabs.push("settings");

  return {
    languages: unique(source.languages, ["en"]),
    currencies: unique(source.currencies, ["USD"]),
    tabs
  };
}

/* KGW_SHELL_LANGUAGE_CURRENCY_SELECT_OWNER_R78 */
function kgwShellLanguageLabelsR78() {
  return {
    "en": "English",
    "ar": "Arabic",
    "de": "German",
    "es": "Spanish",
    "fr": "French",
    "hi": "Hindi",
    "id": "Indonesian",
    "ja": "Japanese",
    "ko": "Korean",
    "ru": "Russian",
    "tr": "Turkish",
    "zh-CN": "Chinese (Simplified)"
  };
}

function kgwShellCurrencyLabelsR78() {
  return {
    "USD": "USD",
    "KAS": "KAS",
    "SAR": "SAR",
    "EUR": "EUR",
    "GBP": "GBP",
    "CHF": "CHF",
    "AUD": "AUD",
    "CAD": "CAD",
    "JPY": "JPY",
    "KRW": "KRW",
    "RUB": "RUB",
    "CNY": "CNY",
    "TRY": "TRY",
    "INR": "INR",
    "IDR": "IDR",
    "HKD": "HKD",
    "SGD": "SGD",
    "BRL": "BRL"
  };
}

function kgwShellEnsureSelectOptionR78(select, value, label, i18nKey) {
  if (!select || !value) return false;

  const existing = Array.from(select.options || []).find((option) => String(option.value || "") === String(value));
  if (existing) {
    if (!existing.textContent || existing.textContent.trim() === "") existing.textContent = label || value;
    if (i18nKey && !existing.dataset.i18n) existing.dataset.i18n = i18nKey;
    return false;
  }

  const option = document.createElement("option");
  option.value = value;
  option.textContent = label || value;
  if (i18nKey) option.dataset.i18n = i18nKey;
  select.appendChild(option);
  return true;
}

function kgwShellEnsureSelectUniverseR78(select, kind) {
  if (!select) return 0;

  const labels = kind === "language" ? kgwShellLanguageLabelsR78() : kgwShellCurrencyLabelsR78();
  let added = 0;

  Object.entries(labels).forEach(([value, label]) => {
    const i18nKey = kind === "language"
      ? (value === "zh-CN" ? "common.lang.zh.cn" : "common.lang." + value)
      : "ui.shell." + value.toLowerCase();

    if (kgwShellEnsureSelectOptionR78(select, value, label, i18nKey)) {
      added += 1;
    }
  });

  return added;
}

function kgwShellSelectIsExplicitShellOwnerR78(select, kind) {
  if (!select) return false;
  const id = String(select.id || "");
  if (kind === "language" && id === "shellLanguageSelect") return true;
  if (kind === "currency" && id === "shellCurrencySelect") return true;
  return false;
}

function kgwShellSetSelectOptionsVisibleR73(selectors, allowedValues, reason) {
  const languageUniverse = kgwShellKnownLanguagesR75();
  const currencyUniverse = kgwShellKnownCurrenciesR75();

  const allowed = Array.isArray(allowedValues) ? allowedValues.map((item) => String(item || "")) : [];
  const kind = allowed.some((value) => languageUniverse.includes(value)) ? "language" : "currency";
  const universe = kind === "language" ? languageUniverse : currencyUniverse;

  const ownerSelectors = kind === "language"
    ? ["#shellLanguageSelect"].concat(selectors || [])
    : ["#shellCurrencySelect"].concat(selectors || []);

  const nodes = kgwShellCollectSelectsForKindR75(ownerSelectors, universe, kind);

  const explicit = kind === "language"
    ? document.getElementById("shellLanguageSelect")
    : document.getElementById("shellCurrencySelect");

  if (explicit && !nodes.includes(explicit)) {
    nodes.unshift(explicit);
  }

  let shown = 0;
  let hidden = 0;
  let added = 0;

  nodes.forEach((select) => {
    if (kgwShellSelectIsExplicitShellOwnerR78(select, kind)) {
      added += kgwShellEnsureSelectUniverseR78(select, kind);
    }

    const stats = kgwShellApplySelectVisibilityR75(select, allowed, reason);
    shown += stats.shown;
    hidden += stats.hidden;
  });

  try {
    kgwMainTabTraceR35C("settings", "r78-select-options-dom", {
      reason: String(reason || ""),
      kind,
      selectCount: String(nodes.length),
      shown: String(shown),
      hidden: String(hidden),
      added: String(added),
      ownerIds: nodes.map((node) => String(node.id || "")).join(","),
      allowed: allowed.join(",")
    });
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
}

function kgwShellSetMenuOptionsVisibleR73(optionSelectors, allowedValues, reason) {
  const languageUniverse = kgwShellKnownLanguagesR75();
  const allowed = Array.isArray(allowedValues) ? allowedValues.map((item) => String(item || "")) : [];
  const kind = allowed.some((value) => languageUniverse.includes(value)) ? "language" : "currency";
  const stats = kgwShellApplyLooseMenuVisibilityR75(kind, allowed, reason);

  try {
    kgwMainTabTraceR35C("settings", "r75-menu-options-dom", {
      reason: String(reason || ""),
      kind,
      shown: String(stats.shown),
      hidden: String(stats.hidden),
      allowed: allowed.join(",")
    });
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
}

function kgwShellApplyDisplayPreferencesDirectR73(prefs, reason = "direct-shell-apply") {
  const normalized = kgwShellNormalizeDisplayPrefsR73(prefs);

  try {
    applyTabs(normalized.tabs);
  } catch (error) {
    try {
      kgwMainTabTraceR35C("settings", "r75-direct-apply-tabs-error", {
        reason: String(reason || ""),
        message: String(error && error.message || error)
      });
    } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }
  }

  kgwShellSetSelectOptionsVisibleR73([
    "#shellLanguageSelect",
    "#languageSelect",
    "#kgwLanguageSelect",
    "#appLanguageSelect",
    "#settingsLanguageSelect",
    "select[name='language']",
    "select[data-language-select]",
    "select[id*='language' i]"
  ], normalized.languages, reason);

  kgwShellSetSelectOptionsVisibleR73([
    "#shellCurrencySelect",
    "#currencySelect",
    "#kgwCurrencySelect",
    "#appCurrencySelect",
    "#settingsCurrencySelect",
    "select[name='currency']",
    "select[data-currency-select]",
    "select[id*='currency' i]"
  ], normalized.currencies, reason);

  kgwShellSetMenuOptionsVisibleR73([], normalized.languages, reason);
  kgwShellSetMenuOptionsVisibleR73([], normalized.currencies, reason);

  try {
    window.dispatchEvent(new CustomEvent("kgw:shell-display-applied-r78", {
      detail: normalized
    }));
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }

  try {
    kgwMainTabTraceR35C("settings", "r78-direct-shell-apply", {
      reason: String(reason || ""),
      languages: normalized.languages.join(","),
      currencies: normalized.currencies.join(","),
      tabs: normalized.tabs.join(",")
    });
  } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }

  return normalized;
}

try {
  window.kgwShellApplyDisplayPreferencesDirectR73 = kgwShellApplyDisplayPreferencesDirectR73;
} catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }

function applyTabs(selectedTabs) {
    const normalizedTabs = kgwShellAsArrayR75(selectedTabs, ["kaspa-node", "kaspa-bridge", "settings"]);
    if (!normalizedTabs.includes("settings")) normalizedTabs.push("settings");

    const visibleTabs = new Set(normalizedTabs);
    const tabButtons = Array.from(document.querySelectorAll("[data-tab]"));
    let shown = 0;
    let hidden = 0;

    for (const button of tabButtons) {
      const tabId = String(button.dataset.tab || "");
      const isSettings = tabId === "settings";
      const visible = isSettings || visibleTabs.has(tabId);

      button.hidden = !visible;

      if (visible) {
        button.style.removeProperty("display");
        button.classList.remove("kgw-display-hidden");
        shown += 1;
      } else {
        button.style.setProperty("display", "none", "important");
        button.classList.add("kgw-display-hidden");
        hidden += 1;
      }

      button.setAttribute("aria-hidden", visible ? "false" : "true");
      button.dataset.kgwDisplayVisible = visible ? "true" : "false";
      button.dataset.kgwDisplayOwner = "R75";
    }

    try {
      kgwMainTabTraceR35C("settings", "r75-apply-tabs-dom", {
        tabs: normalizedTabs.join(","),
        buttons: String(tabButtons.length),
        shown: String(shown),
        hidden: String(hidden)
      });
    } catch (_) { /* Best-effort secondary operation; primary shell behavior is preserved. */ }

    kgwShellDisplayOwnerEnsureActiveTabR59C("apply-tabs-r75");
  }

  function apply(input, reason = "apply") {
    const prefs = save(input || read());

    rebuildSelect("shellLanguageSelect", languageOptions, prefs.languages);
    rebuildSelect("shellCurrencySelect", currencyOptions, prefs.currencies);
    applyTabs(prefs.tabs);

    document.documentElement.dataset.kgwDisplayPreferencesR71 = reason;
    kgwShellDisplayOwnerPublishR59C();
    kgwShellDisplayOwnerEnsureActiveTabR59C(reason);

    return prefs;
  }

  window.kgwShellDisplayPreferencesR71 = {
    storageKey,
    languageOptions,
    currencyOptions,
    tabOptions,
    defaults,
    normalize,
    read,
    save,
    apply
  };

  window.addEventListener("kgw:shell-display-preferences-changed", (event) => {
    apply(event.detail || read(), "event");
  });

  function boot() {
    apply(read(), "boot");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    window.setTimeout(boot, 0);
  }
})();

/* KGW_R73_SHELL_I18N_LANGUAGE_SWITCH_OWNER */
(function installKgwShellI18nLanguageSwitchOwnerR73() {
  if (window.kgwShellI18nLanguageSwitchOwnerR73) return;
  window.kgwShellI18nLanguageSwitchOwnerR73 = true;

  const storageKey = "kgw.shell.language.v73";
  const supportedLanguages = ["en","ar","de","es","fr","hi","id","ja","ko","ru","tr","zh-CN"];
  const cache = new Map();

  /* KGW_R107_CANONICAL_TRANSLATION_RUNTIME_API */
  window.__kgwI18nDictR107 = window.__kgwI18nDictR107 || {};
  window.__kgwI18nLangR107 = window.__kgwI18nLangR107 || "en";

  window.kgwT = function kgwTranslateRuntimeR107(key, fallback = "") {
    const lookupKey = String(key || "");
    const dict = window.__kgwI18nDictR107 || {};
    const value = dict[lookupKey];

    if (typeof value === "string" && value.length > 0) {
      return value;
    }

    if (typeof fallback === "string" && fallback.length > 0) {
      return fallback;
    }

    return lookupKey;
  };

  window.kgwI18n = window.kgwT;

  function normalizeLanguage(lang) {
    return supportedLanguages.includes(lang) ? lang : "en";
  }

  function flatten(obj, prefix = "", out = {}) {
    if (!obj || typeof obj !== "object" || Array.isArray(obj)) return out;

    for (const [key, value] of Object.entries(obj)) {
      const nextKey = prefix ? prefix + "." + key : key;

      if (value && typeof value === "object" && !Array.isArray(value)) {
        flatten(value, nextKey, out);
      } else if (typeof value === "string") {
        out[nextKey] = value;
      }
    }

    return out;
  }

  async function loadLanguage(lang) {
    const normalized = normalizeLanguage(lang);

    if (cache.has(normalized)) return cache.get(normalized);

    try {
      const response = await fetch("./i18n/" + normalized + ".json", { cache: "no-store" });
      if (!response.ok) throw new Error("HTTP " + response.status);

      const data = flatten(await response.json());
      cache.set(normalized, data);
      return data;
    } catch (error) {
      console.warn("[KGW i18n] Failed to load language", normalized, error);
      if (normalized !== "en") return loadLanguage("en");
      return {};
    }
  }


  /* KGW_R99_CANONICAL_I18N_BIND_APPLY_HELPER */
  function normalizeKgwI18nTextR99(value) {
    return String(value || "").replace(/\s+/g, " ").trim();
  }

  function shouldSkipKgwI18nElementR99(element) {
    if (!element || !element.matches) return true;
    if (element.closest("script,style,svg,canvas,[data-kgw-no-i18n='true']")) return true;
    if (element.id === "kgwHeaderPrice" || element.id === "kgwHeaderHashrate" || element.id === "kgwHeaderDifficulty") return true;
    return false;
  }

  function buildKgwI18nReverseIndexR99(...dicts) {
    const index = new Map();

    for (const dict of dicts) {
      if (!dict || typeof dict !== "object") continue;

      for (const [key, value] of Object.entries(dict)) {
        const normalized = normalizeKgwI18nTextR99(value);
        if (!normalized || normalized.length > 160) continue;
        if (!/[A-Za-z\u0600-\u06FF]/.test(normalized)) continue;
        if (!index.has(normalized)) index.set(normalized, key);
      }
    }

    return index;
  }

  function bindMissingI18nAttributesR99(root = document, fallbackDict = {}, selectedDict = {}) {
    const scope = root && root.querySelectorAll ? root : document;
    const reverse = buildKgwI18nReverseIndexR99(fallbackDict, selectedDict);
    const selector = [
      "button", "a", "label", "span", "strong", "legend", "th", "td", "option",
      "h1", "h2", "h3", "h4", "p", "small", "div"
    ].join(",");

    for (const element of Array.from(scope.querySelectorAll(selector))) {
      if (shouldSkipKgwI18nElementR99(element) || !element.dataset) continue;

      if (!element.dataset.i18n) {
        const normalized = normalizeKgwI18nTextR99(element.textContent);
        const key = reverse.get(normalized);
        if (key) element.dataset.i18n = key;
      }

      if (element.hasAttribute("title") && !element.dataset.i18nTitle) {
        const key = reverse.get(normalizeKgwI18nTextR99(element.getAttribute("title")));
        if (key) element.dataset.i18nTitle = key;
      }

      if (element.hasAttribute("placeholder") && !element.dataset.i18nPlaceholder) {
        const key = reverse.get(normalizeKgwI18nTextR99(element.getAttribute("placeholder")));
        if (key) element.dataset.i18nPlaceholder = key;
      }

      if (element.hasAttribute("aria-label") && !element.dataset.i18nAriaLabel) {
        const key = reverse.get(normalizeKgwI18nTextR99(element.getAttribute("aria-label")));
        if (key) element.dataset.i18nAriaLabel = key;
      }
    }

    document.documentElement.dataset.kgwI18nBoundR99 = "ready";
  }

  function setKgwI18nTextSafelyR87(element, value) {
    if (!element || typeof value !== "string") return;

    const tag = String(element.tagName || "").toLowerCase();
    if (tag === "select" || tag === "input" || tag === "textarea") return;

    const childNodes = Array.from(element.childNodes || []);
    const hasElementChildren = childNodes.some((node) => node.nodeType === Node.ELEMENT_NODE);

    if (!hasElementChildren) {
      if (element.textContent !== value) element.textContent = value;
      return;
    }

    const textNode = childNodes.find((node) => {
      return node.nodeType === Node.TEXT_NODE && normalizeKgwI18nTextR99(node.nodeValue).length > 0;
    });

    if (textNode) {
      const prefix = /^\s/.test(textNode.nodeValue || "") ? " " : "";
      const suffix = /\s$/.test(textNode.nodeValue || "") ? " " : "";
      textNode.nodeValue = prefix + value + suffix;
    }
  }
  /* KGW_R100_FLATTEN_I18N_DICTIONARY */
  function flattenKgwI18nDictionaryR100(source, prefix = "", out = {}) {
    if (!source || typeof source !== "object" || Array.isArray(source)) {
      if (prefix) out[prefix] = source;
      return out;
    }

    for (const [key, value] of Object.entries(source)) {
      if (key.includes(".")) {
        if (value && typeof value === "object" && !Array.isArray(value)) {
          flattenKgwI18nDictionaryR100(value, key, out);
        } else {
          out[key] = value;
        }
        continue;
      }

      const next = prefix ? prefix + "." + key : key;

      if (value && typeof value === "object" && !Array.isArray(value)) {
        flattenKgwI18nDictionaryR100(value, next, out);
      } else {
        out[next] = value;
      }
    }

    return out;
  }


  /* KGW_R102_DYNAMIC_DOM_I18N_REAPPLY */
  const kgwI18nRuntimeStateR102 = {
    observer: null,
    pending: false,
    applying: false,
    dict: null,
    fallbackDict: null,
    selectedDict: null,
    lang: null,
    reverseIndex: null,
  };

  function buildKgwI18nRuntimeReverseIndexR102(fallbackDict = {}, selectedDict = {}) {
    const index = new Map();

    for (const dict of [fallbackDict, selectedDict]) {
      if (!dict || typeof dict !== "object") continue;

      for (const [key, value] of Object.entries(dict)) {
        if (typeof value !== "string") continue;

        const normalized = normalizeKgwI18nTextR99(value);
        if (!normalized || normalized.length > 180) continue;
        if (!/[A-Za-z\u0600-\u06FF]/.test(normalized)) continue;

        if (!index.has(normalized)) index.set(normalized, key);
      }
    }

    return index;
  }

  function markDynamicKgwI18nAttributesR102(root = document) {
    const state = kgwI18nRuntimeStateR102;
    const reverse = state.reverseIndex;

    if (!reverse || !reverse.size) return;

    const scope = root && root.querySelectorAll ? root : document;
    const nodes = [];

    if (scope.nodeType === 1) nodes.push(scope);

    if (scope.querySelectorAll) {
      nodes.push(...Array.from(scope.querySelectorAll([
        "button", "a", "label", "span", "strong", "legend", "th", "td", "option",
        "h1", "h2", "h3", "h4", "p", "small", "div"
      ].join(","))));
    }

    for (const element of nodes) {
      if (!element || !element.dataset) continue;
      if (shouldSkipKgwI18nElementR99(element)) continue;

      if (!element.dataset.i18n) {
        const normalized = normalizeKgwI18nTextR99(element.textContent);
        const key = reverse.get(normalized);

        if (key) element.dataset.i18n = key;
      }

      if (element.hasAttribute("title") && !element.dataset.i18nTitle) {
        const key = reverse.get(normalizeKgwI18nTextR99(element.getAttribute("title")));

        if (key) element.dataset.i18nTitle = key;
      }

      if (element.hasAttribute("placeholder") && !element.dataset.i18nPlaceholder) {
        const key = reverse.get(normalizeKgwI18nTextR99(element.getAttribute("placeholder")));

        if (key) element.dataset.i18nPlaceholder = key;
      }

      if (element.hasAttribute("aria-label") && !element.dataset.i18nAriaLabel) {
        const key = reverse.get(normalizeKgwI18nTextR99(element.getAttribute("aria-label")));

        if (key) element.dataset.i18nAriaLabel = key;
      }
    }
  }

  function scheduleDynamicKgwI18nReapplyR102(reason = "dom-mutation") {
    const state = kgwI18nRuntimeStateR102;

    if (state.pending || state.applying || !state.dict || !state.lang) return;

    state.pending = true;

    window.setTimeout(() => {
      state.pending = false;

      if (!state.dict || !state.lang) return;

      state.applying = true;

      try {
        markDynamicKgwI18nAttributesR102(document);
        applyDictionary(state.dict, state.lang);
        document.documentElement.dataset.kgwLastDynamicI18nReapplyR102 = reason;
      } finally {
        window.setTimeout(() => {
          state.applying = false;
        }, 0);
      }
    }, 32);
  }

  function installDynamicKgwI18nObserverR102() {
    const state = kgwI18nRuntimeStateR102;

    if (state.observer || typeof MutationObserver === "undefined") return;

    state.observer = new MutationObserver((mutations) => {
      if (state.applying) return;

      for (const mutation of mutations) {
        if (mutation.type === "childList" && mutation.addedNodes && mutation.addedNodes.length > 0) {
          scheduleDynamicKgwI18nReapplyR102("child-list");
          return;
        }

        if (mutation.type === "attributes") {
          const name = mutation.attributeName || "";

          if (name === "title" || name === "placeholder" || name === "aria-label") {
            scheduleDynamicKgwI18nReapplyR102("attribute-change");
            return;
          }
        }

        if (mutation.type === "characterData") {
          scheduleDynamicKgwI18nReapplyR102("text-change");
          return;
        }
      }
    });

    state.observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["title", "placeholder", "aria-label"],
      characterData: true,
    });

    document.documentElement.dataset.kgwDynamicI18nObserverR102 = "installed";
  }

  function updateDynamicKgwI18nRuntimeR102(dict, lang, fallbackDict = {}, selectedDict = {}) {
    const state = kgwI18nRuntimeStateR102;

    state.dict = dict || {};
    state.lang = lang || "en";
    state.fallbackDict = fallbackDict || {};
    state.selectedDict = selectedDict || {};
    state.reverseIndex = buildKgwI18nRuntimeReverseIndexR102(state.fallbackDict, state.selectedDict);

    installDynamicKgwI18nObserverR102();
    markDynamicKgwI18nAttributesR102(document);
  }


  function applyDictionary(dict, lang) {
    document.documentElement.lang = lang;
    document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";

    for (const element of document.querySelectorAll("[data-i18n]")) {
      const key = element.dataset.i18n;
      const value = dict[key];

      if (typeof value === "string" && element.textContent !== value) {
        setKgwI18nTextSafelyR87(element, value);
      }
    }

    for (const element of document.querySelectorAll("[data-i18n-title]")) {
      const key = element.dataset.i18nTitle;
      const value = dict[key];

      if (typeof value === "string") {
        element.setAttribute("title", value);
      }
    }

    for (const element of document.querySelectorAll("[data-i18n-placeholder]")) {
      const key = element.dataset.i18nPlaceholder;
      const value = dict[key];

      if (typeof value === "string") {
        element.setAttribute("placeholder", value);
      }
    }

    for (const element of document.querySelectorAll("[data-i18n-aria-label]")) {
      const key = element.dataset.i18nAriaLabel;
      const value = dict[key];

      if (typeof value === "string") {
        element.setAttribute("aria-label", value);
      }
    }

    document.documentElement.dataset.kgwLanguageApplied = lang;
  }

  async function setLanguage(lang, reason = "manual") {
    const normalized = normalizeLanguage(lang);
    const selectedDictRaw = await loadLanguage(normalized);
    const fallbackDictRaw = normalized === "en" ? selectedDictRaw : await loadLanguage("en");
    const selectedDict = flattenKgwI18nDictionaryR100(selectedDictRaw);
    const fallbackDict = flattenKgwI18nDictionaryR100(fallbackDictRaw);
    bindMissingI18nAttributesR99(document, fallbackDict, selectedDict);
    const dict = Object.assign({}, fallbackDict, selectedDict);
    window.__kgwI18nDictR107 = dict;
    window.__kgwI18nLangR107 = normalized;
    updateDynamicKgwI18nRuntimeR102(dict, normalized, fallbackDict, selectedDict);

    localStorage.setItem(storageKey, normalized);
    applyDictionary(dict, normalized);

    const select = document.getElementById("shellLanguageSelect");
    if (select && select.value !== normalized) {
      select.value = normalized;
    }

    window.dispatchEvent(new CustomEvent("kgw:language-applied", {
      detail: { language: normalized, reason }
    }));

    return normalized;
  }

  function currentLanguage() {
    const select = document.getElementById("shellLanguageSelect");
    return normalizeLanguage(select?.value || localStorage.getItem(storageKey) || "en");
  }

  function installSelectListener() {
    const select = document.getElementById("shellLanguageSelect");
    if (!select || select.dataset.kgwI18nR73 === "1") return;

    select.dataset.kgwI18nR73 = "1";
    select.addEventListener("change", () => {
      setLanguage(select.value, "dropdown");
    });
  }

  async function boot() {
    installSelectListener();
    await setLanguage(currentLanguage(), "boot");
  }

  window.kgwSetLanguageR73 = setLanguage;
  window.kgwApplyLanguageR73 = () => setLanguage(currentLanguage(), "reapply");

  window.kgwReapplyLanguageSilentlyR89 = async function kgwReapplyLanguageSilentlyR89(reason = "silent-reapply") {
    const normalized = currentLanguage();
    const selectedDictRaw = await loadLanguage(normalized);
    const fallbackDictRaw = normalized === "en" ? selectedDictRaw : await loadLanguage("en");
    const selectedDict = flattenKgwI18nDictionaryR100(selectedDictRaw);
    const fallbackDict = flattenKgwI18nDictionaryR100(fallbackDictRaw);
    const dict = Object.assign({}, fallbackDict, selectedDict);

    window.__kgwI18nDictR107 = dict;
    window.__kgwI18nLangR107 = normalized;
    bindMissingI18nAttributesR99(document, fallbackDict, selectedDict);
    updateDynamicKgwI18nRuntimeR102(dict, normalized, fallbackDict, selectedDict);
    applyDictionary(dict, normalized);

    document.documentElement.dataset.kgwLanguageSilentReapplyReason = String(reason || "silent-reapply");
    return normalized;
  };

  window.addEventListener("kgw:shell-display-preferences-changed", () => {
    window.setTimeout(() => {
      installSelectListener();
      setLanguage(currentLanguage(), "display-preferences");
    }, 0);
  });

  window.addEventListener("kgw:tab-opened", () => {
    window.setTimeout(() => setLanguage(currentLanguage(), "tab-opened"), 0);
  });

  document.addEventListener("click", (event) => {
    if (event.target.closest("[data-tab]")) {
      window.setTimeout(() => setLanguage(currentLanguage(), "tab-click"), 50);
    }
  }, true);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    window.setTimeout(boot, 0);
  }
})();

/* KGW_R85B_ACTIONS_FINAL_REPAIR_BINDING */
(function installKgwActionsFinalRepairBindingR85B() {
  if (window.kgwActionsFinalRepairBindingR85B) return;
  window.kgwActionsFinalRepairBindingR85B = true;

  const bindings = [
  {
    "text": "Add",
    "key": "actions.add"
  },
  {
    "text": "Backup",
    "key": "actions.backup"
  },
  {
    "text": "Cancel",
    "key": "actions.cancel"
  },
  {
    "text": "Clear Caches",
    "key": "actions.clear.caches"
  },
  {
    "text": "Clear caches",
    "key": "actions.clear.caches"
  },
  {
    "text": "Copy",
    "key": "actions.copy"
  },
  {
    "text": "Delete",
    "key": "actions.delete"
  },
  {
    "text": "Fetch",
    "key": "actions.fetch"
  },
  {
    "text": "Filter",
    "key": "actions.filter"
  },
  {
    "text": "Refresh",
    "key": "actions.refresh"
  },
  {
    "text": "Reset",
    "key": "actions.reset"
  },
  {
    "text": "Restore",
    "key": "actions.restore"
  },
  {
    "text": "Save As",
    "key": "actions.save.as"
  },
  {
    "text": "Save as",
    "key": "actions.save.as"
  },
  {
    "text": "Search",
    "key": "actions.search"
  },
  {
    "text": "Update",
    "key": "actions.update"
  }
];

  function normalized(value) {
    return String(value || "").replace(/\s+/g, " ").trim();
  }

  function applyBindings(root = document) {
    const candidates = Array.from(root.querySelectorAll("button, a, label, span, option"));

    for (const item of bindings) {
      for (const element of candidates) {
        if (!element.dataset) continue;

        if (!element.dataset.i18n && normalized(element.textContent) === item.text) {
          element.dataset.i18n = item.key;
        }

        if (element.hasAttribute("title") && !element.dataset.i18nTitle && normalized(element.getAttribute("title")) === item.text) {
          element.dataset.i18nTitle = item.key;
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
    if (event.target.closest("[data-tab]")) {
      window.setTimeout(() => applyBindings(document), 80);
      window.setTimeout(() => applyBindings(document), 350);
      window.setTimeout(() => applyBindings(document), 900);
    }
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(() => applyBindings(document), 0);
  });

  window.kgwApplyActionsI18nR85B = applyBindings;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => applyBindings(document), { once: true });
  } else {
    window.setTimeout(() => applyBindings(document), 0);
  }
})();

/* KGW_R87_I18N_SAFE_APPLY_SELECT_FREEZE_FIX */
(function installKgwI18nSafeApplySelectFreezeFixR87() {
  if (window.kgwI18nSafeApplySelectFreezeFixR87) return;
  window.kgwI18nSafeApplySelectFreezeFixR87 = true;

  const protectedSelectors = [
    "#shellLanguageSelect",
    "#shellCurrencySelect",
    "select",
    "input",
    "textarea"
  ];

  function protectControls() {
    const controls = Array.from(document.querySelectorAll(protectedSelectors.join(",")));

    for (const control of controls) {
      if (!(control instanceof Element)) continue;
      // Runtime forms own their dependencies and ownership locks.
      if (control.closest("#kaspa-node,#kaspa-bridge")) continue;

      control.disabled = false;
      control.style.pointerEvents = "auto";
      control.style.position = control.style.position || "relative";
      control.style.zIndex = "2147483647";

      if (control.id === "shellLanguageSelect" || control.id === "shellCurrencySelect") {
        control.style.direction = "ltr";
        control.style.unicodeBidi = "isolate";
        control.style.textAlign = "left";
      }

      let parent = control.parentElement;
      let depth = 0;

      while (parent && depth < 8) {
        if (parent.dataset) {
          if (parent.querySelector("select,input,textarea")) {
            parent.removeAttribute("data-i18n");
            parent.removeAttribute("data-i18n-title");
            parent.removeAttribute("data-i18n-placeholder");
          }
        }

        parent.style.pointerEvents = "auto";
        parent = parent.parentElement;
        depth += 1;
      }
    }

    document.documentElement.dataset.kgwR87SelectFreezeFix = "ready";
  }

  document.addEventListener("change", (event) => {
    const target = event.target;

    if (!(target instanceof HTMLSelectElement)) return;

    protectControls();

    /*
     * R88: do not call kgwSetLanguageR73 here.
     * R73 owns shellLanguageSelect change handling.
     * Calling it here created duplicate language-apply cycles after RTL changes.
     */
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(protectControls, 0);
    window.setTimeout(protectControls, 120);
    window.setTimeout(protectControls, 500);
  });

  window.addEventListener("kgw:shell-display-preferences-changed", () => {
    window.setTimeout(protectControls, 0);
    window.setTimeout(protectControls, 120);
  });

  document.addEventListener("click", () => {
    window.setTimeout(protectControls, 0);
  }, true);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", protectControls, { once: true });
  } else {
    window.setTimeout(protectControls, 0);
  }

  window.kgwProtectControlsR87 = protectControls;
})();

/* KGW_R88_I18N_ROOT_CAUSE_SELECT_FREEZE_GUARD */
(function installKgwI18nRootCauseSelectFreezeGuardR88() {
  if (window.kgwI18nRootCauseSelectFreezeGuardR88) return;
  window.kgwI18nRootCauseSelectFreezeGuardR88 = true;

  function protectInteractiveControls() {
    const controls = Array.from(document.querySelectorAll("select,input,textarea,button"));

    for (const control of controls) {
      if (!(control instanceof Element)) continue;
      // Runtime forms own their dependencies and ownership locks.
      if (control.closest("#kaspa-node,#kaspa-bridge")) continue;

      control.style.pointerEvents = "auto";

      if (control.id === "shellLanguageSelect" || control.id === "shellCurrencySelect") {
        control.disabled = false;
        control.removeAttribute("disabled");
        control.style.position = "relative";
        control.style.zIndex = "2147483647";
        control.style.direction = "ltr";
        control.style.unicodeBidi = "isolate";
        control.style.textAlign = "left";
      }

      let parent = control.parentElement;
      let depth = 0;

      while (parent && depth < 8) {
        if (parent.dataset && parent.querySelector("select,input,textarea")) {
          parent.removeAttribute("data-i18n");
          parent.removeAttribute("data-i18n-title");
          parent.removeAttribute("data-i18n-placeholder");
        }

        parent.style.pointerEvents = "auto";
        parent = parent.parentElement;
        depth += 1;
      }
    }

    document.documentElement.dataset.kgwR88InteractiveGuard = "ready";
  }

  document.addEventListener("pointerdown", (event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;

    if (target.closest("#shellLanguageSelect,#shellCurrencySelect,select,input,textarea,button")) {
      protectInteractiveControls();
    }
  }, true);

  document.addEventListener("change", () => {
    window.setTimeout(protectInteractiveControls, 0);
    window.setTimeout(protectInteractiveControls, 120);
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(protectInteractiveControls, 0);
    window.setTimeout(protectInteractiveControls, 120);
    window.setTimeout(protectInteractiveControls, 500);
  });

  window.addEventListener("kgw:shell-display-preferences-changed", () => {
    window.setTimeout(protectInteractiveControls, 0);
    window.setTimeout(protectInteractiveControls, 120);
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", protectInteractiveControls, { once: true });
  } else {
    window.setTimeout(protectInteractiveControls, 0);
  }

  window.kgwProtectInteractiveControlsR88 = protectInteractiveControls;
})();

/* KGW_R89_I18N_EVENT_LOOP_SELECT_ROOT_FIX */
(function installKgwI18nEventLoopSelectRootFixR89() {
  if (window.kgwI18nEventLoopSelectRootFixR89) return;
  window.kgwI18nEventLoopSelectRootFixR89 = true;

  let lastLanguageChangeAt = 0;

  function protectShellSelects() {
    const ids = ["shellLanguageSelect", "shellCurrencySelect"];

    for (const id of ids) {
      const select = document.getElementById(id);
      if (!select) continue;

      select.disabled = false;
      select.removeAttribute("disabled");
      select.removeAttribute("aria-disabled");
      select.style.pointerEvents = "auto";
      select.style.position = "relative";
      select.style.zIndex = "2147483647";
      select.style.direction = "ltr";
      select.style.unicodeBidi = "isolate";
      select.style.textAlign = "left";

      let parent = select.parentElement;
      let depth = 0;

      while (parent && depth < 8) {
        parent.style.pointerEvents = "auto";
        if (parent.dataset && parent.querySelector("select,input,textarea")) {
          parent.removeAttribute("data-i18n");
          parent.removeAttribute("data-i18n-title");
          parent.removeAttribute("data-i18n-placeholder");
        }
        parent = parent.parentElement;
        depth += 1;
      }
    }

    document.documentElement.dataset.kgwR89ShellSelectGuard = "ready";
  }

  document.addEventListener("pointerdown", (event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;

    if (target.closest("#shellLanguageSelect,#shellCurrencySelect")) {
      protectShellSelects();
    }
  }, true);

  document.addEventListener("change", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLSelectElement)) return;

    if (target.id === "shellLanguageSelect") {
      lastLanguageChangeAt = Date.now();
      protectShellSelects();
      return;
    }

    if (target.id === "shellCurrencySelect") {
      protectShellSelects();
    }
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    const elapsed = Date.now() - lastLanguageChangeAt;
    if (elapsed < 2000) {
      window.setTimeout(protectShellSelects, 0);
      window.setTimeout(protectShellSelects, 120);
    } else {
      window.setTimeout(protectShellSelects, 0);
    }
  });

  window.addEventListener("kgw:shell-display-preferences-changed", () => {
    window.setTimeout(protectShellSelects, 0);
    window.setTimeout(protectShellSelects, 120);
  });

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", protectShellSelects, { once: true });
  } else {
    window.setTimeout(protectShellSelects, 0);
  }

  window.kgwProtectShellSelectsR89 = protectShellSelects;
})();

/* KGW_R90_VALIDATION_I18N_BINDING */
(function installKgwValidationI18nBindingR90() {
  if (window.kgwValidationI18nBindingR90) return;
  window.kgwValidationI18nBindingR90 = true;

  const bindings = [
  {
    "text": "Missing advanced libraries (networkx, scikit-learn).",
    "key": "validation.missing.advanced.libraries.networkx.scikit.learn"
  },
  {
    "text": "Please select at least one currency.",
    "key": "validation.please.select.at.least.one.currency"
  },
  {
    "text": "Please select at least one language.",
    "key": "validation.please.select.at.least.one.language"
  },
  {
    "text": "User aborted update due to missing hash file.",
    "key": "validation.user.aborted.update.due.to.missing.hash.file"
  }
];

  function normalized(value) {
    return String(value || "").replace(/\s+/g, " ").trim();
  }

  function applyBindings(root = document) {
    const candidates = Array.from(root.querySelectorAll("label, span, p, small, option, button"));

    for (const item of bindings) {
      for (const element of candidates) {
        if (!element.dataset) continue;

        if (!element.dataset.i18n && normalized(element.textContent) === item.text) {
          element.dataset.i18n = item.key;
        }

        if (element.hasAttribute("title") && !element.dataset.i18nTitle && normalized(element.getAttribute("title")) === item.text) {
          element.dataset.i18nTitle = item.key;
        }

        if (element.hasAttribute("placeholder") && !element.dataset.i18nPlaceholder && normalized(element.getAttribute("placeholder")) === item.text) {
          element.dataset.i18nPlaceholder = item.key;
        }
      }
    }

    if (typeof window.kgwReapplyLanguageSilentlyR89 === "function") {
      window.setTimeout(() => window.kgwReapplyLanguageSilentlyR89("r90-validation-binding"), 0);
    }
  }

  document.addEventListener("click", (event) => {
    if (event.target.closest("[data-tab]")) {
      window.setTimeout(() => applyBindings(document), 80);
      window.setTimeout(() => applyBindings(document), 350);
    }
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(() => applyBindings(document), 0);
  });

  window.kgwApplyValidationI18nR90 = applyBindings;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => applyBindings(document), { once: true });
  } else {
    window.setTimeout(() => applyBindings(document), 0);
  }
})();

/* KGW_R92_BRIDGE_I18N_BINDING */
(function installKgwBridgeI18nBindingR92() {
  if (window.kgwBridgeI18nBindingR92) return;
  window.kgwBridgeI18nBindingR92 = true;

  const bindings = [
  {
    "text": "Bridge 1",
    "key": "bridge.bridge.1"
  },
  {
    "text": "Bridge 2",
    "key": "bridge.bridge.2"
  },
  {
    "text": "Bridge files are already up to date.",
    "key": "bridge.bridge.files.are.already.up.to.date"
  },
  {
    "text": "Bridge is already running.",
    "key": "bridge.bridge.is.already.running"
  },
  {
    "text": "Bridge is not running.",
    "key": "bridge.bridge.is.not.running"
  },
  {
    "text": "Bridge update complete.",
    "key": "bridge.bridge.update.complete"
  },
  {
    "text": "Enable Bridge 2",
    "key": "bridge.enable.bridge.2"
  },
  {
    "text": "Kaspa Bridge",
    "key": "bridge.kaspa.bridge"
  },
  {
    "text": "Kaspa Bridge",
    "key": "bridge.tabs.kaspabridge"
  },
  {
    "text": "Please stop Bridge 2 before disabling it.",
    "key": "bridge.please.stop.bridge.2.before.disabling.it"
  },
  {
    "text": "tabs.kaspaBridge",
    "key": "bridge.tabs.kaspabridge"
  }
];

  function normalized(value) {
    return String(value || "").replace(/\s+/g, " ").trim();
  }

  function applyBindings(root = document) {
    const bridgeRoot =
      document.querySelector("[data-tab-panel='kaspa-bridge']") ||
      document.querySelector("[data-tab='kaspa-bridge']") ||
      document;

    const scope = root === document ? bridgeRoot : root;
    const candidates = Array.from(scope.querySelectorAll("label, span, p, small, option, button, h1, h2, h3, h4"));

    for (const item of bindings) {
      for (const element of candidates) {
        if (!element.dataset) continue;

        if (!element.dataset.i18n && normalized(element.textContent) === item.text) {
          element.dataset.i18n = item.key;
        }

        if (element.hasAttribute("title") && !element.dataset.i18nTitle && normalized(element.getAttribute("title")) === item.text) {
          element.dataset.i18nTitle = item.key;
        }

        if (element.hasAttribute("placeholder") && !element.dataset.i18nPlaceholder && normalized(element.getAttribute("placeholder")) === item.text) {
          element.dataset.i18nPlaceholder = item.key;
        }
      }
    }

    if (typeof window.kgwReapplyLanguageSilentlyR89 === "function") {
      window.setTimeout(() => window.kgwReapplyLanguageSilentlyR89("r92-bridge-binding"), 0);
    }
  }

  document.addEventListener("click", (event) => {
    if (event.target.closest("[data-tab='kaspa-bridge'], [data-tab='bridge'], [data-tab-panel='kaspa-bridge']")) {
      window.setTimeout(() => applyBindings(document), 80);
      window.setTimeout(() => applyBindings(document), 350);
    }
  }, true);

  window.addEventListener("kgw:language-applied", () => {
    window.setTimeout(() => applyBindings(document), 0);
  });

  window.kgwApplyBridgeI18nR92 = applyBindings;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => applyBindings(document), { once: true });
  } else {
    window.setTimeout(() => applyBindings(document), 0);
  }
})();

/* KGW_R98_FINAL_I18N_OWNER_SENTINEL */
(function installKgwFinalI18nOwnerSentinelR98() {
  if (window.kgwFinalI18nOwnerSentinelR98) return;
  window.kgwFinalI18nOwnerSentinelR98 = true;

  const state = {
    lastLanguageAppliedAt: 0,
    recentLanguageApplyCount: 0,
    lastWarningAt: 0
  };

  function warnOnce(message, details = {}) {
    const now = Date.now();
    if (now - state.lastWarningAt < 3000) return;
    state.lastWarningAt = now;
    console.warn("[KGW i18n][R98 owner sentinel]", message, details);
  }

  function protectShellSelects() {
    for (const id of ["shellLanguageSelect", "shellCurrencySelect"]) {
      const select = document.getElementById(id);
      if (!select) continue;

      select.disabled = false;
      select.removeAttribute("disabled");
      select.removeAttribute("aria-disabled");
      select.style.pointerEvents = "auto";
      select.style.position = "relative";
      select.style.zIndex = "2147483647";
      select.style.direction = "ltr";
      select.style.unicodeBidi = "isolate";
      select.style.textAlign = "left";
    }
  }

  window.addEventListener("kgw:language-applied", () => {
    const now = Date.now();

    if (now - state.lastLanguageAppliedAt < 600) {
      state.recentLanguageApplyCount += 1;
    } else {
      state.recentLanguageApplyCount = 1;
    }

    state.lastLanguageAppliedAt = now;

    if (state.recentLanguageApplyCount > 4) {
      warnOnce("possible repeated language-apply cycle detected", {
        recentLanguageApplyCount: state.recentLanguageApplyCount
      });
    }

    window.setTimeout(protectShellSelects, 0);
    window.setTimeout(protectShellSelects, 120);
  });

  document.addEventListener("pointerdown", (event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;

    if (target.closest("#shellLanguageSelect,#shellCurrencySelect,select,input,textarea,button")) {
      protectShellSelects();
    }
  }, true);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", protectShellSelects, { once: true });
  } else {
    window.setTimeout(protectShellSelects, 0);
  }

  window.kgwFinalI18nOwnerSentinelR98 = {
    protectShellSelects,
    getState: () => ({ ...state })
  };
})();
