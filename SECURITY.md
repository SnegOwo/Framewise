# Security

Framewise is an offline Windows application. It makes no network requests by default,
does not create accounts, and sends no analytics. The optional monthly update check is
off unless enabled, and only fetches GitHub release metadata.

## Supported versions

| Version | Supported |
| --- | --- |
| 1.0.x | Yes |

## Reporting a vulnerability

Please use GitHub's **private vulnerability reporting** for this repository
(Security → Report a vulnerability). Do not open a public issue for a security
problem.

Include:
- the affected version and whether the build is installed or portable
- Windows version
- a description of the issue and its impact
- minimal reproduction steps
- any relevant local files (with personal data removed)

## Scope

Areas worth reviewing:
- SQLite import/backup path handling
- portable data-directory resolution
- Tauri CSP and capability configuration
- WebView2 profile isolation
- app-path lookup for the inspection card
- process/permission handling in `winapi.rs`

Out of scope:
- third-party software shown in the app
- data loss caused by deleting the portable folder manually
- issues that require a compromised local machine or administrator access

## Privacy

Framewise stores executable names and time intervals in a local SQLite file. It does
not read window titles, keystrokes, clipboard content, or file contents. App
inspection resolves an executable path locally and on demand; it is never transmitted.
