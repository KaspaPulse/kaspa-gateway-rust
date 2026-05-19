/*
 * KGW_EXPLORER_BALANCE_PRICE_FORMATTING
 *
 * Behavior-preserving extraction from explorer.js.
 * This module must stay dependency-light:
 * - no DOM ownership
 * - no Tauri invoke ownership
 * - no transaction fetch/render orchestration
 */
export function formatKas(value) {

  const number = Number(value || 0);

  if (!Number.isFinite(number) || number === 0) {
    return "";
  }

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  });
}

export function formatUsd(value) {

  const number = Number(value || 0);

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  });
}


export function kgwSummaryFormatKas(value) {

  const number = Number(value || 0);

  if (!Number.isFinite(number) || number === 0) {
    return "";
  }

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  });
}

export function kgwSummaryFormatUsd(value) {

  const number = Number(value || 0);

  if (!Number.isFinite(number) || number <= 0) {
    return "";
  }

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  });
}

export function kgwClean2Kas(value) {

  const number = Number(value || 0);

  if (!Number.isFinite(number) || number === 0) {
    return "";
  }

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  });
}

export function kgwClean2Usd(value) {

  const number = Number(value || 0);

  if (!Number.isFinite(number) || number <= 0) {
    return "";
  }

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 3,
    maximumFractionDigits: 3
  });
}
