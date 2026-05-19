export const tabRegistry = new Map();

export function registerTab(id, controller) {
  if (!id || !controller) {
    throw new Error("registerTab requires id and controller.");
  }

  tabRegistry.set(id, controller);
}

export function initRegisteredTabs() {
  for (const [id, controller] of tabRegistry.entries()) {
    if (typeof controller.init === "function") {
      controller.init();
    }
  }
}

export function getTabController(id) {
  return tabRegistry.get(id) || null;
}
