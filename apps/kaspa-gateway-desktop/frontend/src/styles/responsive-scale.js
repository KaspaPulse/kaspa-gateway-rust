export function initResponsiveScale() {
  const root = document.documentElement;

  function applyScale() {
    const width = Math.max(960, window.innerWidth || 960);
    const scale = Math.max(0.72, Math.min(1, width / 1600));
    root.style.setProperty("--kgw-scale", String(scale));
  }

  applyScale();
  window.addEventListener("resize", applyScale);
}
