export function byId(id) {
  return document.getElementById(id);
}

export function query(selector, root = document) {
  return root.querySelector(selector);
}

export function queryAll(selector, root = document) {
  return Array.from(root.querySelectorAll(selector));
}

export function bindOnce(element, eventName, handler, key) {
  if (!element) return false;

  const flag = key || `bound_${eventName}`;

  if (element.dataset[flag] === "true") {
    return false;
  }

  element.dataset[flag] = "true";
  element.addEventListener(eventName, handler);
  return true;
}

export function escapeHtml(value) {
  const div = document.createElement("div");
  div.textContent = value === null || typeof value === "undefined" ? "" : String(value);
  return div.innerHTML;
}
