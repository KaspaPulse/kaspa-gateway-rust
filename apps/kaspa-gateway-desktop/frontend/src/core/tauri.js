export async function invoke(command, args = {}) {
  const tauriInvoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI__?.invoke;

  if (!tauriInvoke) {
    throw new Error("Tauri invoke is not available.");
  }

  return await tauriInvoke(command, args);
}
