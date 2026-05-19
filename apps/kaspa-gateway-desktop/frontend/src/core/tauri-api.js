export async function tauriInvoke(command, args = {}) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_INVOKE__;

  if (!invoke) {
    throw new Error("Tauri invoke is not available.");
  }

  return await invoke(command, args);
}
