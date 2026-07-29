import assert from "node:assert/strict";
import crypto from "node:crypto";

export const rawLogRejectionMarkers = Object.freeze([
  "kgw_raw_process_log_v1",
  "[KGW_CHILD_STDOUT]",
  "[KGW_CHILD_STDERR]",
  "diagnostic_transport_record",
  ";source=self-worker;",
  ";runtime_role=",
  ";received_ms="
]);

export function sha256Hex(text) {
  return crypto.createHash("sha256").update(String(text ?? ""), "utf8").digest("hex");
}

export function normalizeClipboardText(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").replace(/\n/g, "\r\n");
}

export function lineCount(text) {
  const value = String(text ?? "");
  return value.length === 0 ? 0 : value.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n").length;
}

export function containsTransportWrapper(text) {
  const value = String(text ?? "");
  if (!value) return false;
  const lower = value.toLowerCase();
  if (rawLogRejectionMarkers.some((marker) => lower.includes(marker.toLowerCase()))) return true;
  return value
    .split(/\r?\n/)
    .some((line) => {
      const trimmed = line.trimStart();
      return trimmed.startsWith("{") &&
        /"stage"\s*:/.test(trimmed) &&
        /"network"\s*:/.test(trimmed) &&
        (/"source"\s*:/.test(trimmed) || /"eventKind"\s*:\s*"diagnostic_transport_record"/.test(trimmed));
    });
}

export function parseKeyValueLine(text) {
  const fields = {};
  for (const part of String(text ?? "").split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;
    fields[part.slice(0, index).trim()] = part.slice(index + 1).trim();
  }
  return fields;
}

export function pidFromStatus(statusText) {
  const fields = parseKeyValueLine(statusText);
  const pid = Number(fields.pid);
  return Number.isSafeInteger(pid) && pid > 0 ? pid : null;
}

export function assertNoTransportWrappers(text, label) {
  assert.equal(containsTransportWrapper(text), false, `${label} contains diagnostic transport wrapper text`);
}

export function assertDirectRawPayload({ text, label, mustMatch = [], mustNotMatch = [] }) {
  assert.ok(String(text ?? "").trim().length > 0, `${label} raw payload is empty`);
  assertNoTransportWrappers(text, label);
  for (const pattern of mustMatch) {
    assert.match(text, pattern, `${label} did not match ${pattern}`);
  }
  for (const pattern of mustNotMatch) {
    assert.doesNotMatch(text, pattern, `${label} unexpectedly matched ${pattern}`);
  }
}

export function assertRuntimeLogReport(report, { network, role, bridgeInstanceId }) {
  assert.equal(report?.version, "kgw_runtime_logs_v1", "runtime log report version");
  assert.ok(Array.isArray(report.entries), "runtime log report entries must be an array");
  assert.ok(report.entries.length > 0, `expected raw ${role} entries for ${network}`);
  for (const entry of report.entries) {
    assert.equal(String(entry.network || "").toLowerCase(), network, "entry network");
    assert.equal(String(entry.runtimeRole || entry.runtime_role || "").toLowerCase(), role, "entry runtime role");
    assert.ok(["stdout", "stderr"].includes(String(entry.stream || "").toLowerCase()), "entry stream");
    if (bridgeInstanceId) {
      assert.equal(String(entry.bridgeInstanceId ?? entry.bridge_instance_id ?? ""), String(bridgeInstanceId), "bridge instance");
    }
    assertNoTransportWrappers(entry.rawText ?? entry.raw_text ?? "", "runtime rawText");
  }
}

