import { KGW_TABS } from "./src/tabs/tab-registry.js";
import initShellWasm, {
  shellAuxInstall as wasmShellAuxInstall,
  shellDisplayInstall as wasmShellDisplayInstall,
  shellI18nInstall as wasmShellI18nInstall,
  shellRuntimeActivateTab as wasmShellActivateTab,
  shellRuntimeApplyTheme as wasmShellApplyTheme,
  shellRuntimeInstall as wasmShellInstall,
  shellRuntimeOpenTab as wasmShellOpenTab,
} from "./generated/kgw_frontend_wasm/kgw_frontend_wasm.js";

await initShellWasm();
wasmShellInstall(KGW_TABS);
wasmShellAuxInstall();
wasmShellDisplayInstall();
wasmShellI18nInstall();

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

/* Core shell i18n runtime is Rust-owned in shell_i18n.rs. */

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
