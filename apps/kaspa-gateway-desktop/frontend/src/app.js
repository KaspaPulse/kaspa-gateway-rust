/*
  Disabled intentionally.

  The desktop shell is owned by frontend/main.js + src/tabs/tab-registry.js.
  Keeping this file as an auto-initializer causes duplicate tab initialization
  when imported accidentally.
*/

export function initModularFrontend() {
  return Promise.resolve({
    ready: false,
    disabled: true,
    owner: "main.js"
  });
}
