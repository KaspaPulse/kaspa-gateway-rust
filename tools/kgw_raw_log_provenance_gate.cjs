#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repo = process.cwd();
const files = {
  integrated: "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
  lib: "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
  nodeOwner: "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs",
  bridgeOwner: "crates/kaspa-gateway-rk-bridge/src/lib.rs",
  smoke: "apps/kaspa-gateway-desktop/src-tauri/src/bin/kgw-provenance-smoke.rs",
  nodeFrontend: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  bridgeFrontend: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
};

function fail(message) {
  console.error("KGW raw log provenance gate FAILED");
  console.error(message);
  process.exit(1);
}

function read(relative) {
  const filename = path.join(repo, relative);
  if (!fs.existsSync(filename)) fail("Missing file: " + relative);
  return fs.readFileSync(filename, "utf8");
}

function count(source, literal) {
  return source.split(literal).length - 1;
}

function functionBody(source, name) {
  const rustMarker = `fn ${name}`;
  const jsMarker = `function ${name}`;
  const rustStart = source.indexOf(rustMarker);
  const jsStart = source.indexOf(jsMarker);
  const start = rustStart >= 0 ? rustStart : jsStart;
  if (start < 0) fail("Missing function: " + name);
  const open = source.indexOf("{", start);
  if (open < 0) fail("Missing function body: " + name);

  let depth = 0;
  let quote = "";
  let lineComment = false;
  let blockComment = false;
  for (let index = open; index < source.length; index += 1) {
    const char = source[index];
    const next = source[index + 1] || "";

    if (lineComment) {
      if (char === "\n") lineComment = false;
      continue;
    }
    if (blockComment) {
      if (char === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (char === "\\") {
        index += 1;
        continue;
      }
      if (char === quote) quote = "";
      continue;
    }
    if (char === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "{") depth += 1;
    if (char === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(start, index + 1);
    }
  }
  fail("Unterminated function: " + name);
}

const integrated = read(files.integrated);
const lib = read(files.lib);
const nodeOwner = read(files.nodeOwner);
const bridgeOwner = read(files.bridgeOwner);
const smoke = read(files.smoke);
const nodeFrontend = read(files.nodeFrontend);
const bridgeFrontend = read(files.bridgeFrontend);

const checks = [];
function check(name, ok) {
  checks.push([name, Boolean(ok)]);
}

check("one production raw entry construction call", count(integrated, "kgw_worker_raw_log_entry_v1(") === 2);
check("one production raw buffer insertion call", count(integrated, "kgw_worker_push_raw_log(") === 2);
check("one underlying raw push_back", count(integrated, "guard.push_back(entry);") === 1);
check("raw construction is inside child pipe reader", functionBody(integrated, "kgw_worker_spawn_reader").includes("kgw_worker_raw_log_entry_v1"));
check("raw insertion is inside child pipe reader", functionBody(integrated, "kgw_worker_spawn_reader").includes("kgw_worker_push_raw_log"));
check("raw constructor accepts no Bridge listener attribution", !functionBody(integrated, "kgw_worker_raw_log_entry_v1").includes("_bridge_instance_id"));
check("stdout and stderr are OS pipes", count(integrated, ".stdout(Stdio::piped())") >= 1 && count(integrated, ".stderr(Stdio::piped())") >= 1);
check("raw buffer is registered immediately before spawn", /kgw_worker_replace_raw_log_buffer_v1\(&role, &network\)\?;\s*let mut child = match command\.spawn\(\)/.test(integrated));
check("test raw capture helper is cfg(test) scoped", /#\[cfg\(test\)\]\s*#\[allow\(dead_code\)\][^]*?fn kgw_capture_raw_pipe_for_test_v1/.test(integrated));
check("official sentinel test is wired into the true raw log gate", /official_sentinel_stdout_and_stderr_use_the_production_pipe_reader_unchanged/.test(read("tools/kgw_true_raw_log_gate.ps1")));
check("official sentinel test is wired into the full local gate", /official_sentinel_stdout_and_stderr_use_the_production_pipe_reader_unchanged/.test(read("tools/kgw_full_local_gate.ps1")));

const emitterPattern = /(?:println|eprintln|print|eprint)!|(?:tracing|log)::(?:trace|debug|info|warn|error)!|\b(?:trace|debug|info|warn|error)!/;
for (const name of [
  "try_run_kgw_self_worker_from_args",
  "kgw_run_node_self_worker",
  "kgw_run_bridge_self_worker",
]) {
  check(`no KGW emitter in production self-worker function ${name}`, !emitterPattern.test(functionBody(lib, name)));
}
const bridgeWriterPlumbing = functionBody(lib, "kgw_init_bridge_self_worker_raw_tracing_r23");
check("Bridge writer plumbing emits no KGW record", !emitterPattern.test(bridgeWriterPlumbing) && bridgeWriterPlumbing.includes("with_writer(std::io::stderr)"));
check("no KGW node owner log emitter", !/(?:println|eprintln|print|eprint)!|(?:tracing|log)::(?:trace|debug|info|warn|error)!/.test(nodeOwner.split("#[cfg(test)]\nmod tests")[0]));
check("no KGW Bridge owner log emitter", !/(?:println|eprintln|print|eprint)!|(?:tracing|log)::(?:trace|debug|info|warn|error)!/.test(bridgeOwner.split("#[cfg(test)]\nmod tests")[0]));
check("test fixture emitters are cfg(test) scoped", integrated.includes("#[cfg(test)]\n#[test]\nfn kgw_test_self_worker_hold") && integrated.includes("#[cfg(test)]\n#[test]\nfn kgw_test_self_worker_fail") && integrated.includes("#[cfg(test)]\n#[test]\nfn kgw_test_self_worker_delayed_fail"));

check("Node frontend has no raw content blacklist", !nodeFrontend.includes("RawLogTextHasTransportWrapper") && !functionBody(nodeFrontend, "kgwNodeNormalizeRawLogEntryV1").includes("rawTextValue).includes"));
check("Bridge frontend has no raw content blacklist", !bridgeFrontend.includes("RawLogTextHasTransportWrapper") && !functionBody(bridgeFrontend, "kgwBridgeNormalizeRawLogEntryV1").includes("rawTextValue).includes"));
check("Node legacy appendLog is inert", !/(textContent|records\.|\.push\(|\.set\()/.test(functionBody(nodeFrontend, "appendLog")));
check("Bridge legacy appendLog is inert", !/(textContent|records\.|\.push\(|\.set\()/.test(functionBody(bridgeFrontend, "appendLog")));
check("Bridge raw buffer key is process-level", functionBody(bridgeFrontend, "kgwBridgeRawLogBufferKeyV1").includes("void instanceId") && !functionBody(bridgeFrontend, "kgwBridgeRawLogBufferKeyV1").includes("String(instanceId"));
check("Bridge raw normalizer ignores UI listener selection", functionBody(bridgeFrontend, "kgwBridgeNormalizeRawLogEntryV1").includes("void expectedInstanceId"));
check("startup control is a typed file side channel", lib.includes("KgwStartupControlMessageV1") && lib.includes("kgw_write_startup_control_v1") && !functionBody(lib, "try_run_kgw_self_worker_from_args").includes("println!"));
check("live smoke parent output is emitted after child shutdown", smoke.indexOf("let stop_result = kgw_kgw_disable_network_v1") < smoke.lastIndexOf("println!"));

const failed = checks.filter(([, ok]) => !ok);
if (failed.length > 0) {
  fail(failed.map(([name]) => "- " + name).join("\n"));
}

console.log("KGW raw log provenance gate");
for (const [name] of checks) console.log("PASS " + name);
console.log("rawProducersUnknown: 0");
console.log("productionTextFiltering: false");
console.log("status: PASS");
