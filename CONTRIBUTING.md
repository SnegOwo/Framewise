# Contributing to Framewise

Framewise is a small, local-first Windows app. Contributions are welcome if they keep
it quiet, offline and predictable.

## Ground rules

- **Offline first.** No runtime network requests, telemetry, remote fonts, CDNs or
  analytics. New assets must be bundled. `npm run audit:offline` must pass.
- **Privacy by default.** Never collect window titles, keystrokes, clipboard data or
  file contents. The tracker records foreground executable names and time only.
- **Do not break tracking semantics.** Midnight splitting, idle cutoff, lock/sleep
  handling and Adobe attribution are covered by tests. Change them deliberately and
  with tests.
- **One visual language.** Use the tokens in `src/app.css`; accent for live/progress,
  project colours for projects. No gradients/glass for decoration, no emoji.
- **Keep it small.** If a screen is not better with an element, remove the element.

## Setup

Requirements: Windows 10/11, Node.js 20+, Rust with the MSVC toolchain, WebView2.

```bash
npm install
npm run tauri dev
```

The frontend alone cannot exercise Tauri IPC; use `tauri dev` for real work.

## Before opening a PR

Run everything from the repository root, with cargo commands in `src-tauri/`:

```bash
npm run check
npm run build
npm run audit:offline
cargo test --lib
cargo check
```

All four should be clean. UI changes should also be checked in dark and light mode at
1200×840 and 800×600.

## Code style

**Rust**
- `rustfmt` defaults; no dead code or unused imports.
- Keep Win32 calls inside `winapi.rs`.
- SQL stays in `db.rs`; IPC stays in `commands.rs`.
- Tracker state is guarded by the existing `Tracker` mutex; do not add a second loop.

**Svelte / TypeScript**
- Svelte 5 runes (`$state`, `$derived`, `$effect`, `$props`).
- Components under `src/lib/components`, pages under `src/lib/pages`.
- Types for every IPC payload in `src/lib/types.ts`.
- Use the shared helpers in `format.ts`; dates are English by default with a System
  option.
- Motion: transform/opacity only, 160–240 ms, springs for layout, and always respect
  `prefers-reduced-motion`.

## Tests

Backend tests live beside the code in `#[cfg(test)]` modules. Add cases for:
- midnight and multi-day intervals
- idle and sleep gaps
- attribution and ignored apps
- goal progress and streaks
- report grouping and CSV escaping
- session reassignment

There is no frontend test runner yet; keep components small and pure where possible.

## Commits and pull requests

- One focused change per commit; write the message in the imperative
  (`Fix idle cutoff at midnight`).
- Describe the user-facing result, not only the files touched.
- Include before/after screenshots for UI work.
- Do not commit generated output (`dist/`, `target/`, `release/`).
- The landing page lives in `site/` and deploys through GitHub Pages. Keep it static,
  self-contained and free of remote fonts or assets.
- Do not commit real user databases or exports.

## Reporting bugs

Open an issue with:
- Windows version and Framewise version (Settings → About)
- whether the build is installed or portable
- steps to reproduce
- expected vs actual behaviour
- a screenshot if it is visual

Never attach a real `framewise.sqlite3` unless you have removed personal data.

## License

By contributing you agree that your contribution is licensed under the MIT license.
