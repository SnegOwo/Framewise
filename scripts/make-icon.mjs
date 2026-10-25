// Uses the supplied Framewise logo artwork as the icon source.
// Run: node scripts/make-icon.mjs && npx tauri icon src-tauri/app-icon.png
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = resolve(root, "Assets", "FrameWiseLogoPNG.png");
const target = resolve(root, "src-tauri", "app-icon.png");

mkdirSync(dirname(target), { recursive: true });
copyFileSync(source, target);
console.log(`copied ${source} -> ${target}`);
