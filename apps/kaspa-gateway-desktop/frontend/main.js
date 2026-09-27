import { KGW_TABS } from "./src/tabs/tab-registry.js";
import initShellWasm, {
  shellAuxInstall as wasmShellAuxInstall,
  shellDisplayInstall as wasmShellDisplayInstall,
  shellRuntimeActivateTab as wasmShellActivateTab,
  shellRuntimeApplyTheme as wasmShellApplyTheme,
  shellRuntimeInstall as wasmShellInstall,
  shellRuntimeOpenTab as wasmShellOpenTab,
} from "./generated/kgw_frontend_wasm/kgw_frontend_wasm.js";

await initShellWasm();
wasmShellInstall(KGW_TABS);
wasmShellAuxInstall();
wasmShellDisplayInstall();

async function openTab(tabId, options = {}) {
  return await wasmShellOpenTab(String(tabId || ""), options || {});
}

function activateTab(tabId) {
  return wasmShellActivateTab(String(tabId || ""));
}

function applyTheme(theme) {
  return wasmShellApplyTheme(String(theme || ""));
}

window.kgwOpenTab = openTab;
window.kgwActivateTab = activateTab;
window.kgwApplyTheme = applyTheme;

import("./src/core/header-live-metrics.js");


/* Shell display preferences / active-tab visibility are Rust-owned in shell_display.rs. */

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
