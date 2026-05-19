import explorerHtml from "./explorer/explorer.template.js";
import kaspaNodeHtml from "./kaspa-node/kaspa-node.template.js";
import kaspaBridgeHtml from "./kaspa-bridge/kaspa-bridge.template.js";
import analysisHtml from "./analysis/analysis.template.js";
import topAddressesHtml from "./top-addresses/top-addresses.template.js";
import logHtml from "./log/log.template.js";
import settingsHtml from "./settings/settings.template.js";

export const KGW_TABS = [
  {
    id: "explorer",
    css: "./src/tabs/explorer/explorer.css",
    html: explorerHtml,
    module: () => import("./explorer/explorer.js"),
    init: "initExplorerTab"
  },
  {
    id: "kaspa-node",
    css: "./src/tabs/kaspa-node/kaspa-node.css",
    html: kaspaNodeHtml,
    module: () => import("./kaspa-node/kaspa-node.js"),
    init: "initKaspaNodeTab"
  },
  {
    id: "kaspa-bridge",
    css: "./src/tabs/kaspa-bridge/kaspa-bridge.css",
    html: kaspaBridgeHtml,
    module: () => import("./kaspa-bridge/kaspa-bridge.js"),
    init: "initKaspaBridgeTab"
  },
  {
    id: "analysis",
    css: "./src/tabs/analysis/analysis.css",
    html: analysisHtml,
    module: () => import("./analysis/analysis.js"),
    init: "initAnalysisTab"
  },
  {
    id: "top-addresses",
    css: "./src/tabs/top-addresses/top-addresses.css",
    html: topAddressesHtml,
    module: () => import("./top-addresses/top-addresses.js"),
    init: "initTopAddressesTab"
  },
  {
    id: "log",
    css: "./src/tabs/log/log.css",
    html: logHtml,
    module: () => import("./log/log.js"),
    init: "initLogTab"
  },
  {
    id: "settings",
    css: "./src/tabs/settings/settings.css",
    html: settingsHtml,
    module: () => import("./settings/settings.js"),
    init: "initSettingsTab"
  }
];
