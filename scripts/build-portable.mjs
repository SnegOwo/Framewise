// Builds the portable Framewise folder:
//   framewise.exe + framewise.portable + README.txt
//
// The marker file switches the app into portable mode: the SQLite database and
// the WebView2 profile are written to ./data next to the executable, and
// launch-at-login is disabled. No installer, no archive, no registry keys.
import { execSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = pkg.version;

console.log("Building release binary (no installer)…");
execSync("npx tauri build --no-bundle", { cwd: root, stdio: "inherit" });

const exe = join(root, "src-tauri", "target", "release", "framewise.exe");
if (!existsSync(exe)) {
  throw new Error(`Missing release binary: ${exe}`);
}

const bundleDir = join(root, "release");
const folder = join(bundleDir, `Framewise_${version}_x64_portable`);
mkdirSync(bundleDir, { recursive: true });
rmSync(folder, { recursive: true, force: true });
mkdirSync(folder, { recursive: true });

copyFileSync(exe, join(folder, "framewise.exe"));
writeFileSync(join(folder, "framewise.portable"), "Framewise portable mode\n");

const readme = `Framewise ${version} — portable

1. Keep framewise.exe, framewise.portable and this README in one folder.
2. Run framewise.exe. There is no installer and no archive.
3. Everything is written to the "data" folder beside the exe:
     data\\framewise.sqlite3   your sessions, projects and settings
     data\\webview\\           the WebView2 profile and cache
   Nothing is written to AppData or LocalAppData.
4. Launch at login is disabled in portable mode.
5. To remove Framewise, delete this folder.
6. Framewise needs the Microsoft Edge WebView2 runtime. It ships with
   Windows 10 and 11. If it is missing, Windows shows a local error dialog
   explaining how to install it; no download happens on its own.

Tray: click once for the mini timer, double-click for the full app.
Framewise works fully offline. No accounts, no telemetry, no remote requests.
Made by Sneg HQ. MIT licensed.
`;

writeFileSync(join(folder, "README.txt"), readme);

// Remove any legacy zip so the folder is the only portable artifact.
for (const name of readdirSync(bundleDir)) {
  if (name.endsWith(".zip")) rmSync(join(bundleDir, name), { force: true });
}

console.log(`\nPortable folder: ${folder}`);
