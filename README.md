<p align="center">
  <img src="Assets/FrameWiseLogoPNG.png" width="92" alt="Framewise logo" />
</p>

<h1 align="center">Framewise</h1>

<p align="center">
  <strong>A quiet, offline time tracker for Adobe creatives.</strong><br />
  Windows 10/11 &nbsp;·&nbsp; Portable by default &nbsp;·&nbsp; MIT
</p>

<p align="center">
  <a href="https://snegowo.github.io/Framewise/">Website</a> ·
  <a href="https://snegowo.github.io/Framewise/docs.html">Docs</a> ·
  <a href="#why-framewise">Why</a> ·
  <a href="#screenshots">Screenshots</a> ·
  <a href="#portable">Portable</a> ·
  <a href="#privacy">Privacy</a> ·
  <a href="#development">Development</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

<p align="center">
  <img src="docs/screenshots/dark-1200x840-dashboard.png" width="100%" alt="Framewise dashboard: 5h 28m today on YouTube Client 2, a 24-hour timeline, four stat tiles and a 294-hour heatmap" />
</p>

Framewise follows the app in focus and credits Premiere, After Effects and Photoshop
time to the project you are actually on. It lives in the tray, keeps every number on
your machine, and stays out of the edit.

> **Status:** Framewise 1.0 is available for Windows 10/11. Open source under MIT.
> No telemetry, no accounts. The only optional network request is a monthly update
> check, off by default.

## Contents

- [Why Framewise](#why-framewise)
- [Highlights](#highlights)
- [Screenshots](#screenshots)
- [Download](#download)
- [Website](#website)
- [Privacy](#privacy)
- [How it works](#how-it-works)
- [Keyboard](#keyboard)
- [Development](#development)
- [Architecture](#architecture)
- [Tests](#tests)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Why Framewise

Most time trackers ask you to classify your work after the fact. Framewise starts
from the app you are actually using. If Premiere is in focus, the time belongs to the
project you picked. If you switch to a browser, that time is still recorded, but never
silently assigned to client work.

The result is a small, legible record of a creative day: one live sentence, one
timeline, one heatmap, and reports you can export without an account.

## Highlights

- **Adobe-first attribution.** Adobe apps get the active project. Everything else is
  tracked for general stats, or credited to the project once you give it focus tools.
  No project selected means unassigned, never guessed.
- **Focus tools.** Give each project its tools — Premiere, Photoshop, Zen, Flameshot,
  anything. Start from a **Montage**, **YouTube video**, **Motion graphics** or
  **Photo edit** template, or pick from apps already saved in Framewise. A small green
  or red bubble on the dashboard tells you whether the app in focus counts, and each
  project separates selected tools from other time with a focus percentage.
- **A real 24-hour view.** App tracks, session pills and a now cursor, so the day is
  legible at a glance.
- **Goals that feel finished.** Daily and weekly rings, a contribution heatmap, six
  report ranges, CSV/JSON/Markdown export, and a quiet time-bloom reward when a goal
  completes.
- **Know what you are looking at.** Click any app name to see its friendly name,
  category, executable and — when it is running — the full path.
- **A tray that behaves.** Single click opens a mini timer near the tray; double click
  opens the full app. The mini shows **Pin** — pin it to keep it open and drag it
  anywhere. Drag the corner to resize it, all the way down to a time-only clock.
  Unpinned, it behaves like a tray popover and closes when it loses focus.
- **Portable and private.** One folder, no installer, no remote fonts, no accounts,
  no telemetry. The only network request is the opt-in monthly update check, and it
  is disabled in portable mode.
- **A guided first run.** Choose theme, whether goals count focus time only,
  notifications and launch at login. Scan for Adobe apps or pick their `.exe` files —
  or skip everything to the defaults.
- **A real theme switch.** Dark and light with a circular reveal from the top-right,
  animated sun/moon glyphs, and a reduced-motion fallback.

## Screenshots

<p align="center">
  <img src="docs/screenshots/light-1200x840-dashboard.png" width="49%" alt="Framewise dashboard in light mode" />
  <img src="docs/screenshots/dark-1200x840-projects.png" width="49%" alt="Framewise projects with client colors and weekly progress" />
</p>

<p align="center">
  <img src="docs/screenshots/dark-1200x840-reports.png" width="49%" alt="Framewise reports with a weekly range, summary and daily activity chart" />
  <img src="docs/screenshots/dark-1200x840-sessions.png" width="49%" alt="Framewise sessions grouped by day with inline project reassignment" />
</p>

<p align="center">
  <img src="docs/screenshots/dark-1200x840-settings.png" width="49%" alt="Framewise settings with autosaving rows, theme and date format" />
  <img src="docs/screenshots/mini-360x132-dark.png" width="36%" alt="Framewise mini timer near the tray" />
</p>

## Download

Framewise 1.0 is available for Windows 10/11 from
[GitHub Releases](https://github.com/SnegOwo/Framewise/releases/latest):

- **Installer** — `Framewise_1.0.2_x64-setup.exe`
- **Portable** — `Framewise_1.0.2_x64_portable.zip`

The portable zip contains:

```
Framewise_1.0.2_x64_portable/
├── framewise.exe
├── framewise.portable
└── README.txt
```

Run `framewise.exe` from that folder. A `framewise.portable` marker switches the app
into portable mode:

```
data/framewise.sqlite3   sessions, projects and settings
data/webview/            the WebView2 profile and cache
```

Nothing is written to AppData or LocalAppData. Launch at login is disabled. Delete the
folder to remove every trace. **Settings → About** reports whether the running build
is installed or portable, with the active data path.

**Updates.** Installer builds can opt into a monthly update check during first-run
setup or in Settings. It is off by default. Portable builds update by downloading a
new portable zip; the in-app updater is disabled in portable mode. Update manifests
and signed artifacts are attached to each GitHub release.

**Mini timer:** a single tray click opens the mini near the tray. It is a popover by
default and closes on focus loss. Click **Pin** to keep it open, then drag it anywhere
from the timer body. Drag the bottom-right corner to resize it — below 280 × 108 it
becomes a time-only clock. Pin and size are remembered.

Build the folder with:

```bash
npm run build:portable
```

## Website

The landing page is at <https://snegowo.github.io/Framewise/>. Its source is in
[`site/`](site/) and deploys from `main` through GitHub Pages, with no build step and
no external assets. The full guide lives at
<https://snegowo.github.io/Framewise/docs.html>. The 26-second launch film is on the
landing page.

## Privacy

Framewise makes no network requests unless you enable **Settings → Updates → Monthly
update check**. When enabled, it asks GitHub's release metadata once every 30 days
whether a newer installer exists. No usage data, project data or identifiers are sent.
No telemetry, accounts, remote fonts or CDN assets. The content security policy allows
only the app's own files and the Tauri IPC bridge. Your data is a single SQLite file in
WAL mode with a 15-second flush and a checkpoint on quit.

The app records **executable names**, not window titles. App inspection looks up the
full path locally, on demand, and never sends it anywhere.

## How it works

**Attribution.** The foreground executable is sampled every 1.5 seconds. Ignored
executables are never tracked. Adobe executables credit the active project. Once a
project has **focus tools**, every tracked app is credited to that project and marked
selected or other, so the dashboard can show a green/red bubble and the project can
report a focus percentage. Projects without tools keep non-Adobe time unassigned.
Framewise's own windows are never credited; while Framewise is in focus it keeps the
most recently used real app's session alive, so opening the app does not create a gap.

**Pausing.** Idle, lock, sleep and hibernation stop the clock. Idle time is cut back
to the last real input, so idle minutes are never billed. A sleep gap is never
bridged with phantom time.

**Goals.** A new install starts with **no goals** — no surprise 4-hour target. Create
daily or weekly goals from the dashboard; rings grow on first view only and the value
can be edited inline. By default, **only selected focus tools count toward goals**:
if a project has 100 minutes but only 10 are in its tools, the goal gets 10. Turn this
off in Settings to count all tracked project time; projects without tools always count
in full. When a goal is reached, a time-bloom reward plays with an optional soft chime.
Sound can be disabled in Settings.

**App inspection.** Click any app name or icon (dashboard status, sessions, timeline)
to see its friendly name, category, executable and, when it is running, the full path.
That is how `mpvnet.exe` becomes `mpv.net` with a description.

**Projects.** Name, client, color, archive, delete (confirm only for delete), active
project, per-project stats. Creating a project opens a tool picker with Montage,
YouTube video, Motion graphics and Photo edit templates, plus the apps already saved
in Framewise; you can skip it and add tools later. A project detail page has focus
tools, a focus percentage, selected/other app bars, a 30-day chart and a six-month mini
heatmap. Type a name that does not match in the dashboard project field and Framewise
offers to create it inline.

**Reports.** This week, last week, this month, 30 days, this year, or a custom range.
A single project filter and a single export menu. Goal line inside the chart; no
overlapping labels.

**Sessions.** Grouped by day with sticky headings. Reassign a session's project
inline, edit or delete on hover, or add a manual session from the dock.

**Settings.** Autosaving rows, a stepper for idle timeout, segmented theme and date
format controls, native `.exe` pickers for the Adobe and ignore lists, a scan for
installed Adobe apps, a sound toggle, and a switch for counting only focus time toward
goals.

**Theme.** Dark and light with a circular reveal from the top-right. Motion is
transform/opacity only and respects `prefers-reduced-motion`.

## Keyboard

| Shortcut | Action |
| --- | --- |
| `/` or `Ctrl+K` | Open the dock / project switcher |
| `1` … `5` | Dashboard, Projects, Reports, Sessions, Settings |
| `↑` `↓` | Move through dock results |
| `Enter` | Run the selected command |
| `Esc` | Close the dock or a dialog |

## Development

Requirements: Windows 10/11, Node.js 20+, Rust with the MSVC toolchain, WebView2.

```bash
npm install
npm run tauri dev
```

Checks:

```bash
npm run check          # svelte-check / TypeScript / a11y
npm run build          # Vite production build
npm run audit:offline  # fails on any remote asset or endpoint
cargo test --lib       # 34 backend tests
cargo check            # backend type check
```

Run the cargo commands from `src-tauri/`.

## Architecture

```
src/
  App.svelte                 shell, polling, shortcuts, event bridge
  app.css                    design tokens, light/dark themes, components
  lib/
    api.ts                   typed Tauri command wrappers
    stores.ts                bootstrap, status, toasts, app inspection
    format.ts                local dates, durations, app names
    router.ts                pages and cross-page filters
    export.ts                CSV/JSON/Markdown export
    theme.ts                 dark, light, circular reveal
    celebration.ts           goal reward state
    components/              dock, charts, heatmap, timeline, mini, reward
    pages/                   Dashboard, Projects, Reports, Sessions, Settings
src-tauri/
  src/
    lib.rs                   Tauri builder, windows, portable setup, state
    portable.rs              portable marker, data folder, WebView2 profile
    tracker.rs               polling engine and attribution rules
    db.rs                    schema, sessions, focus tools, reports, goals, backup/import
    commands.rs              IPC commands
    winapi.rs                foreground window, idle, lock, app paths, CPU
    timeutil.rs              local day, midnight and week helpers
    goals.rs                 goal progress and streaks
    export.rs                report writers
    tray.rs                  tray menu, mini toggle, project switching
```

## Tests

The backend keeps **34 unit tests** covering midnight splitting, multi-day intervals,
idle cutoff, sleep gap detection, Adobe attribution, focus tools, ignored apps, goal
progress and streaks, report grouping, CSV escaping and reassignment.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first.

The short version: keep the app offline and local, never add telemetry or remote
assets, keep the tracking behaviour covered by tests, and run the checks before a PR.

## Security

Please do not open a public issue for a security problem. See [SECURITY.md](SECURITY.md).

## License

MIT. See [LICENSE](LICENSE). Copyright © 2026 Sneg HQ.
