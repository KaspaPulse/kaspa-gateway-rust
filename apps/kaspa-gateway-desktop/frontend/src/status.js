// Presentation only. Stable state keys remain independent of translated labels.
const GROUPS = Object.freeze({
  positive: ["running", "active", "enabled", "healthy", "ok", "success", "hashing", "synchronized", "synced"],
  negative: ["stopped", "disabled", "error", "failed", "offline", "unavailable", "disconnected", "not connected", "not ready"],
  ready: ["ready", "available", "connected", "verified", "validated", "complete", "completed"],
  warning: ["warning", "warn", "degraded", "partial", "starting", "stopping", "syncing", "not synchronized", "not synced", "pending", "loading", "busy", "fetching", "validating", "reconciling", "waiting for work", "no recent hashing reported"],
  neutral: ["idle", "unknown", "neutral", "info", "empty", "cancelled", "not reported", "not checked", "synchronization not reported"],
});
const TONES = new Map(Object.entries(GROUPS).flatMap(([tone, states]) =>
  states.map(state => [state, tone])));
export function statusTone(state) {
  return TONES.get(String(state || "").trim().toLowerCase().replace(/[_-]+/g, " ").replace(/\s+/g, " ")) || "neutral";
}
export function applyStatusTone(element, state) {
  if (!element) return;
  // A previous translated state must not overwrite a new runtime/action message.
  const key = element.dataset.i18n;
  if (key && typeof window.kgwT === "function" && element.textContent !== window.kgwT(key)) {
    element.removeAttribute("data-i18n");
  }
  element.dataset.statusTone = statusTone(state);
}
// Summary values are text nodes, never HTML from a runtime/API response.
export function renderStatusSummary(element, text) {
  if (!element) return;
  if (element.dataset.statusSummary === text && element.querySelector(".kgw-status-value")) return;
  const content = document.createDocumentFragment();
  for (const [index, part] of String(text).split(" | ").entries()) {
    if (index) content.append(document.createTextNode(" | "));
    const split = part.indexOf(": ");
    const label = split < 0 ? "" : part.slice(0, split + 2);
    const value = split < 0 ? part : part.slice(split + 2);
    const item = document.createElement("span");
    item.className = "kgw-status-item";
    item.append(document.createTextNode(label));
    const badge = document.createElement("span");
    badge.className = "kgw-status-value";
    badge.textContent = value;
    applyStatusTone(badge, label === "RPC error: " ? "error" : value);
    item.append(badge);
    content.append(item);
  }
  element.replaceChildren(content);
  element.dataset.statusSummary = text;
}
