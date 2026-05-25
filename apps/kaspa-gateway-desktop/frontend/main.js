
/* KGW_SHELL_EXPLORER_FETCH_BUSY_TAB_POLICY
   Shell owns top-tab navigation.
   During Explorer transaction fetch:
   - Explorer tab allowed.
   - Log tab allowed.
   - Other top tabs disabled/blocked.
*/
(function installKgwShellExplorerFetchBusyTabPolicy() {
  if (window.__kgwShellExplorerFetchBusyTabPolicyInstalled) return;
  window.__kgwShellExplorerFetchBusyTabPolicyInstalled = true;

  function textOf(element) {
    return String(element?.textContent || "").trim().toLowerCase();
  }

  function keyOf(element) {
    if (!element) return "";

    return [
      element.id || "",
      element.className || "",
      element.getAttribute?.("data-tab") || "",
      element.getAttribute?.("data-target") || "",
      element.getAttribute?.("aria-controls") || "",
      element.getAttribute?.("href") || "",
      textOf(element)
    ].join(" ").toLowerCase();
  }

  function tabNameOf(element) {
    const key = keyOf(element);

    if (key.includes("explorer") || key.includes("المستكشف")) return "explorer";
    if (key.includes(" log") || key === "log" || key.includes("#log") || key.includes("لوج") || key.includes("السجل")) return "log";
    if (key.includes("kaspa node") || key.includes("kaspa-node")) return "kaspa-node";
    if (key.includes("kaspa bridge") || key.includes("kaspa-bridge")) return "kaspa-bridge";
    if (key.includes("analysis")) return "analysis";
    if (key.includes("top addresses") || key.includes("top-addresses")) return "top-addresses";
    if (key.includes("settings")) return "settings";

    return "";
  }

  function isTopTabControl(element) {
    if (!element) return false;

    const key = keyOf(element);
    const name = tabNameOf(element);

    if (!name) return false;

    return (
      element.matches?.("button,a,[role='tab'],[data-tab],[data-target],[aria-controls]") ||
      key.includes("tab") ||
      key.includes("nav")
    );
  }

  function setTabControlEnabled(element, enabled) {
    if (!element) return;

    if ("disabled" in element) {
      element.disabled = !enabled;
    }

    if (enabled) {
      element.removeAttribute("disabled");
      element.removeAttribute("aria-disabled");
      element.classList?.remove("disabled");
      element.classList?.remove("is-disabled");
      element.style.pointerEvents = "auto";
      element.style.cursor = "pointer";
    } else {
      element.setAttribute("aria-disabled", "true");
      element.classList?.add("disabled");
      element.style.pointerEvents = "none";
      element.style.cursor = "not-allowed";
    }
  }

  window.kgwApplyShellExplorerFetchBusyPolicy = function kgwApplyShellExplorerFetchBusyPolicy(isBusy) {
    const busy = Boolean(isBusy);
    window.__kgwExplorerFetchBusy = busy;

    const candidates = document.querySelectorAll(
      "button,a,[role='tab'],[data-tab],[data-target],[aria-controls]"
    );

    candidates.forEach((element) => {
      if (!isTopTabControl(element)) return;

      const name = tabNameOf(element);

      if (!busy) {
        setTabControlEnabled(element, true);
        return;
      }

      setTabControlEnabled(element, name === "explorer" || name === "log");
    });
  };

  window.addEventListener("kgw:explorer-fetch-busy", (event) => {
    window.kgwApplyShellExplorerFetchBusyPolicy(Boolean(event.detail?.busy));
  });

  document.addEventListener(
    "click",
    (event) => {
      if (!window.__kgwExplorerFetchBusy) return;

      const tab = event.target?.closest?.(
        "button,a,[role='tab'],[data-tab],[data-target],[aria-controls]"
      );

      if (!isTopTabControl(tab)) return;

      const name = tabNameOf(tab);

      if (name === "explorer" || name === "log") {
        setTabControlEnabled(tab, true);
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      if (typeof event.stopImmediatePropagation === "function") {
        event.stopImmediatePropagation();
      }
    },
    true
  );

  window.kgwApplyShellExplorerFetchBusyPolicy(Boolean(window.__kgwExplorerFetchBusy));
})();

import { kgwLog, kgwFatal, createLogger } from "./src/core/shell-logger.js";
import { KGW_TABS } from "./src/tabs/tab-registry.js";

const shellLog = createLogger("shell");

const KGW_THEME_KEY = "kgw-shell-theme";

const loadedModules = new Map();
const mountedTabs = new Set();
const initializedTabs = new Set();

let bootRunning = false;
let bootDone = false;

/* KGW_SHELL_BOOT_CONTENT_SETTINGS_BOUNDARY_PATCH_R63F
 * Targeted async-safe boundary repair for canonical shell startup content.
 * This keeps R59C as the single shell display and active-tab owner.
 */
function kgwShellCanonicalDefaultTabR63F() {
  return "kaspa-node";
}

function kgwShellCanonicalVisibleTabsR63F() {
  /* KGW_SHELL_R63F_STARTUP_BOUNDARY_NOT_DISPLAY_FILTER_R71C
   * R63F is a startup safety boundary, not a permanent Settings display filter.
   * All real tab visibility must come from the R59C/R71 shell display owners.
   */
  try {
    const ids = typeof allTabIds === "function" ? allTabIds() : [];
    if (Array.isArray(ids) && ids.length) {
      return new Set(ids);
    }
  } catch (_) {}

  return new Set(["explorer", "kaspa-node", "kaspa-bridge", "analysis", "top-addresses", "log", "settings"]);
}

function kgwShellResolveStartupTabR63F(candidate, reason = "startup") {
  const requested = String(candidate || "").replace(/^#/, "").trim();
  const canonical = (() => {
    try {
      return window.kgwShellDisplayAndActiveTabOwnerR59C || null;
    } catch (_) {
      return null;
    }
  })();

  if (canonical && typeof canonical.resolveTabId === "function") {
    const resolved = canonical.resolveTabId(requested, "r63f-" + reason);
    if (resolved) return resolved;
  }

  const allowed = kgwShellCanonicalVisibleTabsR63F();
  if (requested && allowed.has(requested)) return requested;

  return kgwShellCanonicalDefaultTabR63F();
}

function kgwShellApplyDisplayOwnerBeforeBootR63F(reason = "boot-before-open-tab") {
  /* KGW_SHELL_ANY_SAVED_MAIN_TAB_RESTORE_R102C
   * Existing display owners still apply preferences; after each apply, schedule saved-tab restoration.
   */
  const scheduleRestore = (suffix) => {
    try { kgwShellScheduleSavedMainTabRestoreR102C(String(reason || "") + "-" + suffix); } catch (_) {}
  };

  try {
    const shell = window.kgwShellDisplayPreferencesR71;
    if (shell && typeof shell.read === "function" && typeof shell.apply === "function") {
      const stored = shell.read();
      shell.apply(stored && typeof stored === "object" ? stored : shell.defaults ? shell.defaults() : null, "r63f-" + reason);
      scheduleRestore("r71-apply");
      return true;
    }
  } catch (_) {}

  try {
    const canonical = window.kgwShellDisplayAndActiveTabOwnerR59C;
    if (canonical && typeof canonical.read === "function" && typeof canonical.applyPreferences === "function") {
      const stored = canonical.read();
      canonical.applyPreferences(stored && typeof stored === "object" ? stored : canonical.defaults(), "r63f-" + reason);
      scheduleRestore("r59c-read-apply");
      return true;
    }
  } catch (_) {}

  try {
    const canonical = window.kgwShellDisplayAndActiveTabOwnerR59C;
    if (canonical && typeof canonical.defaults === "function" && typeof canonical.applyPreferences === "function") {
      canonical.applyPreferences(canonical.defaults(), "r63f-" + reason);
      scheduleRestore("r59c-default-apply");
      return true;
    }
  } catch (_) {}

  scheduleRestore("no-apply-owner");
  return false;
}


function tabById(tabId) {
  /* KGW_SHELL_ANY_SAVED_MAIN_TAB_RESTORE_R102C */
  const requestedKnownTabR102C = kgwShellKnownMainTabR102C(tabId);
  const resolvedTabId = requestedKnownTabR102C || kgwShellResolveStartupTabR63F(tabId, "tab-by-id");
  const exact = KGW_TABS.find((tab) => tab.id === resolvedTabId);
  if (exact) return exact;

  const canonicalDefault = KGW_TABS.find((tab) => tab.id === kgwShellCanonicalDefaultTabR63F());
  if (canonicalDefault) return canonicalDefault;

  const settingsTab = KGW_TABS.find((tab) => tab.id === "settings");
  if (settingsTab) return settingsTab;

  throw new Error("No valid KGW tab configuration is available.");
}

function allTabIds() {
  return KGW_TABS.map((tab) => tab.id);
}

function ensureCss(tab) {
  if (document.querySelector(`link[data-kgw-tab-css="${tab.id}"]`)) return;

  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = tab.css;
  link.dataset.kgwTabCss = tab.id;
  document.head.appendChild(link);
}

function parseTabHtml(tab) {
  const template = document.createElement("template");
  template.innerHTML = String(tab.html || "").trim();

  const panel = template.content.querySelector(`#${CSS.escape(tab.id)}`);
  if (!panel) {
    throw new Error(`${tab.id}: tab template does not contain #${tab.id}`);
  }

  return panel;
}

function preparePanel(panel, tab, active) {
  panel.id = tab.id;
  panel.dataset.tabPanel = tab.id;
  panel.dataset.kgwMounted = "true";
  panel.dataset.kgwOwner = tab.id;

  panel.classList.add("page");
  panel.classList.toggle("active", active);

  if (active) {
    panel.hidden = false;
    panel.removeAttribute("hidden");
    panel.setAttribute("aria-hidden", "false");
    panel.style.display = "";
  } else {
    panel.classList.remove("active");
    panel.hidden = true;
    panel.setAttribute("aria-hidden", "true");
    panel.style.display = "none";
  }

  return panel;
}

function mountTabHtml(tab, active = false) {
  let current = kgwStrictEnsureSinglePanel(tab.id);
  if (!current) {
    const main = document.getElementById("kgwMain");
    if (!main) {
      throw new Error(`Missing shell main container #kgwMain`);
    }

    current = document.createElement("section");
    current.id = tab.id;
    current.className = "page";
    current.dataset.tabPanel = tab.id;
    current.hidden = true;
    current.setAttribute("aria-hidden", "true");
    main.appendChild(current);
  }

  if (current.dataset.kgwMounted === "true") {
    preparePanel(current, tab, active || current.classList.contains("active") || current.hidden === false);
    mountedTabs.add(tab.id);
    return current;
  }

  const wasActive = active || current.classList.contains("active") || current.hidden === false;
  const panel = preparePanel(parseTabHtml(tab), tab, wasActive);

  current.replaceWith(panel);
  mountedTabs.add(tab.id);

  return panel;
}

async function importTabModule(tab) {
  if (loadedModules.has(tab.id)) {
    return loadedModules.get(tab.id);
  }

  const module = await tab.module();
  loadedModules.set(tab.id, module);
  return module;
}

async function initTab(tab) {
  ensureCss(tab);
  mountTabHtml(tab, tab.id === currentSelectedTabId());

  const module = await importTabModule(tab);

  if (!initializedTabs.has(tab.id)) {
    const init = module && typeof module[tab.init] === "function"
      ? module[tab.init]
      : typeof window[tab.init] === "function"
        ? window[tab.init]
        : null;

    if (init) {
      await init();
    }

    initializedTabs.add(tab.id);
  }

  const finalPanel = document.getElementById(tab.id);
  if (finalPanel) {
    finalPanel.dataset.kgwMounted = "true";
    finalPanel.dataset.kgwInitialized = "true";
    finalPanel.dataset.kgwOwner = tab.id;
    finalPanel.classList.add("page");
  }
}

function currentSelectedTabId() {
  const activeButton =
    document.querySelector("[data-tab].active") ||
    document.querySelector('[data-tab][aria-selected="true"]');

  const buttonTab = activeButton?.dataset?.tab;
  if (buttonTab && allTabIds().includes(buttonTab)) {
    return kgwShellResolveStartupTabR63F(buttonTab, "current-selected-button");
  }

  const hash = String(window.location.hash || "").replace(/^#/, "");
  if (hash && allTabIds().includes(hash)) {
    return kgwShellResolveStartupTabR63F(hash, "current-selected-hash");
  }

  return kgwShellCanonicalDefaultTabR63F();
}

function forcePanelIntoViewport(tabId) {
  const panel = document.getElementById(tabId);
  const main = document.getElementById("kgwMain");
  if (!panel || !main) return;

  if (panel.parentElement !== main) {
    main.appendChild(panel);
  }

  panel.style.position = "absolute";
  panel.style.inset = "0";
  panel.style.width = "100%";
  panel.style.height = "100%";
  panel.style.margin = "0";
  panel.style.display = "";
  panel.style.visibility = "visible";
  panel.style.opacity = "1";
}

function kgwStrictSetImportant(node, property, value) {
  try {
    node.style.setProperty(property, value, "important");
  } catch (_) {
    node.style[property] = value;
  }
}

function kgwStrictHidePanel(panel) {
  if (!panel) return;

  panel.classList.remove("active");
  panel.hidden = true;
  panel.setAttribute("hidden", "");
  panel.setAttribute("aria-hidden", "true");
  panel.dataset.kgwShellActive = "false";
  panel.dataset.kgwShellHidden = "true";

  kgwStrictSetImportant(panel, "display", "none");
  kgwStrictSetImportant(panel, "visibility", "hidden");
  kgwStrictSetImportant(panel, "opacity", "0");
  kgwStrictSetImportant(panel, "pointer-events", "none");
  kgwStrictSetImportant(panel, "z-index", "0");
}

function kgwStrictShowPanel(panel) {
  if (!panel) return;

  panel.classList.add("page");
  panel.classList.add("active");
  panel.hidden = false;
  panel.removeAttribute("hidden");
  panel.setAttribute("aria-hidden", "false");
  panel.dataset.kgwShellActive = "true";
  panel.dataset.kgwShellHidden = "false";

  kgwStrictSetImportant(panel, "position", "absolute");
  kgwStrictSetImportant(panel, "inset", "0");
  kgwStrictSetImportant(panel, "width", "100%");
  kgwStrictSetImportant(panel, "height", "100%");
  kgwStrictSetImportant(panel, "margin", "0");
  kgwStrictSetImportant(panel, "display", "block");
  kgwStrictSetImportant(panel, "visibility", "visible");
  kgwStrictSetImportant(panel, "opacity", "1");
  kgwStrictSetImportant(panel, "pointer-events", "auto");
  kgwStrictSetImportant(panel, "z-index", "50");
}

function kgwStrictEnsureSinglePanel(tabId) {
  const main = document.getElementById("kgwMain");
  if (!main) return document.getElementById(tabId);

  const all = Array.from(document.querySelectorAll(`#${CSS.escape(tabId)}`));

  if (!all.length) {
    const placeholder = document.createElement("section");
    placeholder.id = tabId;
    placeholder.className = "page";
    placeholder.dataset.tabPanel = tabId;
    placeholder.dataset.kgwOwner = tabId;
    kgwStrictHidePanel(placeholder);
    main.appendChild(placeholder);
    return placeholder;
  }

  let keeper =
    all.find((node) => node.parentElement === main && node.dataset.kgwMounted === "true") ||
    all.find((node) => node.parentElement === main) ||
    all[0];

  if (keeper.parentElement !== main) {
    main.appendChild(keeper);
  }

  for (const node of all) {
    if (node !== keeper) {
      node.remove();
    }
  }

  keeper.classList.add("page");
  keeper.dataset.tabPanel = tabId;
  keeper.dataset.kgwOwner = tabId;

  return keeper;
}

function kgwStrictNormalizeAllPanels(selectedId) {
  const main = document.getElementById("kgwMain");
  if (!main) return;

  for (const tab of KGW_TABS) {
    const panel = kgwStrictEnsureSinglePanel(tab.id);

    if (tab.id === selectedId) {
      kgwStrictShowPanel(panel);
    } else {
      kgwStrictHidePanel(panel);
    }
  }

  /* Hide any accidental direct children in kgwMain that are not official pages. */
  Array.from(main.children).forEach((child) => {
    const isKnown = KGW_TABS.some((tab) => tab.id === child.id);
    if (!isKnown) {
      kgwStrictHidePanel(child);
    }
  });
}

function kgwStrictSnapshot(selectedId) {
  const main = document.getElementById("kgwMain");
  if (!main) return { selectedId, mainExists: false };

  return {
    selectedId,
    mainChildren: Array.from(main.children).map((node) => ({
      id: node.id || "",
      className: node.className || "",
      hidden: node.hidden,
      attrHidden: node.hasAttribute("hidden"),
      shellActive: node.dataset.kgwShellActive || "",
      display: getComputedStyle(node).display,
      zIndex: getComputedStyle(node).zIndex,
      textLength: (node.textContent || "").trim().length
    }))
  };
}

function activateTab(tabId) {
  shellLog.log("activateTab", tabId);

  const selected = tabById(tabId).id;

  kgwStrictNormalizeAllPanels(selected);

  KGW_TABS.forEach((tab) => {
    const active = tab.id === selected;

    document.querySelectorAll(`[data-tab="${CSS.escape(tab.id)}"]`).forEach((button) => {
      button.classList.toggle("active", active);
      button.setAttribute("aria-selected", String(active));
    });
  });

  if (window.location.hash.replace(/^#/, "") !== selected) {
    window.history.replaceState(null, "", `#${selected}`);
  }

  shellLog.log("tab isolation snapshot", kgwStrictSnapshot(selected));
}

/* KGW_CALENDAR_TAB_LIFECYCLE_CLEANUP_R11B
   Existing shell tab lifecycle cleanup.
   main.js owns top-level tab switching through openTab(tabId).
   Calendar popovers are attached to document.body by existing Explorer/Analysis owners,
   so stale body popovers must be closed before switching tabs.
*/
function kgwCloseCalendarPopoversForTabLifecycleR11B(reason = "tab-switch") {
  document.querySelectorAll(".kgw-calendar-popover").forEach((node) => node.remove());

  document.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    delete node.dataset.kgwCalendarOpen;
    delete node.dataset.kgwCalendarScope;
  });

  document.documentElement.dataset.kgwCalendarLifecycleCleanupR11B = String(reason || "tab-switch");
}


// KGW_EXPLICIT_MAIN_TAB_TRACE_OWNER_R35C_BEGIN
function kgwMainTabTraceR35C(tabId, phase, details) {
  try {
    const safeTab = String(tabId || "unknown");
    const safePhase = String(phase || "unknown");
    const payload = {
      patch: "KGW_EXPLICIT_MAIN_TAB_TRACE_PATCH_R35C",
      owner: "main-existing-tab-owner",
      tabId: safeTab,
      phase: safePhase,
      details: details && typeof details === "object" ? details : {}
    };

    const invoke =
      window.__TAURI__?.core?.invoke ||
      window.__TAURI__?.tauri?.invoke ||
      window.__TAURI_INVOKE__ ||
      null;

    if (typeof invoke === "function") {
      invoke("kgw_frontend_button_trace_v1", {
        scope: "shell",
        net: "ui",
        action: "tab-navigation",
        phase: safePhase,
        details: JSON.stringify(payload)
      }).catch(function () {});
    }
  } catch (_) {}
}
// KGW_EXPLICIT_MAIN_TAB_TRACE_OWNER_R35C_END

/* KGW_SHELL_LAST_FULL_LOCATION_RESTORE_R101W2
 * Corrected version of R101W.
 * Persists/restores the last valid top-level app tab without removing async from openTab/boot.
 */
const KGW_SHELL_LAST_MAIN_TAB_KEY_R101W2 = "kgw.shell.lastMainTab";

function kgwShellIsValidMainTabR101W2(tabId) {
  const candidate = String(tabId || "").trim();
  if (!candidate) return false;
  try { tabById(candidate); return true; } catch (_) { return false; }
}

function kgwShellReadLastMainTabR101W2() {
  try {
    const saved = localStorage.getItem(KGW_SHELL_LAST_MAIN_TAB_KEY_R101W2);
    return kgwShellIsValidMainTabR101W2(saved) ? saved : "";
  } catch (_) { return ""; }
}

function kgwShellSaveLastMainTabR101W2(tabId) {
  const candidate = String(tabId || "").trim();
  if (!kgwShellIsValidMainTabR101W2(candidate)) return "";
  try { localStorage.setItem(KGW_SHELL_LAST_MAIN_TAB_KEY_R101W2, candidate); } catch (_) {}
  return candidate;
}

function kgwShellResolveStartupTabR101W2(hashValue, reason = "startup") {
  /* R101Y_SAFE_USER_LOCATION_PERSISTENCE_FIX */
  /* KGW_SHELL_ANY_SAVED_MAIN_TAB_RESTORE_R102C
   * Saved user location is app state, not a temporary button visibility decision.
   */
  const saved = kgwShellSavedMainTabR102C();
  if (saved) return saved;

  const requested = kgwShellKnownMainTabR102C(hashValue);
  if (requested) return requested;

  return kgwShellResolveStartupTabR63F("", reason + "-default");
}

/* KGW_SHELL_ANY_SAVED_MAIN_TAB_RESTORE_R102C
 * Restore any known saved main tab, even when its button is temporarily hidden during boot/display preference loading.
 * Corrected parser patch: function body extraction ignores default parameter objects such as options = {}.
 */
let kgwShellPendingSavedMainTabR102C = "";

function kgwShellKnownMainTabR102C(tabId) {
  const candidate = String(tabId || "").replace(/^#/, "").trim();
  if (!candidate) return "";
  try {
    if (Array.isArray(KGW_TABS) && KGW_TABS.some((tab) => tab && tab.id === candidate)) return candidate;
  } catch (_) {}
  return "";
}

function kgwShellSavedMainTabR102C() {
  try {
    return kgwShellKnownMainTabR102C(kgwShellReadLastMainTabR101W2());
  } catch (_) {
    return "";
  }
}

function kgwShellShouldBypassDisplayFilterR102C(tabId, options) {
  const requested = kgwShellKnownMainTabR102C(tabId);
  if (!requested) return false;
  const openOptions = options && typeof options === "object" ? options : {};
  if (openOptions.allowHiddenSavedTab === true) return true;
  const saved = kgwShellSavedMainTabR102C();
  return Boolean(saved && saved === requested && openOptions.persist !== true);
}

function kgwShellScheduleSavedMainTabRestoreR102C(reason = "schedule") {
  const saved = kgwShellSavedMainTabR102C();
  if (!saved) return false;
  kgwShellPendingSavedMainTabR102C = saved;

  [0, 80, 250, 800, 1600].forEach((delay) => {
    window.setTimeout(() => {
      const pending = kgwShellKnownMainTabR102C(kgwShellPendingSavedMainTabR102C || kgwShellSavedMainTabR102C());
      if (!pending) return;

      try {
        const activeButton = typeof kgwShellDisplayOwnerActiveButtonR59C === "function" ? kgwShellDisplayOwnerActiveButtonR59C() : null;
        const activeTabId = activeButton && activeButton.dataset ? String(activeButton.dataset.tab || "") : String(window.location.hash || "").replace(/^#/, "");
        if (activeTabId === pending) return;
      } catch (_) {}

      try {
        kgwMainTabTraceR35C(pending, "r102c-saved-main-tab-deferred-restore", {
          reason: String(reason || ""),
          delay,
          activeHash: String(window.location.hash || ""),
          visibleNow: typeof kgwShellDisplayOwnerTabIdVisibleR59C === "function" ? kgwShellDisplayOwnerTabIdVisibleR59C(pending) : null
        });
      } catch (_) {}

      try {
        void openTab(pending, {
          persist: false,
          allowHiddenSavedTab: true,
          reason: "r102c-deferred-saved-main-tab-restore"
        });
      } catch (_) {}
    }, delay);
  });

  return true;
}

async function openTab(tabId, options = {}) {
  shellLog.log("openTab", tabId);

  const requestedTabId = String(tabId || "");
  const openOptions = options && typeof options === "object" ? options : {};
  const openReason = String(openOptions.reason || "programmatic");
  const shouldPersistLastMainTabR101Y = openOptions.persist === true;
  const shouldBypassDisplayFilterR102C = kgwShellShouldBypassDisplayFilterR102C(requestedTabId, openOptions);
  /* KGW_SHELL_DISPLAY_OWNER_SCOPE_REPAIR_R60 */
  /* KGW_SHELL_LAST_FULL_LOCATION_RESTORE_R101W2 */
  /* R101Y_SAFE_USER_LOCATION_PERSISTENCE_FIX */
  /* KGW_SHELL_ANY_SAVED_MAIN_TAB_RESTORE_R102C */
  const kgwCanonicalOwnerR60 = (() => {
    try { return window.kgwShellDisplayAndActiveTabOwnerR59C || null; } catch (_) { return null; }
  })();

  const resolvedTabId = shouldBypassDisplayFilterR102C
    ? kgwShellKnownMainTabR102C(requestedTabId)
    : (kgwCanonicalOwnerR60 && typeof kgwCanonicalOwnerR60.resolveTabId === "function"
      ? kgwCanonicalOwnerR60.resolveTabId(requestedTabId, "openTab")
      : requestedTabId);

  if (resolvedTabId && resolvedTabId !== requestedTabId) {
    try {
      kgwMainTabTraceR35C(resolvedTabId, "r59c-canonical-open-tab-resolve", {
        requestedTabId,
        resolvedTabId,
        activeHash: String(window.location.hash || ""),
        openReason,
        persistAllowed: shouldPersistLastMainTabR101Y,
        bypassDisplayFilterR102C: shouldBypassDisplayFilterR102C
      });
    } catch (_) {}
    tabId = resolvedTabId;
  } else if (resolvedTabId) {
    tabId = resolvedTabId;
  }

  kgwCloseCalendarPopoversForTabLifecycleR11B("open-tab-before-init");

  const tab = tabById(tabId);
  await initTab(tab);
  activateTab(tab.id);

  if (shouldPersistLastMainTabR101Y) {
    kgwShellSaveLastMainTabR101W2(tab.id);
    kgwShellPendingSavedMainTabR102C = tab.id;
  }

  kgwMainTabTraceR35C(tab.id, "r35c-open-tab", {
    requestedTabId: String(requestedTabId || ""),
    activeHash: String(window.location.hash || ""),
    persistedLastMainTab: kgwShellReadLastMainTabR101W2(),
    persistAllowed: shouldPersistLastMainTabR101Y,
    openReason,
    bypassDisplayFilterR102C: shouldBypassDisplayFilterR102C
  });

  window.dispatchEvent(new CustomEvent("kgw:tab-opened", { detail: { tabId: tab.id } }));

  if (typeof window.kgwReapplyLanguageSilentlyR89 === "function") {
    window.setTimeout(() => window.kgwReapplyLanguageSilentlyR89("tab-opened-after-mount"), 0);
  }
}

function bindNavigation() {
  document.querySelectorAll("[data-tab]").forEach((button) => {
    if (button.dataset.kgwBound === "true") return;
    button.dataset.kgwBound = "true";

    button.addEventListener("click", async (event) => {
      try {
        const trusted = Boolean(event && event.isTrusted);
        kgwMainTabTraceR35C(button.dataset.tab, "r35c-tab-click", {
          trusted,
          text: String(button.textContent || "").trim(),
          id: String(button.id || ""),
          className: String(button.className || ""),
          persistRequested: trusted
        });

        await openTab(button.dataset.tab, {
          persist: trusted,
          reason: trusted ? "trusted-main-tab-click" : "untrusted-main-tab-click"
        });
      } catch (error) {
        kgwFatal(error, "shell");
      }
    });
  });
}

function readTheme() {
  try {
    return localStorage.getItem(KGW_THEME_KEY) || "dark";
  } catch (_) {
    return "dark";
  }
}

function applyTheme(theme) {
  const value = ["dark", "superhero", "kaspa", "slate", "midnight", "blue"].includes(theme)
    ? theme
    : "dark";

  document.documentElement.dataset.kgwTheme = value;

  const select = document.getElementById("shellThemeSelect");
  if (select && select.value !== value) {
    select.value = value;
  }

  try {
    localStorage.setItem(KGW_THEME_KEY, value);
  } catch (_) {}

  window.dispatchEvent(new CustomEvent("kgw:theme-changed", { detail: { theme: value } }));
}

function bindShellControls() {
  const themeSelect = document.getElementById("shellThemeSelect");

  if (themeSelect && themeSelect.dataset.kgwBound !== "true") {
    themeSelect.dataset.kgwBound = "true";
    themeSelect.addEventListener("change", () => applyTheme(themeSelect.value));
  }

  applyTheme(readTheme());
}

function showFatal(error) {
  const main = document.getElementById("kgwMain");
  if (!main) return;

  main.innerHTML = `
    <pre style="padding:16px;color:#fca5a5;white-space:pre-wrap;font:13px Consolas,monospace">
Shell boot failed:
${String(error && error.stack || error)}
    </pre>
  `;
}

async function boot() {
  shellLog.log("boot start", location.href);
  if (bootRunning || bootDone) return;

  bootRunning = true;

  try {
    bindShellControls();
    bindNavigation();

    const savedBeforeDisplayR102C = kgwShellSavedMainTabR102C();
    if (savedBeforeDisplayR102C) kgwShellPendingSavedMainTabR102C = savedBeforeDisplayR102C;

    kgwShellApplyDisplayOwnerBeforeBootR63F("boot-before-open-tab");

    const hash = String(window.location.hash || "").replace(/^#/, "");
    const initial = kgwShellResolveStartupTabR101W2(hash, "boot-initial");

    await openTab(initial, {
      persist: false,
      allowHiddenSavedTab: Boolean(savedBeforeDisplayR102C && savedBeforeDisplayR102C === initial),
      reason: "boot-restore"
    });

    kgwShellScheduleSavedMainTabRestoreR102C("boot-after-open-tab");

    bootDone = true;
  } catch (error) {
    kgwFatal(error, "shell");
  } finally {
    bootRunning = false;
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", boot);
} else {
  boot();
}

window.kgwOpenTab = openTab;
window.kgwActivateTab = activateTab;
window.kgwApplyTheme = applyTheme;

import("./src/core/header-live-metrics.js");


/* TX_BUSY_PROGRESS_UI_1_GLOBAL_CONTROLLER
   One global UI busy controller for transaction fetch:
   - Locks header selects outside Explorer: Language / Currency / Theme.
   - Keeps Explorer and Log navigation usable via existing shell policy.
   - Shows an indeterminate progress bar in the bottom-left Ready area.
   - Does not create a second fetch path.
*/
(function installKgwGlobalFetchBusyUi() {
  if (window.__kgwGlobalFetchBusyUiInstalled) return;
  window.__kgwGlobalFetchBusyUiInstalled = true;

  const STATE = {
    busy: false,
    lastText: "Ready"
  };

  function isInsideExplorer(element) {
    return Boolean(element?.closest?.("#explorer,.explorer-python-root"));
  }

  function kgwSaveAndSetDisabled(element, disabled) {
    if (!element) return;

    const canDisable = "disabled" in element;

    if (disabled) {
      if (element.dataset.kgwGlobalPrevDisabled === undefined) {
        element.dataset.kgwGlobalPrevDisabled = canDisable && element.disabled ? "true" : "false";
      }

      if (canDisable) element.disabled = true;

      element.setAttribute("aria-disabled", "true");
      element.classList.add("disabled");
      element.classList.add("is-disabled");
      element.style.pointerEvents = "none";
      element.style.cursor = "not-allowed";
      return;
    }

    const previous = element.dataset.kgwGlobalPrevDisabled;

    if (previous !== undefined) {
      if (canDisable) element.disabled = previous === "true";
      delete element.dataset.kgwGlobalPrevDisabled;
    } else if (canDisable) {
      element.disabled = false;
    }

    element.removeAttribute("aria-disabled");
    element.classList.remove("disabled");
    element.classList.remove("is-disabled");
    element.style.pointerEvents = "";
    element.style.cursor = "";
  }

  function kgwHeaderSelects() {
    return Array.from(document.querySelectorAll("select"))
      .filter((select) => {
        if (select.id === "shellLanguageSelect" || select.id === "shellCurrencySelect") return false;
        return !isInsideExplorer(select);
      });
  }

  function kgwLockHeaderDropdowns(busy) {
    const selects = kgwHeaderSelects();

    for (const select of selects) {
      kgwSaveAndSetDisabled(select, Boolean(busy));
    }

    console.log("[KGW Explorer][busy-ui] header dropdown lock", {
      busy: Boolean(busy),
      selects: selects.map((x) => ({
        id: x.id || "",
        name: x.name || "",
        value: x.value || "",
        text: String(x.closest("label")?.textContent || x.getAttribute("aria-label") || "").trim()
      }))
    });
  }

  function kgwEnsureProgressStyle() {
    if (document.getElementById("kgwInlineReadyProgressStyle")) return;

    const style = document.createElement("style");
    style.id = "kgwInlineReadyProgressStyle";
    style.textContent = `
      #kgwShellReadyStatus.kgw-ready-fetching {
        display: flex;
        align-items: center;
        gap: 10px;
        min-width: 0;
        width: 100%;
        overflow: hidden;
      }

      #kgwShellReadyStatus .kgw-ready-progress-track {
        width: 280px;
        max-width: 34vw;
        min-width: 160px;
        height: 8px;
        overflow: hidden;
        background: rgba(96, 165, 250, 0.18);
        border: 0;
        border-radius: 999px;
        flex: 0 0 auto;
      }

      #kgwShellReadyStatus .kgw-ready-progress-bar {
        height: 100%;
        width: 38%;
        border-radius: 999px;
        background: linear-gradient(90deg, rgba(125,211,252,.2), rgba(125,211,252,.95), rgba(125,211,252,.2));
        animation: kgwReadyProgressSlide 1.05s linear infinite;
      }

      #kgwShellReadyStatus .kgw-ready-progress-text {
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      @keyframes kgwReadyProgressSlide {
        0% { transform: translateX(-130%); }
        100% { transform: translateX(290%); }
      }
    `;

    document.head.appendChild(style);
  }


  function kgwEnsureProgressNode() {
    kgwEnsureProgressStyle();

    const ready = document.getElementById("kgwShellReadyStatus");
    if (ready) {
      if (ready.dataset.kgwOriginalReady === undefined) {
        ready.dataset.kgwOriginalReady = ready.textContent || "Ready";
      }

      const oldFloating = document.getElementById("kgwGlobalFetchProgress");
      if (oldFloating) oldFloating.remove();

      return ready;
    }

    let node = document.getElementById("kgwGlobalFetchProgress");
    if (node) return node;

    node = document.createElement("div");
    node.id = "kgwGlobalFetchProgress";
    node.dataset.busy = "false";
    node.style.display = "none";
    document.body.appendChild(node);
    return node;
  }


  function kgwSetProgressVisible(busy, text) {
    const node = kgwEnsureProgressNode();

    STATE.busy = Boolean(busy);

    if (typeof text === "string" && text.trim()) {
      STATE.lastText = text.trim();
    }

    if (node.id === "kgwShellReadyStatus") {
      if (STATE.busy) {
        node.dataset.busy = "true";
        node.classList.add("kgw-ready-fetching");
        node.innerHTML = `
          <span class="kgw-ready-progress-track" aria-hidden="true">
            <span class="kgw-ready-progress-bar"></span>
          </span>
          <span class="kgw-ready-progress-text"></span>
        `;

        const label = node.querySelector(".kgw-ready-progress-text");
        if (label) label.textContent = STATE.lastText || "Fetching transactions...";
      } else {
        node.dataset.busy = "false";
        node.classList.remove("kgw-ready-fetching");
        node.textContent = (window.kgwT ? window.kgwT("runtime.ready") : "Ready");
      }

      console.log("[KGW Explorer][busy-ui] ready inline progress", {
        busy: STATE.busy,
        text: node.textContent || ""
      });

      return;
    }

    node.dataset.busy = STATE.busy ? "true" : "false";
    node.textContent = STATE.busy ? (STATE.lastText || "Fetching...") : "Ready";
  }


  window.kgwSetGlobalFetchBusy = function kgwSetGlobalFetchBusy(busy, detail = {}) {
    const isBusy = Boolean(busy);
    const text = String(detail?.text || detail?.status || "").trim();

    kgwLockHeaderDropdowns(isBusy);
    kgwSetProgressVisible(isBusy, text || (isBusy ? "Fetching transactions..." : "Ready"));
  };

  window.kgwSetGlobalFetchProgressText = function kgwSetGlobalFetchProgressText(text) {
    if (!STATE.busy && !window.__kgwExplorerFetchBusy) return;

    const clean = String(text || "").trim();
    if (!clean) return;

    kgwSetProgressVisible(true, clean);
  };

  window.addEventListener("kgw:explorer-fetch-busy", (event) => {
    const busy = Boolean(event.detail?.busy);

    window.kgwSetGlobalFetchBusy(busy, {
      text: busy ? "Fetching transactions..." : "Ready"
    });
  });

  for (const eventName of ["pointerdown", "click", "input", "change"]) {
    document.addEventListener(
      eventName,
      (event) => {
        if (!window.__kgwExplorerFetchBusy) return;

        const target = event.target?.closest?.("select,button,input,textarea,a,[role='button']");
        if (!target) return;

        if (isInsideExplorer(target)) return;

        if (target.tagName === "SELECT") {
          if (target.id === "shellLanguageSelect" || target.id === "shellCurrencySelect") {
            return;
          }

          event.preventDefault();
          event.stopPropagation();
          event.stopImmediatePropagation();

          console.warn("[KGW Explorer][busy-ui] blocked non-shell select while fetch is busy", {
            eventType: eventName,
            id: target.id || "",
            value: target.value || ""
          });
        }
      },
      true
    );
  }
console.log("[KGW Explorer][busy-ui] global controller installed");
})();


/* TX_BOTTOM_LINKS_SINGLE_OWNER_3
   One handler only for footer links.
*/
(function installKgwBottomLinksSingleOwner() {
  if (window.__kgwBottomLinksSingleOwnerInstalled) return;
  window.__kgwBottomLinksSingleOwnerInstalled = true;

  const links = {
    github: "https://github.com/KaspaPulse",
    donations: "https://kaspa.stream/addresses/kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g",
    twitter: "https://x.com/KaspaPulse"
  };

  async function openExternal(url) {
    const clean = String(url || "").trim();
    if (!/^https:\/\//i.test(clean)) return;

    try {
      const opener =
        window.__TAURI__?.opener?.openUrl ||
        window.__TAURI__?.opener?.open ||
        window.__TAURI__?.shell?.open ||
        window.__TAURI__?.shell?.openUrl;

      if (typeof opener === "function") {
        await opener(clean);
        return;
      }
    } catch (error) {
      console.warn("[KGW links] Tauri opener failed", error);
    }

    try {
      window.open(clean, "_blank", "noopener,noreferrer");
    } catch (error) {
      console.warn("[KGW links] window.open failed", error);
      location.href = clean;
    }
  }

  document.addEventListener("click", (event) => {
    const button = event.target?.closest?.("[data-kgw-link]");
    if (!button) return;

    const key = String(button.dataset.kgwLink || "").trim().toLowerCase();
    const url = links[key];

    if (!url) return;

    event.preventDefault();
    event.stopPropagation();

    console.log("[KGW links] open", { key, url });
    void openExternal(url);
  }, true);
})();

/* KGW_FOOTER_LINKS_OWNER_PATCH_V1
   Existing-owner footer link fix.
   Do not add a new external-links layer.
   This patches only the current footer owner path: index.html data-kgw-link + main.js.
*/
(function () {
  "use strict";

  const footerUrls = {
    github: "https://github.com/KaspaPulse",
    donations: "https://kaspa.stream/addresses/kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g",
    twitter: "https://x.com/KaspaPulse"
  };

  function getTauriInvoke() {
    return (
      window.__TAURI__?.core?.invoke ||
      window.__TAURI__?.tauri?.invoke ||
      window.__TAURI_INVOKE__ ||
      null
    );
  }

  async function openFooterUrl(url) {
    if (!url) return false;

    try {
      if (window.__TAURI__?.opener?.openUrl) {
        await window.__TAURI__.opener.openUrl(url);
        return true;
      }
    } catch (_) {}

    try {
      if (window.__TAURI__?.shell?.open) {
        await window.__TAURI__.shell.open(url);
        return true;
      }
    } catch (_) {}

    const invoke = getTauriInvoke();
    if (typeof invoke === "function") {
      const attempts = [
        ["open_external_url", { url }],
        ["plugin:opener|open_url", { url }],
        ["plugin:opener|openUrl", { url }],
        ["plugin:shell|open", { path: url }],
        ["open_external_url", { url }],
        ["open_url", { url }]
      ];

      for (const [command, args] of attempts) {
        try {
          await invoke(command, args);
          return true;
        } catch (_) {}
      }
    }

    try {
      const opened = window.open(url, "_blank", "noopener,noreferrer");
      return !!opened;
    } catch (_) {
      return false;
    }
  }

  function normalizeFooterButtons() {
    document.querySelectorAll("[data-kgw-link]").forEach((el) => {
      const key = String(el.getAttribute("data-kgw-link") || "").trim();
      const url = footerUrls[key];
      if (!url) return;

      el.setAttribute("data-kgw-footer-url", url);
      el.setAttribute("title", url);
      el.style.cursor = "pointer";
    });
  }

  function installFooterLinksOwnerPatch() {
    normalizeFooterButtons();

    if (document.documentElement.dataset.kgwFooterLinksOwnerPatch === "1") return;
    document.documentElement.dataset.kgwFooterLinksOwnerPatch = "1";

    document.addEventListener(
      "click",
      async (event) => {
        const target = event.target.closest("[data-kgw-link]");
        if (!target) return;

        const key = String(target.getAttribute("data-kgw-link") || "").trim();
        const url = footerUrls[key] || target.getAttribute("data-kgw-footer-url");
        if (!url) return;

        event.preventDefault();
        event.stopPropagation();

        const ok = await openFooterUrl(url);
        if (!ok) {
          console.warn("[KGW] Failed to open footer URL:", url);
        }
      },
      true
    );
  }

  document.addEventListener("DOMContentLoaded", installFooterLinksOwnerPatch);
  setTimeout(installFooterLinksOwnerPatch, 250);
  setTimeout(installFooterLinksOwnerPatch, 1000);

  window.kgwInstallFooterLinksOwnerPatch = installFooterLinksOwnerPatch;
})();

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
    shellLog.log("canonical tab resolved", {
      patch: "R59C",
      reason,
      requestedTab: requested || null,
      fallbackTab: fallback || null
    });
  } catch (_) {}

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
    kgwShellPendingSavedMainTabR102C = savedTabId;
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
    } catch (_) {}
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
  } catch (_) {}
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
    } catch (_) {}
  });

  try {
    document.querySelectorAll("select").forEach((node) => {
      if (kgwShellSelectLooksLikeSetR75(node, universe, kind) && !nodes.includes(node)) {
        nodes.push(node);
      }
    });
  } catch (_) {}

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
    } catch (_) {}
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
    } catch (_) {}
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
  } catch (_) {}
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
  } catch (_) {}
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
    } catch (_) {}
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
  } catch (_) {}

  try {
    kgwMainTabTraceR35C("settings", "r78-direct-shell-apply", {
      reason: String(reason || ""),
      languages: normalized.languages.join(","),
      currencies: normalized.currencies.join(","),
      tabs: normalized.tabs.join(",")
    });
  } catch (_) {}

  return normalized;
}

try {
  window.kgwShellApplyDisplayPreferencesDirectR73 = kgwShellApplyDisplayPreferencesDirectR73;
} catch (_) {}

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
    } catch (_) {}

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
