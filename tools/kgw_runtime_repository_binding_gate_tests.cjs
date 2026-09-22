#!/usr/bin/env node
"use strict";
const fs = require("node:fs"), os = require("node:os"), path = require("node:path"), cp = require("node:child_process"), assert = require("node:assert/strict");
const root = process.cwd(), fixture = fs.mkdtempSync(path.join(os.tmpdir(), "kgw-binding-gate-"));
const gate = path.join(root, "tools/kgw_runtime_repository_binding_gate.cjs");
const files = ["config/runtime-repository-bindings.json", "crates/kaspa-gateway-rk-node/Cargo.toml", "crates/kaspa-gateway-rk-bridge/Cargo.toml", "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs", "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs", "crates/kaspa-gateway-rk-bridge/src/lib.rs", "Cargo.lock"];
function baseline() { for (const rel of files) { const p = path.join(fixture, rel); fs.mkdirSync(path.dirname(p), {recursive:true}); fs.copyFileSync(path.join(root,rel), p); } }
function run() { const p=cp.spawnSync(process.execPath,[gate,"--offline","--strict","--json"],{cwd:fixture,encoding:"utf8"}); return {status:p.status,result:JSON.parse(p.stdout)}; }
function mutate(rel, change) { const p=path.join(fixture,rel); fs.writeFileSync(p,change(fs.readFileSync(p,"utf8"))); }
function rejected(name, change, issue) { baseline(); change(); const r=run(); assert.notEqual(r.status,0,name); assert.ok(r.result.findings.some(x=>x.issue===issue),name+": "+JSON.stringify(r.result.findings)); console.log("PASS "+name); }
try {
 baseline(); assert.equal(run().status,0,"actual source/lock must align");
 console.log("PASS aligned official source");
 rejected("RPC alias revision drift",()=>mutate(files[1],s=>s.replace(/(kaspa-grpc-client-tn13 = .*rev = ")[a-f0-9]{40}/,(_,prefix)=>prefix+"0".repeat(40))),"cargo-rev-mismatch");
 rejected("missing Node core alias",()=>mutate(files[1],s=>s.replace(/^kaspa-core-mainline = .*\r?\n/m,"")),"missing-cargo-alias");
 rejected("Bridge source drift",()=>mutate(files[2],s=>s.replace(/(kaspa-grpc-client-mainline = .*git = ")[^"]+/,(_,prefix)=>prefix+"https://example.invalid/fork.git")),"cargo-repo-mismatch");
 rejected("missing locked RPC package",()=>mutate("Cargo.lock",s=>s.split("[[package]]").filter(b=>!(b.includes('name = "kaspa-rpc-core"')&&b.includes("ad45e241"))).join("[[package]]")),"cargo-lock-runtime-package-drift");
 rejected("missing lockfile",()=>fs.unlinkSync(path.join(fixture,"Cargo.lock")),"cargo-lock-required");
 rejected("unapproved experimental source",()=>mutate(files[0],s=>{const x=JSON.parse(s);x.networks.testnet13.repo="https://example.invalid/fork.git";return JSON.stringify(x)}),"tn13-must-use-official-dagknight");
 rejected("extra network binding",()=>mutate(files[0],s=>{const x=JSON.parse(s);x.networks.unsupported={};return JSON.stringify(x)}),"unexpected-network-binding");
 console.log("PASS 8 source/lock drift cases; no runtime processes started");
} finally { fs.rmSync(fixture,{recursive:true,force:true}); }
