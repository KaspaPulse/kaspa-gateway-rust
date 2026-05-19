#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const repo = process.cwd();
const frontendRoot = path.join(repo, "apps", "kaspa-gateway-desktop", "frontend");
const i18nDir = path.join(frontendRoot, "i18n");

function read(file) { return fs.readFileSync(file, "utf8"); }
function normalizeText(value) { return String(value || "").replace(/\s+/g, " ").trim(); }

function flattenDictionary(source, prefix = "", out = {}) {
  if (!source || typeof source !== "object" || Array.isArray(source)) {
    if (prefix) out[prefix] = source;
    return out;
  }

  for (const [key, value] of Object.entries(source)) {
    if (key.includes(".")) {
      if (value && typeof value === "object" && !Array.isArray(value)) flattenDictionary(value, key, out);
      else out[key] = value;
      continue;
    }

    const next = prefix ? prefix + "." + key : key;

    if (value && typeof value === "object" && !Array.isArray(value)) flattenDictionary(value, next, out);
    else out[next] = value;
  }

  return out;
}

const criticalKeys = [
  "settings.language",
  "settings.currency",
  "settings.theme",
  "tabs.explorer",
  "tabs.analysis",
  "tabs.log",
  "tabs.settings",
  "status.ready",
  "runtime.ready",
  "common.loading",
  "metrics.price",
  "metrics.hashrate",
  "metrics.difficulty",
  "common.force.fetch",
  "actions.fetch",
  "actions.cancel",
  "actions.export.results",
  "ui.analysis.reset.filter",
  "ui.analysis.save.as.csv",
  "ui.analysis.save.as.html",
  "ui.analysis.save.as.pdf",
  "ui.explorer.select.saved.address",
  "ui.explorer.balance",
  "ui.explorer.date.time.sort",
  "ui.explorer.transaction.id.sort",
  "ui.explorer.direction.sort",
  "ui.explorer.amount.kas.sort",
  "ui.explorer.value.usd.sort",
  "ui.explorer.type.sort",
  "ui.explorer.transaction.table.font.size",
  "explorer.noTransactionsToDisplay",
  "links.donations"
];

const approvedSameAsEnglish = [
  {
    "lang": "de",
    "key": "tabs.explorer",
    "value": "Explorer",
    "reason": "Accepted German UI loanword; translating it artificially would be worse UX."
  }
];

function isApprovedSameAsEnglish(lang, key, value) {
  const normalized = normalizeText(value);

  return approvedSameAsEnglish.some((item) => {
    return item.lang === lang &&
      item.key === key &&
      normalizeText(item.value) === normalized;
  });
}

const languageFiles = fs.readdirSync(i18nDir)
  .filter((name) => name.endsWith(".json"))
  .filter((name) => !["map.json", "lang_map.json"].includes(name));

const dictionaries = {};

for (const name of languageFiles) {
  const lang = name.replace(/\.json$/, "");
  dictionaries[lang] = flattenDictionary(JSON.parse(read(path.join(i18nDir, name))));
}

const missing = [];
const sameAsEnglish = [];
const approved = [];

for (const key of criticalKeys) {
  const en = dictionaries.en && dictionaries.en[key];

  for (const [lang, dict] of Object.entries(dictionaries)) {
    const value = dict[key];

    if (typeof value !== "string" || !value.trim()) {
      missing.push({ lang, key });
      continue;
    }

    if (lang !== "en" && typeof en === "string" && normalizeText(value) === normalizeText(en)) {
      if (isApprovedSameAsEnglish(lang, key, value)) {
        approved.push({ lang, key, value });
      } else {
        sameAsEnglish.push({ lang, key, value });
      }
    }
  }
}

console.log("KGW i18n locale coverage gate");
console.log("criticalKeys:", criticalKeys.length);
console.log("missing:", missing.length);
console.log("sameAsEnglish:", sameAsEnglish.length);
console.log("approvedSameAsEnglish:", approved.length);

if (missing.length || sameAsEnglish.length) {
  console.error(JSON.stringify({ missing, sameAsEnglish, approvedSameAsEnglish: approved }, null, 2));
  process.exit(1);
}
