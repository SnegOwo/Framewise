// Fails if the built app or source references remote assets or network endpoints.
// Framewise is offline-only, so any external URL is a bug.
import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..");

const targets = ["dist", "src", "public", "index.html"];
const textExtensions = /\.(js|mjs|ts|svelte|css|html|json|svg|txt)$/i;

// Tauri uses these two IPC origins internally; they never leave the machine.
// Svelte embeds documentation links in runtime error messages, never fetched.
const allowed = [
  /http:\/\/ipc\.localhost/,
  /https?:\/\/(www\.)?w3\.org\//,
  /https:\/\/svelte\.dev\/e\//,
];
const pattern = /https?:\/\/[^\s"'`)<>]+/g;

const findings = [];

function scan(file) {
  if (!textExtensions.test(file)) return;
  let content;
  try {
    content = readFileSync(file, "utf8");
  } catch {
    return;
  }
  const matches = content.match(pattern);
  if (!matches) return;
  for (const match of new Set(matches)) {
    if (allowed.some((rule) => rule.test(match))) continue;
    findings.push({ file: relative(root, file), match });
  }
}

function walk(target) {
  const full = join(root, target);
  if (!existsSync(full)) return;
  const stats = statSync(full);
  if (stats.isDirectory()) {
    for (const entry of readdirSync(full)) walk(join(target, entry));
  } else {
    scan(full);
  }
}

for (const target of targets) walk(target);

if (findings.length > 0) {
  console.error("Offline audit failed. Remote references found:");
  for (const item of findings) console.error(`  ${item.file}: ${item.match}`);
  process.exit(1);
}

console.log("Offline audit passed: no remote assets or network endpoints found.");
