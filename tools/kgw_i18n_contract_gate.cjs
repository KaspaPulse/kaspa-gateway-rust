#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const repo = process.cwd();
const frontendRoot = path.join(repo, "apps", "kaspa-gateway-desktop", "frontend");
const i18nDir = path.join(frontendRoot, "i18n");
const mainPath = path.join(frontendRoot, "main.js");

function read(file) { return fs.readFileSync(file, "utf8"); }
function normalizeText(value) { return String(value || "").replace(/\s+/g, " ").trim(); }
function listFiles(dir, predicate, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) listFiles(full, predicate, out);
    else if (predicate(full)) out.push(full);
  }
  return out;
}
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
function stripBlocks(html) {
  return html
    .replace(/<script\b[\s\S]*?<\/script>/gi, "")
    .replace(/<style\b[\s\S]*?<\/style>/gi, "")
    .replace(/<svg\b[\s\S]*?<\/svg>/gi, "")
    .replace(/<template\b[\s\S]*?<\/template>/gi, "");
}
function isStructuralHtmlFragment(value) {
  const text = normalizeText(value);
  if (!text) return true;
  if (/^<\s*(div|span|pre|tr|td|th|option|tbody|thead|table)\b/i.test(text)) return true;
  if (/^<\s*\/\s*(div|span|pre|tr|td|th|option|tbody|thead|table)\s*>/i.test(text)) return true;
  if (/^<[^>]+=$/.test(text)) return true;
  if (/^<[^>]+\s*$/.test(text)) return true;
  return false;
}
function isLikelyUserText(value) {
  const text = normalizeText(value);
  if (!text || isStructuralHtmlFragment(text)) return false;
  if (text.length > 220) return false;
  if (["N/A", "—", "×", "i"].includes(text)) return false;
  if (/^[0-9.,:%()\-+]+$/.test(text)) return false;
  if (/^\{\{.*\}\}$/.test(text)) return false;
  if (/^https?:\/\//i.test(text)) return false;
  if (text.includes("${")) return false;
  if (text.includes("=>")) return false;
  if (text.includes("querySelector")) return false;
  if (text.includes("addEventListener")) return false;
  if (text.includes("pointer-events:")) return false;
  if (text.includes("z-index:")) return false;
  if (text.includes("window.kgwT")) return false;
  return /[A-Za-z\u0600-\u06FF]/.test(text);
}
function extractHtmlRefs(html) {
  const refs = [];
  const clean = stripBlocks(html);
  const regex = /\b(data-i18n|data-i18n-title|data-i18n-placeholder|data-i18n-aria-label)=["']([^"']+)["']/g;
  let match;
  while ((match = regex.exec(clean))) {
    const key = normalizeText(match[2]);
    if (key) refs.push(key);
  }
  return refs;
}
function extractJsRefs(js) {
  const refs = [];
  const patterns = [
    /\bkgwT\s*\(\s*["']([^"']+)["']\s*\)/g,
    /\bkgwI18n\s*\(\s*["']([^"']+)["']\s*\)/g,
    /\bt\s*\(\s*["']([^"']+)["']\s*\)/g,
    /\bdataset\.i18n\s*=\s*["']([^"']+)["']/g,
    /\bsetAttribute\(\s*["']data-i18n["']\s*,\s*["']([^"']+)["']\s*\)/g,
    /\bdata-i18n=["']([^"']+)["']/g,
    /\bdata-i18n=\\"([^"\\]+)\\"/g
  ];
  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(js))) {
      const key = normalizeText(match[1]);
      if (key && !key.includes(" ")) refs.push(key);
    }
  }
  return refs;
}
function extractUnboundHtmlText(html, rel) {
  const clean = stripBlocks(html);
  const findings = [];
  const regex = /<([a-zA-Z][a-zA-Z0-9:-]*)([^>]*)>\s*([^<>{}]*[A-Za-z\u0600-\u06FF][^<>{}]*)\s*<\/\1>/g;
  let match;
  while ((match = regex.exec(clean))) {
    const tag = match[1].toLowerCase();
    const attrs = match[2] || "";
    const text = normalizeText(match[3]);
    if (!isLikelyUserText(text)) continue;
    if (["script", "style", "svg", "template"].includes(tag)) continue;
    if (attrs.includes("data-i18n")) continue;
    if (attrs.includes("data-kgw-no-i18n")) continue;
    findings.push({ file: rel, tag, text });
  }
  return findings;
}
function extractDynamicLiteralFindings(js, rel) {
  const findings = [];
  const patterns = [
    { kind: "textContent", regex: /\.(?:textContent|innerText)\s*=\s*["'`]([^"'`]*[A-Za-z\u0600-\u06FF][^"'`]*)["'`]/g },
    { kind: "innerHTML", regex: /\.innerHTML\s*=\s*["'`]([^"'`]*[A-Za-z\u0600-\u06FF][^"'`]*)["'`]/g },
    { kind: "insertAdjacentHTML", regex: /\.insertAdjacentHTML\s*\(\s*["'`][^"'`]*["'`]\s*,\s*["'`]([^"'`]*[A-Za-z\u0600-\u06FF][^"'`]*)["'`]\s*\)/g },
    { kind: "newOption", regex: /new\s+Option\s*\(\s*["'`]([^"'`]*[A-Za-z\u0600-\u06FF][^"'`]*)["'`]/g }
  ];
  for (const pattern of patterns) {
    let match;
    while ((match = pattern.regex.exec(js))) {
      const literal = normalizeText(match[1]);
      if (!isLikelyUserText(literal)) continue;
      if (literal.includes("data-i18n=")) continue;
      findings.push({ file: rel, kind: pattern.kind, literal });
    }
  }
  return findings;
}
function extractQuoteRisks(js, rel) {
  const findings = [];
  const lines = js.split(/\r?\n/);
  lines.forEach((line, index) => {
    if (!line.includes('data-i18n="')) return;
    if (line.includes("`")) return;
    const dataIndex = line.indexOf('data-i18n="');
    const before = line.slice(0, dataIndex);
    const quoteCountBefore = (before.match(/"/g) || []).length;
    if (quoteCountBefore % 2 === 1) findings.push({ file: rel, line: index + 1, text: line.trim() });
  });
  return findings;
}

if (!fs.existsSync(frontendRoot)) {
  console.error("Missing frontend root:", frontendRoot);
  process.exit(1);
}
if (!fs.existsSync(i18nDir)) {
  console.error("Missing i18n dir:", i18nDir);
  process.exit(1);
}

const languageFiles = fs.readdirSync(i18nDir).filter((name) => name.endsWith(".json")).filter((name) => !["map.json", "lang_map.json"].includes(name));
const dictionaries = {};
for (const name of languageFiles) {
  const lang = name.replace(/\.json$/, "");
  dictionaries[lang] = JSON.parse(read(path.join(i18nDir, name)));
}

const htmlFiles = listFiles(frontendRoot, (file) => file.endsWith(".html"));
const jsFiles = listFiles(frontendRoot, (file) => file.endsWith(".js"));

const refs = new Set();
const unboundHtmlText = [];
const dynamicLiterals = [];
const quoteRisks = [];

for (const htmlFile of htmlFiles) {
  const rel = path.relative(frontendRoot, htmlFile).replace(/\\/g, "/");
  const html = read(htmlFile);
  for (const ref of extractHtmlRefs(html)) refs.add(ref);
  for (const finding of extractUnboundHtmlText(html, rel)) unboundHtmlText.push(finding);
}
for (const jsFile of jsFiles) {
  const rel = path.relative(frontendRoot, jsFile).replace(/\\/g, "/");
  const js = read(jsFile);
  for (const ref of extractJsRefs(js)) refs.add(ref);
  for (const finding of extractDynamicLiteralFindings(js, rel)) dynamicLiterals.push(finding);
  for (const finding of extractQuoteRisks(js, rel)) quoteRisks.push(finding);
}

const missingRefs = [];
for (const [lang, dict] of Object.entries(dictionaries)) {
  const flat = flattenDictionary(dict);
  for (const key of Array.from(refs).sort()) {
    if (typeof flat[key] !== "string" || !flat[key].trim()) missingRefs.push({ lang, key });
  }
}

const mainJs = read(mainPath);
const runtimeFindings = [];
const requiredMarkers = [
  "KGW_R99_CANONICAL_I18N_BIND_APPLY_HELPER",
  "KGW_R100_FLATTEN_I18N_DICTIONARY",
  "KGW_R102_DYNAMIC_DOM_I18N_REAPPLY",
  "KGW_R107_CANONICAL_TRANSLATION_RUNTIME_API",
  "window.kgwT = function kgwTranslateRuntimeR107",
  "window.kgwI18n = window.kgwT",
  "window.__kgwI18nDictR107 = dict;",
  "kgw:tab-opened",
  "tab-opened-after-mount",
  "flattenKgwI18nDictionaryR100",
  "bindMissingI18nAttributesR99",
  "updateDynamicKgwI18nRuntimeR102"
];

for (const marker of requiredMarkers) {
  if (!mainJs.includes(marker)) runtimeFindings.push("missing-main-marker:" + marker);
}

console.log("KGW i18n contract gate");
console.log("refs:", refs.size);
console.log("missingRefs:", missingRefs.length);
console.log("unboundHtmlText:", unboundHtmlText.length);
console.log("dynamicLiterals:", dynamicLiterals.length);
console.log("quoteRisks:", quoteRisks.length);
console.log("runtimeFindings:", runtimeFindings.length);

const blockers = [];
if (missingRefs.length > 0) blockers.push("missingRefs=" + missingRefs.length);
if (unboundHtmlText.length > 0) blockers.push("unboundHtmlText=" + unboundHtmlText.length);
if (dynamicLiterals.length > 0) blockers.push("dynamicLiterals=" + dynamicLiterals.length);
if (quoteRisks.length > 0) blockers.push("quoteRisks=" + quoteRisks.length);
if (runtimeFindings.length > 0) blockers.push("runtimeFindings=" + runtimeFindings.length);

if (blockers.length > 0) {
  console.error(JSON.stringify({ missingRefs, unboundHtmlText, dynamicLiterals, quoteRisks, runtimeFindings }, null, 2));
  console.error("i18n contract gate failed:", blockers.join(", "));
  process.exit(1);
}
