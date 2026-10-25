# Framewise website

The static landing page for [Framewise](https://github.com/SnegOwo/Framewise),
published at <https://snegowo.github.io/Framewise/>.

It uses the supplied logo, real screenshots, the bundled logo animation and local
fonts. There are no external assets or requests.

## Preview

From this folder:

```bash
python -m http.server 8099
```

Open <http://127.0.0.1:8099>. Use `?theme=light` to preview the light theme directly.

## Deploy

There is no build step. The `site/` folder is uploaded as a Pages artifact by
[`.github/workflows/pages.yml`](../.github/workflows/pages.yml) on every push to
`main` that touches the site. Keep every asset path relative so the page works under
the `/Framewise/` project path.

## Notes

- `assets/logo.png` — the supplied artwork, used unchanged.
- `assets/logo-animation.mp4` — the launch animation (768×768, 24 fps, 5.2 s, muted on the site).
- `assets/screens/` — dark and light screenshots from the current build.
- `assets/fonts/` — Hanken Grotesk and JetBrains Mono, bundled locally.
- There is no waitlist or form. The closing section is a static release note.
- The hero preview tabs swap between Dashboard, Projects and Reports.
- The gallery lightbox supports arrow keys and prev/next buttons.
- The theme toggle uses the browser View Transitions API for the circular reveal
  from the top-right, with an instant fallback.
- Scroll progress and the active nav link update as you move down the page, and the
  hero frame has a very subtle pointer tilt on fine pointers.
