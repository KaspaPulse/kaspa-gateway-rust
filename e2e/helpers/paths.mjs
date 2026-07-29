import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const repository = process.env.KGW_REPOSITORY || path.resolve(e2eDir, "..");
export const artifactRoot = process.env.KGW_ZERO_TOUCH_ARTIFACT_DIR ||
  path.join(repository, "artifacts", "zero-touch-e2e", `wdio-${Date.now()}`);

export async function ensureDir(directory) {
  await fs.mkdir(directory, { recursive: true });
  return directory;
}

export function caseDir(slug) {
  return path.join(artifactRoot, "cases", slug);
}

export async function writeJson(file, value) {
  await ensureDir(path.dirname(file));
  await fs.writeFile(file, JSON.stringify(value, null, 2), "utf8");
}

export async function writeText(file, value) {
  await ensureDir(path.dirname(file));
  await fs.writeFile(file, String(value ?? ""), "utf8");
}

export function helperScript(name) {
  return path.join(e2eDir, "helpers", name);
}

