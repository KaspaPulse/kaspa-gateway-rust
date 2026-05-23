
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

function tabById(tabId) {
  return KGW_TABS.find((tab) => tab.id === tabId) || KGW_TABS[0];
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
  if (buttonTab && allTabIds().includes(buttonTab)) return buttonTab;

  const hash = String(window.location.hash || "").replace(/^#/, "");
  if (hash && allTabIds().includes(hash)) return hash;

  return "explorer";
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

async function openTab(tabId) {
  shellLog.log("openTab", tabId);

  kgwCloseCalendarPopoversForTabLifecycleR11B("open-tab-before-init");

  const tab = tabById(tabId);
  await initTab(tab);
  activateTab(tab.id);

  kgwMainTabTraceR35C(tab.id, "r35c-open-tab", {
    requestedTabId: String(tabId || ""),
    activeHash: String(window.location.hash || "")
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
        kgwMainTabTraceR35C(button.dataset.tab, "r35c-tab-click", {
          trusted: Boolean(event && event.isTrusted),
          text: String(button.textContent || "").trim(),
          id: String(button.id || ""),
          className: String(button.className || "")
        });

        await openTab(button.dataset.tab);
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

    const hash = String(window.location.hash || "").replace(/^#/, "");
    const initial = allTabIds().includes(hash) ? hash : "explorer";

    await openTab(initial);

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
    return {
      languages: keys(languageOptions),
      currencies: keys(currencyOptions),
      tabs: keys(tabOptions)
    };
  }

  function uniqueKnown(values, knownValues, fallbackValues) {
    const known = new Set(knownValues);
    const clean = Array.from(new Set(Array.isArray(values) ? values : []))
      .filter((value) => known.has(value));

    return clean.length > 0 ? clean : [fallbackValues[0]];
  }

  function normalize(input) {
    const base = defaults();
    const prefs = input && typeof input === "object" ? input : base;

    return {
      languages: uniqueKnown(prefs.languages, base.languages, base.languages),
      currencies: uniqueKnown(prefs.currencies, base.currencies, base.currencies),
      tabs: uniqueKnown(prefs.tabs, base.tabs, base.tabs)
    };
  }

  function readCanonicalSettingsState() {
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

      const hasAny = prefs.languages.length > 0 || prefs.currencies.length > 0 || prefs.tabs.length > 0;
      return hasAny ? normalize(prefs) : null;
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

  function applyTabs(selectedTabs) {
    const visibleTabs = new Set(selectedTabs);
    const tabButtons = Array.from(document.querySelectorAll("[data-tab]"));

    let activeWasHidden = false;
    let firstVisible = null;
    let settingsButton = null;

    for (const button of tabButtons) {
      const tabId = button.dataset.tab;
      const isSettings = tabId === "settings";
      const visible = isSettings || visibleTabs.has(tabId);

      if (isSettings) settingsButton = button;
      if (!firstVisible && visible && !isSettings) firstVisible = button;

      if (!visible && (button.classList.contains("active") || button.classList.contains("is-active"))) {
        activeWasHidden = true;
      }

      if (button.hidden !== !visible) button.hidden = !visible;
      /* KGW_TAB_VISIBILITY_UNIFIED_OWNER_FIX_R1
       * main.js is the only runtime owner for top tab visibility.
       * Use inline !important when hiding because legacy compact tab CSS uses display !important.
       */
      if (visible) {
        button.style.removeProperty("display");
      } else {
        button.style.setProperty("display", "none", "important");
      }
      button.setAttribute("aria-hidden", visible ? "false" : "true");
      button.dataset.kgwDisplayVisible = visible ? "true" : "false";
    }

    if (activeWasHidden) {
      const target = firstVisible || settingsButton;
      if (target) window.setTimeout(() => target.click(), 0);
    }
  }

  function apply(input, reason = "apply") {
    const prefs = save(input || read());

    rebuildSelect("shellLanguageSelect", languageOptions, prefs.languages);
    rebuildSelect("shellCurrencySelect", currencyOptions, prefs.currencies);
    applyTabs(prefs.tabs);

    document.documentElement.dataset.kgwDisplayPreferencesR71 = reason;
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
