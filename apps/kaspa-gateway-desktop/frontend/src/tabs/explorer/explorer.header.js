/*
 * KGW_EXPLORER_HEADER_HELPERS
 *
 * Behavior-preserving extraction from explorer.formatting.js.
 * This module owns small header-derived UI values such as USD price parsing.
 * It may read DOM/header state, but must not own transaction fetch/render orchestration.
 */

export function parseHeaderUsdPrice() {
  const direct = [
    window.__kgwKaspaUsdPrice,
    window.__kgwHeaderPriceUsd,
    window.__kgwLastKasPriceUsd,
    window.__kaspaPriceUsd,
    window.kaspaPriceUsd,
    document.documentElement?.dataset?.kgwKaspaUsdPrice
  ];

  for (const item of direct) {
    const number = Number(String(item || "").replace(/,/g, ""));

    if (Number.isFinite(number) && number > 0) {
      return number;
    }
  }

  const headerPrice = document.getElementById("kgwHeaderPrice");
  const headerText = String(headerPrice?.textContent || "").replace(/,/g, "");
  const headerMatch = headerText.match(/([0-9]+(?:\.[0-9]+)?)/);
  const fromHeader = Number(headerMatch?.[1]);

  if (Number.isFinite(fromHeader) && fromHeader > 0) {
    return fromHeader;
  }

  const text = document.body?.innerText || "";
  const match = text.replace(/,/g, "").match(/(?:Price\s*)?([0-9]+(?:\.[0-9]+)?)\s*USD/i);
  const value = Number(match?.[1]);

  return Number.isFinite(value) && value > 0 ? value : 0;
}
