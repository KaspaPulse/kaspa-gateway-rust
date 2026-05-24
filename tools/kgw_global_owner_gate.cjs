#!/usr/bin/env node
"use strict";

// KGW_RUNTIME_REPOSITORY_BINDING_UNIFIED_GATE_R21C
// The runtime repository binding gate is intentionally read-only.
// Canonical command:
//   node tools/kgw_runtime_repository_binding_gate.cjs --strict --online --json
// Compatibility wrappers:
//   node tools/kgw_runtime_repository_binding_audit.cjs
//   powershell -NoProfile -ExecutionPolicy Bypass -File tools/kgw_runtime_repository_binding_audit.ps1

/*
 * KGW_CANONICAL_GLOBAL_OWNER_GATE_R3C_REGISTRY_REFINEMENT
 *
 * Canonical KGW owner registry and conflict gate.
 *
 * This gate is the source of truth for KGW ownership boundaries.
 * It separates:
 * - active owner source files
 * - reference/wiring files
 * - audit/tool files
 *
 * Default audit mode returns 0 and prints/report JSON.
 * Strict mode returns non-zero when active-source owner conflicts remain.
 *
 * Usage:
 *   node tools/kgw_global_owner_gate.cjs
 *   node tools/kgw_global_owner_gate.cjs --json
 *   node tools/kgw_global_owner_gate.cjs --strict
 *   node tools/kgw_global_owner_gate.cjs --owner bridgeInstances --strict
 *   node tools/kgw_global_owner_gate.cjs --changed-files apps/.../file.js --strict
 */

const fs = require("fs");
const path = require("path");

const REPO_ROOT = path.resolve(__dirname, "..");

const OWNER_REGISTRY = {
  settingsButtons: {
    ownerId: "KGW_SETTINGS_OWNER",
    description: "Node/Bridge settings Save/Restore/Set defaults button ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_SETTINGS_OWNER_V19",
      "KGW_SETTINGS_OWNER_V19_SAFE_FEEDBACK_NO_FREEZE_V25B"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "action=settings-buttons",
      "nativeDisabledExpected",
      "click-received",
      "click-ignored-disabled",
      "action-start",
      "auto-baseline-before-input",
      "holdMs:5000",
      "holdMs:10000",
      "input-locked",
      "change-locked"
    ],
    responsibilities: [
      "settings dirty state",
      "settings action button enable/disable",
      "settings button visual feedback",
      "no duplicate settings button owner"
    ]
  },

  bridgeInstances: {
    ownerId: "KGW_BRIDGE_INSTANCES_OWNER",
    description: "Bridge Instances UI, serializer, defaults, and port conflict validation.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_BRIDGE_INSTANCE_PHASE1_UPSTREAM_SERIALIZER_R1C",
      "KGW_BRIDGE_INSTANCES_UI_PORT_VALIDATOR_R5",
      "KGW_BRIDGE_INSTANCES_FIELDS_TRASH_R6"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_BRIDGE_INSTANCE_PATCH",
      "KGW_BRIDGE_INSTANCE_OWNER_FIX",
      "KGW_INSTANCE_RUNTIME_PATCH",
      "KGW_INSTANCE_UI_PATCH",
      "--instance-log-to-file",
      "--instance-var-diff",
      "--instance-shares-per-min",
      "--instance-var-diff-stats",
      "--instance-pow2-clamp"
    ],
    responsibilities: [
      "bridge instance tab UI",
      "upstream-compatible --instance serializer",
      "instance default false values",
      "instance explicit port/diff/prom fields",
      "instance port conflict validation"
    ]
  },

  calendar: {
    ownerId: "KGW_CALENDAR_OWNER",
    description: "Explorer/Analysis date picker lifecycle, single active popover, and close i18n.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/main.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.css",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.css",
      "apps/kaspa-gateway-desktop/frontend/i18n/ar.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/de.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/en.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/es.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/fr.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/hi.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/id.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/ja.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/ko.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/ru.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/tr.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/zh-CN.json"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_CALENDAR_SINGLE_ACTIVE_POPOVER_FIX_R8",
      "KGW_CALENDAR_TAB_LIFECYCLE_CLEANUP_R11B",
      "KGW_CALENDAR_CLOSE_I18N_FIX_R12"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_CALENDAR_SINGLE_OWNER_FIX_V1",
      "KGW_CALENDAR_UX_RELIABILITY_PRESETS_V2",
      "KGW_CALENDAR_UX_RELIABILITY_PRESETS_V2B",
      "KGW_CALENDAR_NATIVE_INPUT_CLICK_OWNER_CSS_V1",
      "KGW_RUNTIME_DATE_DOM_TRACE_V1",
      "KGW_RUNTIME_DATE_DOM_TRACE_BUTTON_CHANNEL_V2",
      "KGW_EXPLORER_DATE_PICKER_LABELS_V1",
      "KGW_ANALYSIS_DATE_PICKER_COMPACT_FILTERS_V1"
    ],
    responsibilities: [
      "date picker lifecycle",
      "single active calendar popover",
      "close calendars on tab switch",
      "calendar i18n labels"
    ]
  },

  exports: {
    ownerId: "KGW_EXPORT_OWNER",
    description: "Explorer/Analysis/Top Addresses CSV/HTML/PDF export ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/export_commands.rs",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.export.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js"
    ],
    referenceFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/explorer_services.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/export_system.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/real_reports.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/ui_wiring.rs"
    ],
    requiredMarkers: [
      "export_report",
      "ExportClientTable",
      "write_pdf",
      "write_html",
      "write_csv"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "URL.createObjectURL",
      "download =",
      "Phase 2 pending"
    ],
    responsibilities: [
      "backend export generation",
      "native save as routing",
      "csv/html/pdf output",
      "no duplicate blob download export system"
    ]
  },

  runtime: {
    ownerId: "KGW_RUNTIME_OWNER",
    description: "Runtime mapping, raw logs, repository bindings, and self-worker startup.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
      "crates/kaspa-gateway-rk-bridge/src/lib.rs",
      "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs",
      "config/runtime-repository-bindings.json"
    ],
    referenceFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/main.rs",
      "tools/kgw_bridge_node_mode_routing_audit_v1.cjs",
      "tools/kgw_runtime_trace_owner_audit_v20.cjs",
      "tools/kgw_parallel_self_worker_runtime_gate.cjs"
    ],
    requiredMarkers: [
      "kgw_apply_command_preview_overrides",
      "start_mainline_bridge_owner_thread",
      "start_tn12_bridge_owner_thread",
      "try_run_kgw_self_worker_from_args"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "Live refresh active; bridge is still running",
      "Waiting for new runtime stdout/stderr lines"
    ],
    responsibilities: [
      "runtime arg mapping",
      "node/bridge self-worker startup",
      "raw stdout/stderr log flow",
      "repository binding",
      "no fake runtime log injection"
    ]
  },

  i18n: {
    ownerId: "KGW_I18N_OWNER",
    description: "Frontend i18n API, contract gate, locale coverage, and translation safety.",
    activeFiles: [
      "tools/kgw_i18n_contract_gate.cjs",
      "tools/kgw_i18n_locale_coverage_gate.cjs",
      "apps/kaspa-gateway-desktop/frontend/main.js",
      "apps/kaspa-gateway-desktop/frontend/i18n/ar.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/de.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/en.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/es.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/fr.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/hi.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/id.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/ja.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/ko.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/ru.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/tr.json",
      "apps/kaspa-gateway-desktop/frontend/i18n/zh-CN.json"
    ],
    referenceFiles: [],
    requiredMarkers: [],
    requiredFiles: [
      "tools/kgw_i18n_contract_gate.cjs",
      "tools/kgw_i18n_locale_coverage_gate.cjs"
    ],
    forbiddenMarkers: [
      "textContent = \"Close\"",
      "textContent = 'Close'"
    ],
    responsibilities: [
      "translation keys",
      "locale coverage",
      "dynamic literal blocking",
      "runtime i18n API"
    ]
  },

  // KGW_GLOBAL_OWNER_GATE_TRACE_REGISTRY_R52B
  // KGW_EXPLORER_TRACE_GATE_REGISTRY_EXPLICIT_FIX_R53B6
  // KGW_GLOBAL_OWNER_GATE_TRACE_REGISTRY_REFINEMENT_R52C4
  mainTabTrace: {
    ownerId: "KGW_MAIN_TAB_TRACE_OWNER",
    description: "Main shell tab navigation trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/main.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_EXPLICIT_MAIN_TAB_TRACE_PATCH_R35C"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_RUNTIME_DATE_DOM_TRACE_V1",
      "KGW_RUNTIME_DATE_DOM_TRACE_BUTTON_CHANNEL_V2"
    ],
    responsibilities: [
      "main tab click trace",
      "main tab open trace",
      "active hash trace",
      "no duplicate main tab trace owner"
    ]
  },

  bridgeOwnedNodeDisplayOnly: {
    ownerId: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_OWNER",
    description: "Node display-only ownership while Bridge in-process owns a same-network node runtime.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E",
      "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_MAINNET_IMMEDIATE_R65F",
      "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_HYDRATION_R65H2"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65D"
    ],
    responsibilities: [
      "bridge in-process start sets same-network node display-only lock",
      "bridge stop clears same-network node display-only lock",
      "node start/stop/settings actions are blocked while bridge owns the node runtime",
      "network tabs remain available for read-only inspection",
      "no CSS hiding workaround and no global listener"
    ]
  },
  nodeBridgeInternalNavigationTrace: {
    ownerId: "KGW_NODE_BRIDGE_INTERNAL_NAV_TRACE_OWNER",
    description: "Node/Bridge internal network and inner-tab navigation trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
      "KGW_NODE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45F",
      "KGW_BRIDGE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45E"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "kgwBridgeExplicitTraceR27D is not defined",
      "kgwNodeExplicitTraceR27D is not defined",
      "document.addEventListener('click'"
    ],
    responsibilities: [
      "node network tab trace",
      "bridge network tab trace",
      "node inner tab trace",
      "bridge inner tab trace",
      "network select trace",
      "no global document capture"
    ]
  },

  nodeBridgeSettingsChangeTrace: {
    ownerId: "KGW_NODE_BRIDGE_SETTINGS_CHANGE_TRACE_OWNER",
    description: "Node/Bridge settings change trace inside the existing V19 settings owner.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_SETTINGS_OWNER_V19",
      "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_SETTINGS_UNIFIED_OWNER_R12",
      "initial-load-r10",
      "suppressed-by-r9b",
      "legacy-dirty suppressed-by-r9b"
    ],
    responsibilities: [
      "trusted settings input trace",
      "trusted settings change trace",
      "scoped network update trace",
      "keep V19 as the single settings-button owner"
    ]
  },

  nodeBridgeLogControlsTrace: {
    ownerId: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_OWNER",
    description: "Node/Bridge Log Auto-scroll, font A-/A+/Reset, Copy Log, Clear Log trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B2"
    ],
    responsibilities: [
      "node log autoscroll trace",
      "node log font size trace",
      "node copy/clear log trace",
      "bridge log autoscroll trace",
      "bridge log font size trace",
      "bridge copy/clear log trace"
    ]
  },

  settingsUiTrace: {
    ownerId: "KGW_SETTINGS_UI_TRACE_OWNER",
    description: "Settings tab UI movement trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_SETTINGS_UI_TRACE_PATCH_R48B3"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_SETTINGS_UI_TRACE_PATCH_R48B2"
    ],
    responsibilities: [
      "settings outer tab trace",
      "settings inner tab trace",
      "settings select-all master trace",
      "settings select-all child trace",
      "settings input/change trace",
      "settings address/database/log diagnostics trace"
    ]
  },

  logTabTrace: {
    ownerId: "KGW_LOG_TAB_TRACE_OWNER",
    description: "Standalone Log tab trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/log/log.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_LOG_UI_TRACE_PATCH_R49B2"
    ],
    requiredFiles: [],
    forbiddenMarkers: [],
    responsibilities: [
      "log severity filter trace",
      "log search trace",
      "log auto-scroll trace",
      "log font-size trace",
      "log copy/clear trace"
    ]
  },

  topAddressesTrace: {
    ownerId: "KGW_TOP_ADDRESSES_TRACE_OWNER",
    description: "Top Addresses safe controls trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49D"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49B",
      "KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49C"
    ],
    responsibilities: [
      "top addresses refresh trace",
      "top addresses filter/reset trace",
      "top addresses csv/html/pdf export trace",
      "do not patch kgwExportCenteredOpenPromptV10 unless separately audited"
    ]
  },


  analysisResultsTable: {
    ownerId: "KGW_ANALYSIS_RESULTS_TABLE_OWNER",
    description: "Analysis results and Counterparty Breakdown table rendering, row formatting, sorting surface, and layout ownership.",
    activeFiles: [
          "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.js",
          "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.css",
          "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js",
          "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.html",
          "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.template.js"
    ],
    referenceFiles: [
          "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs",
          "apps/kaspa-gateway-desktop/src-tauri/src/export_commands.rs",
          "apps/kaspa-gateway-desktop/frontend/i18n/ar.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/de.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/en.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/es.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/fr.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/hi.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/id.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/ja.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/ko.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/ru.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/tr.json",
          "apps/kaspa-gateway-desktop/frontend/i18n/zh-CN.json"
    ],
    requiredMarkers: [
          "function setAnalysisData(payload = {})",
          "function setAnalysisData",
          "analysisRows = rows.map",
          "kgwAnalysisSetExportButtonsEnabledV1G",
          "renderRows();"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "analysisResultsTableDuplicateOwner",
      "KGW_ANALYSIS_RESULTS_TABLE_LAYER",
      "KGW_ANALYSIS_RESULTS_TABLE_OVERLAY_WORKAROUND",
      "KGW_ANALYSIS_RESULTS_TABLE_MUTATION_OBSERVER_LAYER"
    ],
    responsibilities: [
      "analysis Counterparty Breakdown table render owner",
      "analysis results row formatting",
      "analysis address and transaction display surface",
      "analysis result table sorting and display behavior",
      "analysis export compatibility awareness",
      "no duplicate Analysis results table owner",
      "no overlay workaround and no MutationObserver workaround for this table"
    ]
  },
  analysisUiTrace: {
    ownerId: "KGW_ANALYSIS_UI_TRACE_OWNER",
    description: "Analysis filters, calendar, export, start/cancel trace ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_ANALYSIS_SAFE_CONTROLS_TRACE_PATCH_R50B",
      "KGW_ANALYSIS_START_CANCEL_TRACE_PATCH_R50D3"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_ANALYSIS_START_CANCEL_TRACE_PATCH_R50D2"
    ],
    responsibilities: [
      "analysis filters trace",
      "analysis calendar trace",
      "analysis export trace",
      "analysis start/cancel trace",
      "analysis address input/select trace",
      "do not patch kgwExportCenteredOpenPromptV10 unless separately audited"
    ]
  },

  explorerTrace: {
    ownerId: "KGW_EXPLORER_TRACE_OWNER",
    description: "Explorer UI movement trace ownership. R53B3 safe controls trace is the active Explorer owner.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.export.js"
    ],
    referenceFiles: [],
    requiredMarkers: [
      "KGW_EXPLORER_SAFE_CONTROLS_TRACE_PATCH_R53B3"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "KGW_RUNTIME_DATE_DOM_TRACE_V1",
      "KGW_RUNTIME_DATE_DOM_TRACE_BUTTON_CHANNEL_V2"
    ],
    responsibilities: [
      "explorer fetch trace",
      "explorer force fetch trace",
      "explorer filters trace",
      "explorer calendar/date trace",
      "explorer export trace",
      "R53B3 patched exact safe Explorer owners"
    ]
  },


  // KGW_COMMAND_COMPOSER_GLOBAL_OWNER_GATE_REGISTRY_R2B_SAFE_GATE
  nodeCommandComposer: {
    ownerId: "KGW_NODE_COMMAND_COMPOSER_OWNER",
    description: "Node tab command composer, option include/exclude model, command preview serializer, scoped defaults, and node command UI ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"
    ],
    referenceFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
      "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs",
      "config/runtime-repository-bindings.json"
    ],
    requiredMarkers: [
      "KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7",
      "KGW_NODE_COMMAND_COMPOSER_CHECKBOX_ONLY_R9",
      "KGW_NODE_COMMAND_COMPOSER_CHECKBOX_ONLY_CSS_R9"
    ],
    requiredFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"
    ],
    forbiddenMarkers: [
      "KGW_NODE_COMMAND_COMPOSER_DUPLICATE_OWNER",
      "KGW_NODE_COMMAND_COMPOSER_OVERLAY_WORKAROUND",
      "KGW_NODE_COMMAND_COMPOSER_MUTATION_OBSERVER_LAYER",
      "KGW_NODE_COMMAND_COMPOSER_DOCUMENT_CAPTURE_LAYER"
    ],
    responsibilities: [
      "node command preview ownership",
      "node command option include/exclude state",
      "node option serialization into one command model",
      "node scoped default IP/port values",
      "node validated select-based option values where applicable",
      "no duplicate node command builder",
      "no overlay workaround and no MutationObserver workaround"
    ]
  },

  bridgeCommandComposer: {
    ownerId: "KGW_BRIDGE_COMMAND_COMPOSER_OWNER",
    description: "Bridge tab command composer, option include/exclude model, command preview serializer, IP/default/difficulty option ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"
    ],
    referenceFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
      "crates/kaspa-gateway-rk-bridge/src/lib.rs",
      "config/runtime-repository-bindings.json"
    ],
    requiredMarkers: [
      "KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7",
      "KGW_BRIDGE_COMMAND_COMPOSER_CHECKBOX_ONLY_R9",
      "KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B",
      "KGW_BRIDGE_INPROCESS_SERIALIZATION_CHECKBOX_R13B",
      "KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B",
      "KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B",
      "KGW_BRIDGE_INSTANCE_UPSTREAM_SERIALIZATION_CHECKBOX_R13B",
      "KGW_BRIDGE_INSTANCE_WHOLE_ARG_CHECKBOX_R13B",
      "KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_ACTION_R13B",
      "KGW_BRIDGE_INPROCESS_INSTANCES_COMMAND_CHECKBOX_CSS_R13B",
      "KGW_BRIDGE_DIFFICULTY_DATALIST_R16C",
      "KGW_BRIDGE_DIFFICULTY_DATALIST_CSS_R16C"
    ],
    requiredFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"
    ],
    forbiddenMarkers: [
      "KGW_BRIDGE_COMMAND_COMPOSER_DUPLICATE_OWNER",
      "KGW_BRIDGE_COMMAND_COMPOSER_OVERLAY_WORKAROUND",
      "KGW_BRIDGE_COMMAND_COMPOSER_MUTATION_OBSERVER_LAYER",
      "KGW_BRIDGE_COMMAND_COMPOSER_DOCUMENT_CAPTURE_LAYER"
    ],
    responsibilities: [
      "bridge command preview ownership",
      "bridge command option include/exclude state",
      "bridge option serialization into one command model",
      "bridge scoped default IP/port values",
      "bridge validated difficulty/share select values",
      "bridge var-diff and shares-per-min command option ownership",
      "no duplicate bridge command builder",
      "no overlay workaround and no MutationObserver workaround"
    ]
  },

  commandComposerRuntimeMapping: {
    ownerId: "KGW_COMMAND_COMPOSER_RUNTIME_MAPPING_OWNER",
    description: "Runtime-side command argument mapping for Node/Bridge command composer output.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
      "crates/kaspa-gateway-rk-bridge/src/lib.rs",
      "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs"
    ],
    referenceFiles: [
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "config/runtime-repository-bindings.json"
    ],
    requiredMarkers: [],
    requiredFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"
    ],
    forbiddenMarkers: [
      "KGW_COMMAND_COMPOSER_RUNTIME_DUPLICATE_MAPPING",
      "KGW_COMMAND_COMPOSER_FAKE_COMMAND_LAYER",
      "KGW_COMMAND_COMPOSER_PARALLEL_ARG_BUILDER"
    ],
    responsibilities: [
      "runtime command argument mapping",
      "node/bridge command payload compatibility",
      "keep frontend preview and runtime args consistent",
      "no parallel runtime argument mapper",
      "no fake command preview/runtime divergence"
    ]
  },

  traceBackendGate: {
    ownerId: "KGW_TRACE_BACKEND_GATE_OWNER",
    description: "Rust backend trace gate and dev-only file sink ownership.",
    activeFiles: [
      "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"
    ],
    referenceFiles: [
      "apps/kaspa-gateway-desktop/frontend/main.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/log/log.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.export.js",
      "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js"
    ],
    requiredMarkers: [
      "kgw_frontend_button_trace_v1",
      "KGW_UI_TRACE"
    ],
    requiredFiles: [],
    forbiddenMarkers: [
      "src/core/kgw-global-dev-trace.js"
    ],
    responsibilities: [
      "KGW_UI_TRACE console gate",
      "KGW_UI_TRACE_FILE file gate",
      "KGW_UI_TRACE_DIR path gate",
      "no frontend invoke proxy",
      "no global trace layer"
    ]
  }
};

const SCAN_ROOTS = [
  "tools",
  "apps/kaspa-gateway-desktop/frontend",
  "apps/kaspa-gateway-desktop/src-tauri/src",
  "crates/kaspa-gateway-rk-bridge/src",
  "crates/kaspa-gateway-rk-node/src",
  "config"
];

const SCAN_EXTS = [".js", ".css", ".rs", ".json", ".toml", ".md", ".html", ".cjs"];

const OWNER_AUDIT_TOOL_FILES = new Set([
  "tools/kgw_global_owner_gate.cjs",
  "tools/kgw_runtime_trace_owner_audit_v20.cjs",
  "tools/kgw_i18n_contract_gate.cjs",
  "tools/kgw_i18n_locale_coverage_gate.cjs",
  "tools/kgw_parallel_self_worker_runtime_gate.cjs",
  "tools/kgw_bridge_node_mode_routing_audit_v1.cjs"
]);

function parseArgs(argv) {
  const args = {
    strict: false,
    owner: null,
    changedFiles: null,
    json: false,
    help: false
  };

  for (let i = 0; i < argv.length; i++) {
    const value = argv[i];

    if (value === "--strict") args.strict = true;
    else if (value === "--json") args.json = true;
    else if (value === "--help" || value === "-h") args.help = true;
    else if (value === "--owner") args.owner = argv[++i] || null;
    else if (value.startsWith("--owner=")) args.owner = value.slice("--owner=".length);
    else if (value === "--changed-files") {
      args.changedFiles = [];
      while (argv[i + 1] && !argv[i + 1].startsWith("--")) {
        args.changedFiles.push(normalizeRel(argv[++i]));
      }
    } else if (value.startsWith("--changed-files=")) {
      args.changedFiles = value
        .slice("--changed-files=".length)
        .split(",")
        .map(normalizeRel)
        .filter(Boolean);
    }
  }

  return args;
}

function normalizeRel(value) {
  return String(value || "").replaceAll("\\", "/").replace(/^\.\//, "").trim();
}

function exists(rel) {
  return fs.existsSync(path.join(REPO_ROOT, normalizeRel(rel)));
}

function read(rel) {
  return fs.readFileSync(path.join(REPO_ROOT, normalizeRel(rel)), "utf8");
}

function isListedFile(rel, list) {
  const file = normalizeRel(rel);
  return (list || []).some((entry) => {
    const normalized = normalizeRel(entry);
    return file === normalized || file.startsWith(normalized + "/");
  });
}

function isActiveFile(rel, owner) {
  return isListedFile(rel, owner.activeFiles);
}

function isReferenceFile(rel, owner) {
  return isListedFile(rel, owner.referenceFiles);
}

function isOwnerToolFile(rel) {
  return OWNER_AUDIT_TOOL_FILES.has(normalizeRel(rel));
}

function isOwnerEvidenceFile(rel, owner) {
  return isActiveFile(rel, owner) || isReferenceFile(rel, owner);
}

function walk(rootRel) {
  const root = path.join(REPO_ROOT, rootRel);
  const out = [];
  const skip = new Set([".git", "node_modules", "target", "dist", "build", ".vite", "_TEMP"]);

  function visit(dir) {
    let entries = [];

    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }

    for (const ent of entries) {
      const full = path.join(dir, ent.name);
      const rel = normalizeRel(path.relative(REPO_ROOT, full));

      if (ent.isDirectory()) {
        if (skip.has(ent.name)) continue;
        if (rel.includes("/target/") || rel.includes("/node_modules/") || rel.includes("/tools/_TEMP/")) continue;
        visit(full);
      } else if (ent.isFile()) {
        if (SCAN_EXTS.some((ext) => ent.name.endsWith(ext))) out.push(rel);
      }
    }
  }

  if (fs.existsSync(root)) visit(root);
  return out;
}

function collectFiles(args) {
  if (args.changedFiles && args.changedFiles.length) {
    return [...new Set(args.changedFiles.map(normalizeRel))].filter(exists);
  }

  return [...new Set(SCAN_ROOTS.flatMap(walk))].sort();
}

function lineAt(text, index) {
  return text.slice(0, index).split(/\r?\n/).length;
}

function linesAround(text, line, radius = 4) {
  const lines = text.split(/\r?\n/);
  const start = Math.max(0, line - radius - 1);
  const end = Math.min(lines.length, line + radius);

  return lines.slice(start, end).map((value, offset) => ({
    line: start + offset + 1,
    text: value
  }));
}

function literalHits(text, term) {
  const hits = [];
  let pos = 0;

  while (true) {
    const index = text.indexOf(term, pos);
    if (index < 0) break;

    const line = lineAt(text, index);
    hits.push({
      term,
      line,
      context: linesAround(text, line)
    });

    pos = index + Math.max(1, term.length);
  }

  return hits;
}

function scanOwner(ownerName, owner, files, args) {
  const result = {
    ownerName,
    ownerId: owner.ownerId,
    requiredMissing: [],
    requiredFilesMissing: [],
    requiredFound: [],
    referenceEvidence: [],
    forbiddenHits: [],
    markerOutsideOwnerFiles: [],
    activeEvidenceFiles: [],
    referenceEvidenceFiles: [],
    responsibilities: owner.responsibilities
  };

  const evidenceText = [];

  for (const file of files) {
    if (!exists(file)) continue;

    const text = read(file);
    const active = isActiveFile(file, owner);
    const reference = isReferenceFile(file, owner);
    const tool = isOwnerToolFile(file);
    const evidence = active || reference;

    if (evidence) evidenceText.push(text);

    for (const marker of owner.requiredMarkers || []) {
      if (!text.includes(marker)) continue;

      if (active) {
        result.requiredFound.push({ marker, file, kind: "active" });
        if (!result.activeEvidenceFiles.includes(file)) result.activeEvidenceFiles.push(file);
      } else if (reference) {
        result.requiredFound.push({ marker, file, kind: "reference" });
        result.referenceEvidence.push({ marker, file });
        if (!result.referenceEvidenceFiles.includes(file)) result.referenceEvidenceFiles.push(file);
      } else if (!tool) {
        result.markerOutsideOwnerFiles.push({ marker, file });
      }
    }

    if (tool || reference) continue;

    for (const marker of owner.forbiddenMarkers || []) {
      for (const hit of literalHits(text, marker)) {
        result.forbiddenHits.push({
          marker,
          file,
          line: hit.line,
          context: hit.context
        });
      }
    }
  }

  if (!args.changedFiles || !args.changedFiles.length) {
    const combined = evidenceText.join("\n");

    for (const marker of owner.requiredMarkers || []) {
      if (!combined.includes(marker)) {
        result.requiredMissing.push(marker);
      }
    }

    for (const file of owner.requiredFiles || []) {
      if (!exists(file)) {
        result.requiredFilesMissing.push(file);
      }
    }
  }

  return result;
}

function runGate(args) {
  const selectedOwners = args.owner
    ? Object.entries(OWNER_REGISTRY).filter(([name]) => name === args.owner)
    : Object.entries(OWNER_REGISTRY);

  if (args.owner && selectedOwners.length === 0) {
    return {
      ok: false,
      fatal: true,
      errors: [{ type: "unknown-owner", owner: args.owner }],
      ownerResults: [],
      summary: {}
    };
  }

  const files = collectFiles(args);
  const ownerResults = selectedOwners.map(([name, owner]) => scanOwner(name, owner, files, args));
  const errors = [];

  for (const result of ownerResults) {
    for (const marker of result.requiredMissing) {
      errors.push({
        type: "required-marker-missing",
        ownerName: result.ownerName,
        ownerId: result.ownerId,
        marker
      });
    }

    for (const file of result.requiredFilesMissing) {
      errors.push({
        type: "required-file-missing",
        ownerName: result.ownerName,
        ownerId: result.ownerId,
        file
      });
    }

    for (const hit of result.markerOutsideOwnerFiles) {
      errors.push({
        type: "marker-outside-owner-files",
        ownerName: result.ownerName,
        ownerId: result.ownerId,
        marker: hit.marker,
        file: hit.file
      });
    }

    for (const hit of result.forbiddenHits) {
      errors.push({
        type: "forbidden-marker",
        ownerName: result.ownerName,
        ownerId: result.ownerId,
        marker: hit.marker,
        file: hit.file,
        line: hit.line
      });
    }
  }

  return {
    ok: errors.length === 0,
    fatal: false,
    filesScanned: files.length,
    strict: args.strict,
    ownerFilter: args.owner,
    changedFiles: args.changedFiles,
    errors,
    ownerResults,
    summary: {
      ownersChecked: ownerResults.length,
      errors: errors.length,
      requiredMissing: errors.filter((e) => e.type === "required-marker-missing").length,
      requiredFilesMissing: errors.filter((e) => e.type === "required-file-missing").length,
      markersOutsideOwnerFiles: errors.filter((e) => e.type === "marker-outside-owner-files").length,
      forbiddenMarkers: errors.filter((e) => e.type === "forbidden-marker").length
    }
  };
}

function printHelp() {
  console.log("KGW Canonical Global Owner Gate");
  console.log("");
  console.log("Usage:");
  console.log("  node tools/kgw_global_owner_gate.cjs");
  console.log("  node tools/kgw_global_owner_gate.cjs --json");
  console.log("  node tools/kgw_global_owner_gate.cjs --strict");
  console.log("  node tools/kgw_global_owner_gate.cjs --owner bridgeInstances --strict");
  console.log("  node tools/kgw_global_owner_gate.cjs --changed-files <file...> --strict");
  console.log("");
  console.log("Owners:");
  for (const [name, owner] of Object.entries(OWNER_REGISTRY)) {
    console.log("  " + name + " => " + owner.ownerId);
  }
}

function printHuman(result) {
  console.log("KGW canonical global owner gate");
  console.log("marker: KGW_CANONICAL_GLOBAL_OWNER_GATE_R3C_REGISTRY_REFINEMENT");
  console.log("filesScanned:", result.filesScanned || 0);
  console.log("ownersChecked:", result.summary?.ownersChecked || 0);
  console.log("errors:", result.summary?.errors || 0);
  console.log("");

  for (const owner of result.ownerResults || []) {
    console.log("[" + owner.ownerName + "] " + owner.ownerId);
    console.log("  requiredFound:", owner.requiredFound.length);
    console.log("  requiredMissing:", owner.requiredMissing.length ? owner.requiredMissing.join(", ") : "none");
    console.log("  requiredFilesMissing:", owner.requiredFilesMissing.length ? owner.requiredFilesMissing.join(", ") : "none");
    console.log("  markerOutsideOwnerFiles:", owner.markerOutsideOwnerFiles.length);
    console.log("  forbiddenHits:", owner.forbiddenHits.length);
    console.log("  activeEvidenceFiles:", owner.activeEvidenceFiles.length ? owner.activeEvidenceFiles.join(", ") : "none");
    console.log("  referenceEvidenceFiles:", owner.referenceEvidenceFiles.length ? owner.referenceEvidenceFiles.join(", ") : "none");
  }

  if (result.errors && result.errors.length) {
    console.log("");
    console.log("Owner gate errors:");
    for (const error of result.errors.slice(0, 100)) {
      console.log("  - " + JSON.stringify(error));
    }

    if (result.errors.length > 100) {
      console.log("  ... " + (result.errors.length - 100) + " more");
    }
  }

  console.log("");
  console.log(result.ok ? "KGW_OWNER_GATE_PASS" : "KGW_OWNER_GATE_CONFLICTS_FOUND");
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  if (args.help) {
    printHelp();
    return;
  }

  const result = runGate(args);

  if (args.json) {
    console.log(JSON.stringify(result, null, 2));
  } else {
    printHuman(result);
  }

  if (args.strict && !result.ok) {
    process.exitCode = 1;
  }
}

main();

module.exports = {
  OWNER_REGISTRY,
  runGate
};

