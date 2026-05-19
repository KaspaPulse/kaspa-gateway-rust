/*
 * KGW_EXPLORER_PURE_UTILITIES
 *
 * Behavior-preserving extraction from explorer.js.
 * This module must stay dependency-light:
 * - no DOM ownership
 * - no Tauri invoke ownership
 * - no transaction fetch/render orchestration
 */
export function toEnglishDigits(value) {
  return String(value ?? "")
    .replace(/[٠-٩]/g, (d) => "٠١٢٣٤٥٦٧٨٩".indexOf(d))
    .replace(/[۰-۹]/g, (d) => "۰۱۲۳۴۵۶۷۸۹".indexOf(d));
}

export function pick(...values) {
    for (const value of values) {
      if (value !== null && value !== undefined && value !== "") return value;
    }
    return null;
  }

export function toNumber(value, fallback = 0) {
    const n = Number(value);
    return Number.isFinite(n) ? n : fallback;
  }

export function kgwClean2SafeText(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}
