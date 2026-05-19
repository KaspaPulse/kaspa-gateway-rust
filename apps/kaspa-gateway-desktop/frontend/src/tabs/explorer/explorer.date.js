/*
 * KGW_EXPLORER_DATE_INPUT_UTILITIES
 *
 * Behavior-preserving extraction from explorer.js.
 * This module must stay dependency-light:
 * - no DOM ownership
 * - no Tauri invoke ownership
 * - no transaction fetch/render orchestration
 */

import { toEnglishDigits } from "./explorer.utils.js";
export function normalizeDateInputValue(value) {
  const clean = toEnglishDigits(value).replace(/[^\d-]/g, "");
  const compact = clean.replace(/-/g, "");

  if (/^\d{8}$/.test(compact)) {
    return `${compact.slice(0, 4)}-${compact.slice(4, 6)}-${compact.slice(6, 8)}`;
  }

  return clean.slice(0, 10);
}

export function parseDateSeconds(dateText, endOfDay = false) {
  const value = normalizeDateInputValue(dateText);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;

  const suffix = endOfDay ? "T23:59:59" : "T00:00:00";
  const ms = new Date(`${value}${suffix}`).getTime();

  return Number.isFinite(ms) ? Math.floor(ms / 1000) : null;
}

export function kgwDayToEpochSeconds(day, endOfDay = false) {
  const date = new Date(`${day}T00:00:00Z`);
  const base = Math.floor(date.getTime() / 1000);

  if (!Number.isFinite(base)) return null;

  return endOfDay ? base + 86399 : base;
}

export function kgwTxDayToEpochSeconds(day, endOfDay = false) {
  const date = new Date(`${day}T00:00:00Z`);
  const base = Math.floor(date.getTime() / 1000);

  if (!Number.isFinite(base)) {
    return null;
  }

  return endOfDay ? base + 86399 : base;
}

export function kgwClean2DayToSeconds(day, endOfDay = false) {
  const date = new Date(`${day}T00:00:00Z`);
  const base = Math.floor(date.getTime() / 1000);
  if (!Number.isFinite(base)) return null;
  return endOfDay ? base + 86399 : base;
}

export function kgwTransactionDateKey(row) {
  const raw =
    row?.date ||
    row?.day ||
    row?.datetime ||
    row?.timestamp ||
    row?.time ||
    "";

  const text = String(raw || "");

  const match = text.match(/\d{4}-\d{2}-\d{2}/);
  if (match) return match[0];

  return "Unknown Date";
}
