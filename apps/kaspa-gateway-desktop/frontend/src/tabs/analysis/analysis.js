import {
  analysisViewApplyFilter,
  analysisViewFilteredRows,
  analysisViewRenderRows,
  analysisViewSetData,
  installRustAnalysisBinding
} from "./analysis-rust-binding.js";

function root() {
  return document.getElementById("analysis");
}

function q(selector) {
  const r = root();
  return r ? r.querySelector(selector) : null;
}

function qa(selector) {
  const r = root();
  return r ? Array.from(r.querySelectorAll(selector)) : [];
}

function log() {
  if (typeof window.kgwCreateLogger === "function") {
    return window.kgwCreateLogger("analysis");
  }
  return { log: () => {}, warn: () => {}, error: () => {} };
}

function toWesternDigits(value) {
  return String(value ?? "")
    .replace(/[٠-٩]/g, (digit) => String("٠١٢٣٤٥٦٧٨٩".indexOf(digit)))
    .replace(/[۰-۹]/g, (digit) => String("۰۱۲۳۴۵۶۷۸۹".indexOf(digit)));
}

function setText(selector, value) {
  const node = q(selector);
  if (node) node.textContent = String(value ?? "");
}

function applyFilter() {
  return analysisViewApplyFilter();
}

function renderRows() {
  return analysisViewRenderRows();
}

function setAnalysisData(payload = {}) {
  return analysisViewSetData(payload);
}

function kgwAnalysisPythonBindR9() {
  return analysisViewRenderRows();
}
/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_ANALYSIS_OWNER_START */
/* KGW_CALENDAR_EXISTING_OWNER_REGEX_FIX_R4: fixed regex escaping only; no new owner layer.\n * KGW_CALENDAR_EXISTING_OWNER_DOM_CSS_FIX_R6: robust DOM contract + body-attached compact popover inside existing owner.\n * KGW_CALENDAR_SCOPED_POPOVER_OWNER_FIX_R7: scope-isolated popovers per tab.\n * KGW_CALENDAR_SINGLE_ACTIVE_POPOVER_FIX_R8: removes stale body-attached popovers before opening current tab calendar.
/* */
function kgwAnalysisPad2(value) {
  return String(value).padStart(2, "0");
}

function kgwAnalysisTodayIso() {
  const now = new Date();
  return `${now.getFullYear()}-${kgwAnalysisPad2(now.getMonth() + 1)}-${kgwAnalysisPad2(now.getDate())}`;
}

function kgwAnalysisIsoFromDate(date) {
  return `${date.getFullYear()}-${kgwAnalysisPad2(date.getMonth() + 1)}-${kgwAnalysisPad2(date.getDate())}`;
}

function kgwAnalysisCleanIso(value, fallbackValue = kgwAnalysisTodayIso()) {
  const clean = toWesternDigits(value || fallbackValue).replace(/[^0-9-]/g, "").slice(0, 10);
  return /^\d{4}-\d{2}-\d{2}$/.test(clean) ? clean : fallbackValue;
}

function kgwAnalysisParseIso(value, fallbackValue = kgwAnalysisTodayIso()) {
  const clean = kgwAnalysisCleanIso(value, fallbackValue);
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(clean);
  if (!match) return kgwAnalysisParseIso(fallbackValue, kgwAnalysisTodayIso());

  const year = Number(match[1]);
  const month = Number(match[2]) - 1;
  const day = Number(match[3]);
  const date = new Date(year, month, day);

  if (date.getFullYear() !== year || date.getMonth() !== month || date.getDate() !== day) {
    return kgwAnalysisParseIso(fallbackValue, kgwAnalysisTodayIso());
  }

  return date;
}

function kgwAnalysisMonthLabel(year, monthIndex) {
  return new Intl.DateTimeFormat("en-US", {
    month: "long",
    year: "numeric",
    calendar: "gregory",
    numberingSystem: "latn"
  }).format(new Date(year, monthIndex, 1));
}


// KGW_ANALYSIS_SAFE_CONTROLS_TRACE_PATCH_R50B
function kgwAnalysisUiTraceR50B(action, phase, details) {
  try {
    const safeAction = String(action || "analysis-ui");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "analysis",
      net: "ui",
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_ANALYSIS_SAFE_CONTROLS_TRACE_PATCH_R50B",
        owner: "analysis-existing-safe-owners",
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
    if (typeof invoke === "function") invoke("kgw_frontend_button_trace_v1", args).catch(function () {});
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
}
function kgwAnalysisEscapeSelector(id) {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") return CSS.escape(id);
  return String(id).replace(/["\\]/g, "\\$&");
}

// KGW_ANALYSIS_FILTER_BAR_REBUILD_SINGLE_OWNER_R23
function kgwAnalysisMarkDateTouchedR18(node) {
  if (node) node.dataset.kgwAnalysisUserTouched = "true";
}

function kgwAnalysisBindDateTouchR18() {
  ["analysisFromDate", "analysisFromDateNative", "analysisToDate", "analysisToDateNative"].forEach((id) => {
    const node = q("#" + id);
    if (!node || node.dataset.kgwAnalysisTouchR23 === "true") return;
    node.dataset.kgwAnalysisTouchR23 = "true";
    ["input", "change"].forEach((eventName) => {
      node.addEventListener(eventName, (event) => {
        if (event && event.isTrusted) kgwAnalysisMarkDateTouchedR18(node);
      });
    });
  });
}

function kgwAnalysisEnsureTodayToDateR18(force = false) {
  const range = kgwAnalysisRange();
  if (!range || !range.toInput) return "";

  const today = kgwAnalysisTodayIso();
  const touched = range.toInput.dataset.kgwAnalysisUserTouched === "true"
    || (range.toNative && range.toNative.dataset.kgwAnalysisUserTouched === "true");

  if (force || !touched || !range.toInput.value || range.toInput.value !== today) {
    if (force || !touched) {
      range.toInput.dataset.kgwAnalysisOwnerDefaultR23 = "today";
      if (range.toNative) range.toNative.dataset.kgwAnalysisOwnerDefaultR23 = "today";
      return kgwAnalysisSetDate(range.toInput, range.toNative, today);
    }
  }

  return kgwAnalysisCleanIso(range.toInput.value, today);
}

function kgwAnalysisFilterBarButtonR23(textInput, nativeInput, role) {
  const owner = root();
  if (!owner || !textInput) return null;

  const textId = String(textInput.id || "");
  const nativeId = String(nativeInput && nativeInput.id ? nativeInput.id : "");

  let button =
    (textId ? owner.querySelector(`[data-date-for="${kgwAnalysisEscapeSelector(textId)}"]`) : null)
    || (nativeId ? owner.querySelector(`[data-native-for="${kgwAnalysisEscapeSelector(nativeId)}"]`) : null)
    || textInput.closest(".kgw-analysis-date-field")?.querySelector(".analysis-calendar-btn")
    || textInput.parentElement?.querySelector(".analysis-calendar-btn")
    || null;

  if (!button) {
    button = document.createElement("button");
    button.type = "button";
    button.className = "analysis-calendar-btn kgw-analysis-calendar-created-r23";
    button.textContent = "📅";
    button.setAttribute("aria-label", role === "from" ? "Open from date calendar" : "Open to date calendar");
    button.addEventListener("click", () => {
      if (!nativeInput) {
        textInput.focus();
        return;
      }

      try {
        if (typeof nativeInput.showPicker === "function") {
          nativeInput.showPicker();
        } else {
          nativeInput.focus();
          nativeInput.click();
        }
      } catch {
        nativeInput.focus();
        nativeInput.click();
      }
    });
  }

  button.classList.add("kgw-analysis-filter-calendar-button-r23", `kgw-analysis-filter-${role}-calendar-r23`);
  button.setAttribute("data-kgw-analysis-calendar-role-r23", role);
  button.setAttribute("type", button.getAttribute("type") || "button");

  return button;
}

function kgwAnalysisFilterBarLcaR23(nodes) {
  const filtered = nodes.filter(Boolean);
  if (!filtered.length) return null;

  function ancestors(node) {
    const out = [];
    let current = node;
    while (current) {
      out.push(current);
      current = current.parentElement;
    }
    return out;
  }

  const first = ancestors(filtered[0]);
  return first.find((candidate) => filtered.every((node) => candidate.contains(node))) || null;
}

function kgwAnalysisFilterBarRemoveLabelsR23(node, terms) {
  if (!node) return;

  node.setAttribute("aria-label", terms[0]);
  node.setAttribute("title", terms[0]);

  const owner = root() || document;
  const id = String(node.id || "");

  if (id) {
    owner.querySelectorAll(`label[for="${kgwAnalysisEscapeSelector(id)}"]`).forEach((label) => label.remove());
  }

  const parent = node.parentElement;
  if (parent) {
    for (const child of Array.from(parent.childNodes || [])) {
      if (child.nodeType === Node.TEXT_NODE) {
        let text = child.textContent || "";
        for (const term of terms) {
          text = text.replace(new RegExp("\\b" + term + "\\s*:?", "gi"), "");
        }
        child.textContent = text;
      }
    }
  }

  const prev = node.previousElementSibling;
  if (prev && /^(LABEL|SPAN|SMALL|B)$/i.test(prev.tagName || "")) {
    const text = String(prev.textContent || "").trim().replace(/:$/, "");
    if (terms.some((term) => text.toLowerCase() === term.toLowerCase())) {
      prev.remove();
    }
  }
}

function kgwAnalysisFilterBarMakeDateUnitR23(role, textInput, nativeInput, button) {
  const unit = document.createElement("span");
  unit.className = `kgw-analysis-filter-date-unit-r23 kgw-analysis-filter-date-${role}-r23`;
  unit.setAttribute("data-kgw-analysis-filter-date-unit-r23", role);

  if (textInput) {
    textInput.classList.add("kgw-analysis-filter-date-text-r23");
    textInput.setAttribute("data-kgw-analysis-filter-date-text-r23", role);
    unit.appendChild(textInput);
  }

  if (button) {
    button.classList.add("kgw-analysis-filter-date-button-r23");
    button.setAttribute("data-kgw-analysis-filter-date-button-r23", role);
    unit.appendChild(button);
  }

  if (nativeInput) {
    nativeInput.classList.add("kgw-analysis-filter-date-native-r23");
    nativeInput.setAttribute("data-kgw-analysis-filter-date-native-r23", role);
    unit.appendChild(nativeInput);
  }

  return unit;
}

function kgwAnalysisFilterBarAppendLabelR23(bar, text, className) {
  const label = document.createElement("span");
  label.className = `kgw-analysis-filter-label-r23 ${className}`;
  label.textContent = text;
  bar.appendChild(label);
  return label;
}

function kgwAnalysisFilterBarAppendControlR23(bar, node, className, label) {
  if (!node) return null;
  node.classList.add("kgw-analysis-filter-control-r23", className);
  node.setAttribute("data-kgw-analysis-filter-owner-r23", "true");
  if (label) {
    node.setAttribute("aria-label", label);
    node.setAttribute("title", label);
  }
  bar.appendChild(node);
  return node;
}

function kgwAnalysisNormalizeFilterLayoutR18() {
  const owner = root();
  if (!owner) return;

  const range = kgwAnalysisRange();
  const fromInput = range?.fromInput || q("#analysisFromDate");
  const fromNative = range?.fromNative || q("#analysisFromDateNative");
  const toInput = range?.toInput || q("#analysisToDate");
  const toNative = range?.toNative || q("#analysisToDateNative");

  const type = q("#analysisType");
  const direction = q("#analysisDirection");
  const search = q("#analysisSearch");
  const filter = q("#analysisFilter");
  const reset = q("#analysisResetFilter");

  const fromButton = kgwAnalysisFilterBarButtonR23(fromInput, fromNative, "from");
  const toButton = kgwAnalysisFilterBarButtonR23(toInput, toNative, "to");

  kgwAnalysisFilterBarRemoveLabelsR23(type, ["Type"]);
  kgwAnalysisFilterBarRemoveLabelsR23(direction, ["Direction"]);
  kgwAnalysisFilterBarRemoveLabelsR23(search, ["Search"]);

  const controls = [fromInput, fromButton, toInput, toButton, type, direction, search, filter, reset].filter(Boolean);
  let host = kgwAnalysisFilterBarLcaR23(controls);

  if (!host || host === owner || host === document.body || host === document.documentElement) {
    host = filter?.closest(".analysis-filter-row, .kgw-analysis-filter-calendar-r18, .kgw-analysis-filter-bar-owner-r20, .kgw-analysis-date-calendar-owner-r22")
      || search?.closest(".analysis-filter-row, .kgw-analysis-filter-calendar-r18, .kgw-analysis-filter-bar-owner-r20, .kgw-analysis-date-calendar-owner-r22")
      || filter?.parentElement
      || owner;
  }

  if (host === owner) {
    const fallback = filter?.parentElement || search?.parentElement;
    if (fallback) host = fallback;
  }

  const fromUnit = kgwAnalysisFilterBarMakeDateUnitR23("from", fromInput, fromNative, fromButton);
  const toUnit = kgwAnalysisFilterBarMakeDateUnitR23("to", toInput, toNative, toButton);

  const bar = document.createElement("div");
  bar.className = "kgw-analysis-filter-bar-rebuild-r23";
  bar.setAttribute("data-kgw-analysis-filter-bar-owner-r23", "true");

  kgwAnalysisFilterBarAppendLabelR23(bar, "From:", "kgw-analysis-filter-from-label-r23");
  bar.appendChild(fromUnit);
  kgwAnalysisFilterBarAppendLabelR23(bar, "To:", "kgw-analysis-filter-to-label-r23");
  bar.appendChild(toUnit);
  kgwAnalysisFilterBarAppendControlR23(bar, type, "kgw-analysis-filter-type-r23", "Type");
  kgwAnalysisFilterBarAppendControlR23(bar, direction, "kgw-analysis-filter-direction-r23", "Direction");
  kgwAnalysisFilterBarAppendControlR23(bar, search, "kgw-analysis-filter-search-r23", "Search by Address or Transaction");
  kgwAnalysisFilterBarAppendControlR23(bar, filter, "kgw-analysis-filter-apply-r23", "Filter");
  kgwAnalysisFilterBarAppendControlR23(bar, reset, "kgw-analysis-filter-reset-r23", "Reset Filter");

  if (search) search.placeholder = "Search by Address/Transaction...";

  while (host.firstChild) host.removeChild(host.firstChild);

  host.classList.remove(
    "kgw-analysis-filter-calendar-r18",
    "kgw-analysis-filter-bar-owner-r20",
    "kgw-analysis-date-calendar-owner-r22"
  );
  host.classList.add("kgw-analysis-filter-host-r23");
  host.setAttribute("data-kgw-analysis-filter-host-r23", "true");
  host.appendChild(bar);

  kgwAnalysisEnsureTodayToDateR18(false);
}

function kgwAnalysisSetDate(textInput, nativeInput, value) {
  const clean = kgwAnalysisCleanIso(value);
  if (textInput) {
    textInput.value = clean;
    textInput.dispatchEvent(new Event("input", { bubbles: true }));
    textInput.dispatchEvent(new Event("change", { bubbles: true }));
  }
  if (nativeInput) {
    nativeInput.value = clean;
    nativeInput.dispatchEvent(new Event("change", { bubbles: true }));
  }
  return clean;
}

function kgwAnalysisRange() {
  const fromInput = q("#analysisFromDate");
  const toInput = q("#analysisToDate");
  const fromNative = q("#analysisFromDateNative");
  const toNative = q("#analysisToDateNative");
  if (!fromInput || !toInput) return null;
  return { fromInput, toInput, fromNative, toNative };
}

function kgwAnalysisCloseCalendar(scope = "analysis") {
  document.querySelectorAll(`.kgw-calendar-popover[data-kgw-calendar-scope="${scope}"]`).forEach((node) => node.remove());

  const owner = root() || document;
  owner.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    if (!node.dataset.kgwCalendarScope || node.dataset.kgwCalendarScope === scope) {
      delete node.dataset.kgwCalendarOpen;
      delete node.dataset.kgwCalendarScope;
    }
  });
}

function kgwAnalysisApplyPreset(activeTextId, presetName) {
  const today = kgwAnalysisParseIso(kgwAnalysisTodayIso());
  const range = kgwAnalysisRange();

  let from = kgwAnalysisIsoFromDate(today);
  let to = kgwAnalysisIsoFromDate(today);

  if (presetName === "last7") {
    const start = new Date(today);
    start.setDate(start.getDate() - 6);
    from = kgwAnalysisIsoFromDate(start);
  } else if (presetName === "last30") {
    const start = new Date(today);
    start.setDate(start.getDate() - 29);
    from = kgwAnalysisIsoFromDate(start);
  } else if (presetName === "thisMonth") {
    from = kgwAnalysisIsoFromDate(new Date(today.getFullYear(), today.getMonth(), 1));
  } else if (presetName === "sinceLaunch") {
    from = "2021-11-07";
  }

  if (range) {
    kgwAnalysisSetDate(range.fromInput, range.fromNative, from);
    kgwAnalysisSetDate(range.toInput, range.toNative, to);
  } else {
    const activeInput = q(`#${kgwAnalysisEscapeSelector(activeTextId)}`);
    kgwAnalysisSetDate(activeInput, null, presetName === "today" ? to : from);
  }

  renderRows();
  kgwAnalysisCloseCalendar("analysis");
}

function kgwAnalysisAttachPopover(popover, anchor) {
  document.querySelectorAll(".kgw-calendar-popover").forEach((node) => node.remove());
  document.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    delete node.dataset.kgwCalendarOpen;
    delete node.dataset.kgwCalendarScope;
  });

  popover.dataset.kgwCalendarScope = "analysis";
  popover.classList.add("kgw-calendar-popover-analysis");

  document.body.append(popover);

  const rect = anchor.getBoundingClientRect();
  const width = Math.min(236, Math.max(218, window.innerWidth - 24));
  const heightLimit = Math.min(326, window.innerHeight - 24);
  const left = Math.min(Math.max(12, rect.left), Math.max(12, window.innerWidth - width - 12));
  const preferredTop = rect.bottom + 6;
  const top = preferredTop + heightLimit <= window.innerHeight - 12
    ? preferredTop
    : Math.max(12, rect.top - heightLimit - 6);

  popover.style.position = "fixed";
  popover.style.left = `${Math.round(left)}px`;
  popover.style.top = `${Math.round(top)}px`;
  popover.style.width = `${Math.round(width)}px`;
  popover.style.maxHeight = `${Math.round(heightLimit)}px`;
  popover.style.zIndex = "2147483000";
}

/* KGW_CALENDAR_CLOSE_I18N_FIX_R12: i18n-safe calendar close label; no new calendar owner. */
function kgwAnalysisI18nText(key, fallback) {
  const api = globalThis.kgwI18n || globalThis.KGW_I18N || globalThis.i18n || null;
  const candidates = [
    api && typeof api.t === "function" ? api.t.bind(api) : null,
    typeof globalThis.t === "function" ? globalThis.t.bind(globalThis) : null
  ];

  for (const translate of candidates) {
    if (!translate) continue;
    try {
      const value = translate(key);
      if (typeof value === "string" && value.trim() && value !== key) return value;
    } catch {
      /* keep fallback */
    }
  }

  return fallback;
}

function kgwAnalysisOpenCalendar(textInput, nativeInput, textId) {
  const section = root();
  if (!section || !textInput) return;

  const host = textInput.closest(".kgw-analysis-date-field") || textInput.parentElement || section;
  const wasOpen = host.dataset.kgwCalendarOpen === "1";
  kgwAnalysisCloseCalendar("analysis");
  if (wasOpen) return;

  host.dataset.kgwCalendarOpen = "1";
  host.dataset.kgwCalendarScope = "analysis";

  let activeDate = kgwAnalysisParseIso(textInput.value);
  let displayYear = activeDate.getFullYear();
  let displayMonth = activeDate.getMonth();

  const popover = document.createElement("div");
  popover.className = "kgw-calendar-popover";
  popover.lang = "en-US";
  popover.dir = "ltr";
  popover.setAttribute("role", "dialog");
  popover.setAttribute("aria-label", "Date picker");

  function render() {
    popover.textContent = "";

    const header = document.createElement("div");
    header.className = "kgw-calendar-header";

    const prev = document.createElement("button");
    prev.type = "button";
    prev.className = "kgw-calendar-nav";
    prev.textContent = "‹";
    prev.setAttribute("aria-label", "Previous month");

    const title = document.createElement("div");
    title.className = "kgw-calendar-title";
    title.textContent = kgwAnalysisMonthLabel(displayYear, displayMonth);

    const next = document.createElement("button");
    next.type = "button";
    next.className = "kgw-calendar-nav";
    next.textContent = "›";
    next.setAttribute("aria-label", "Next month");

    prev.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-prev-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || ""),
        year: displayYear,
        month: displayMonth
      });
      displayMonth -= 1;
      if (displayMonth < 0) {
        displayMonth = 11;
        displayYear -= 1;
      }
      render();
    });

    next.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-next-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || ""),
        year: displayYear,
        month: displayMonth
      });
      displayMonth += 1;
      if (displayMonth > 11) {
        displayMonth = 0;
        displayYear += 1;
      }
      render();
    });

    header.append(prev, title, next);

    const presets = document.createElement("div");
    presets.className = "kgw-calendar-presets";

    [
      ["today", "Today"],
      ["last7", "Last 7 Days"],
      ["last30", "Last 30 Days"],
      ["thisMonth", "This Month"],
      ["sinceLaunch", "Since Launch"]
    ].forEach(([value, label]) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "kgw-calendar-preset";
      button.textContent = label;
      button.addEventListener("click", (event) => {
        event.preventDefault();
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-preset-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(textId || ""),
          preset: String(value || ""),
          label: String(label || "")
        });
        kgwAnalysisApplyPreset(textId, value);
      });
      presets.append(button);
    });

    const weekdays = document.createElement("div");
    weekdays.className = "kgw-calendar-weekdays";
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].forEach((day) => {
      const node = document.createElement("span");
      node.textContent = day;
      weekdays.append(node);
    });

    const grid = document.createElement("div");
    grid.className = "kgw-calendar-grid";

    const firstDay = new Date(displayYear, displayMonth, 1);
    const daysInMonth = new Date(displayYear, displayMonth + 1, 0).getDate();
    const offset = firstDay.getDay();

    for (let i = 0; i < offset; i++) {
      const empty = document.createElement("span");
      empty.className = "kgw-calendar-empty";
      grid.append(empty);
    }

    const selectedIso = kgwAnalysisCleanIso(textInput.value);
    const todayIso = kgwAnalysisTodayIso();

    for (let day = 1; day <= daysInMonth; day++) {
      const iso = kgwAnalysisIsoFromDate(new Date(displayYear, displayMonth, day));
      const button = document.createElement("button");
      button.type = "button";
      button.className = "kgw-calendar-day";
      button.textContent = String(day);
      button.dataset.iso = iso;

      if (iso === selectedIso) button.classList.add("is-selected");
      if (iso === todayIso) button.classList.add("is-today");

      button.addEventListener("click", (event) => {
        event.preventDefault();
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-day-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(textId || ""),
          iso: String(iso || "")
        });
        kgwAnalysisSetDate(textInput, nativeInput, iso);
        renderRows();
        kgwAnalysisCloseCalendar("analysis");
      });

      grid.append(button);
    }

    const footer = document.createElement("div");
    footer.className = "kgw-calendar-footer";

    const close = document.createElement("button");
    close.type = "button";
    close.className = "kgw-calendar-close";
    close.textContent = kgwAnalysisI18nText("calendar.close", "Close");
    close.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-close-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || "")
      });
      kgwAnalysisCloseCalendar("analysis");
    });

    footer.append(close);
    popover.append(header, presets, weekdays, grid, footer);
  }

  render();
  kgwAnalysisAttachPopover(popover, textInput);
}

function bindCalendar() {
  const pairs = [
    {
      textId: "analysisFromDate",
      nativeId: "analysisFromDateNative",
      role: "from"
    },
    {
      textId: "analysisToDate",
      nativeId: "analysisToDateNative",
      role: "to"
    }
  ];

  function resolveButton(textInput, textId, role, index) {
    const escapedTextId = kgwAnalysisEscapeSelector(textId);
    const host = textInput?.closest(".kgw-analysis-date-field") || textInput?.parentElement || root();

    return q(`[data-date-for="${escapedTextId}"]`)
      || q(`[data-native-for="${kgwAnalysisEscapeSelector(role === "from" ? "analysisFromDateNative" : "analysisToDateNative")}"]`)
      || host?.querySelector(".analysis-calendar-btn")
      || host?.querySelector("button")
      || qa(".analysis-calendar-btn")[index]
      || null;
  }

  function bindPair(pair, index) {
    const textInput = q(`#${kgwAnalysisEscapeSelector(pair.textId)}`);
    const nativeInput = q(`#${kgwAnalysisEscapeSelector(pair.nativeId)}`);
    if (!textInput) return;

    textInput.setAttribute("lang", "en-US");
    textInput.setAttribute("dir", "ltr");

    if (nativeInput) {
      nativeInput.setAttribute("lang", "en-US");
      nativeInput.setAttribute("dir", "ltr");
    }

    const button = resolveButton(textInput, pair.textId, pair.role, index);
    if (button && button.dataset.kgwCalendarBound !== "true") {
      button.dataset.kgwCalendarBound = "true";
      button.dataset.dateFor = pair.textId;
      button.dataset.nativeFor = pair.nativeId;
      button.classList.add("analysis-calendar-btn");
      button.setAttribute("lang", "en-US");
      button.setAttribute("dir", "ltr");
      button.setAttribute("type", "button");

      button.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-button-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          nativeId: String(pair.nativeId || ""),
          role: String(pair.role || "")
        });
        kgwAnalysisSetDate(textInput, nativeInput, textInput.value || kgwAnalysisTodayIso());
        kgwAnalysisOpenCalendar(textInput, nativeInput, pair.textId);
      });
    }

    if (nativeInput && nativeInput.dataset.kgwCalendarNativeBound !== "true") {
      nativeInput.dataset.kgwCalendarNativeBound = "true";
      nativeInput.addEventListener("change", (event) => {
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-native-date-change", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          nativeId: String(pair.nativeId || ""),
          value: String(nativeInput.value || "")
        });
        textInput.value = kgwAnalysisCleanIso(nativeInput.value, textInput.value);
        renderRows();
      });
    }

    if (textInput.dataset.kgwCalendarTextBound !== "true") {
      textInput.dataset.kgwCalendarTextBound = "true";
      textInput.addEventListener("input", (event) => {
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-text-date-input", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          value: String(textInput.value || "")
        });
        textInput.value = kgwAnalysisCleanIso(textInput.value, textInput.value);
      });
      textInput.addEventListener("change", (event) => {
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-text-date-change", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          value: String(textInput.value || "")
        });
        renderRows();
      });
    }
  }

  pairs.forEach(bindPair);
}
/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_ANALYSIS_OWNER_END */


/* Analysis export ownership moved to Rust/WASM in analysis_binding.rs. */


function bindControls() {
  kgwAnalysisPythonBindR9();
  // KGW_ANALYSIS_FILTER_CALENDAR_PATCH_R18
  kgwAnalysisNormalizeFilterLayoutR18();

  const search = q("#analysisSearch");
  const type = q("#analysisType");
  const direction = q("#analysisDirection");
  const filter = q("#analysisFilter");
  const reset = q("#analysisResetFilter");

  [search, type, direction].forEach((node) => {
    if (!node || node.dataset.bound === "true") return;
    node.dataset.bound = "true";

    const eventName = node.tagName === "INPUT" ? "input" : "change";
    node.addEventListener(eventName, (event) => {
      kgwAnalysisUiTraceR50B("analysis-filter", "r50b-analysis-filter-change", {
        trusted: Boolean(event && event.isTrusted),
        element: "node",
        id: String(node.id || ""),
        tag: String(node.tagName || ""),
        eventName: String(eventName || ""),
        value: String(node.value || "")
      });
      renderRows();
    });
  });

  if (filter && filter.dataset.bound !== "true") {
    filter.dataset.bound = "true";
    filter.addEventListener("click", (event) => {
      kgwAnalysisUiTraceR50B("analysis-filter", "r50b-analysis-filter-click", {
        trusted: Boolean(event && event.isTrusted),
        id: String(filter.id || ""),
        text: String(filter.textContent || "").trim()
      });
      renderRows();
    });
  }

  if (reset && reset.dataset.bound !== "true") {
    reset.dataset.bound = "true";
    reset.addEventListener("click", (event) => {
      kgwAnalysisUiTraceR50B("analysis-filter", "r50b-analysis-reset-filter-click", {
        trusted: Boolean(event && event.isTrusted),
        id: String(reset.id || ""),
        text: String(reset.textContent || "").trim()
      });
      if (search) search.value = "";
      if (type) type.value = "ALL";
      if (direction) direction.value = "ALL";
      // KGW_ANALYSIS_FILTER_CALENDAR_PATCH_R18
      kgwAnalysisEnsureTodayToDateR18(true);
      renderRows();
    });
  }
}

function installHook() {
  if (window.__kgwAnalysisRebuildHookInstalled) return;
  window.__kgwAnalysisRebuildHookInstalled = true;

  window.addEventListener("kgw:analysis", (event) => {
    setAnalysisData(event.detail || {});
  });

  window.kgwSetAnalysisData = (payload) => {
    setAnalysisData(payload || {});
  };
}

export async function initAnalysisTab() {
  if (!root()) return;

  bindControls();
  bindCalendar();
  // KGW_ANALYSIS_FILTER_CALENDAR_PATCH_R18
  kgwAnalysisNormalizeFilterLayoutR18();
  kgwAnalysisBindDateTouchR18();
  kgwAnalysisEnsureTodayToDateR18(false);
  installHook();

  if (root().dataset.analysisInitialized !== "true") {
    root().dataset.analysisInitialized = "true";
    setAnalysisData({ rows: [], summary: {} });
    setText("#analysisStatus", "Load an address to see analysis.");
    log().log("analysis rebuilt owner initialized");
  } else {
    renderRows();
  }
}

installRustAnalysisBinding();
