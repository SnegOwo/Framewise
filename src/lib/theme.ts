export type ThemeSetting = "dark" | "light" | "system";
export type ResolvedTheme = "dark" | "light";

type ViewTransitionDocument = Document & {
  startViewTransition?: (callback: () => void) => { ready: Promise<void> };
};

export function resolveTheme(setting: ThemeSetting): ResolvedTheme {
  if (setting === "system") {
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  }
  return setting;
}

export function applyTheme(setting: ThemeSetting, animate = true) {
  const resolved = resolveTheme(setting);
  const root = document.documentElement;

  const commit = () => {
    root.classList.toggle("dark", resolved === "dark");
    root.dataset.theme = resolved;
    root.dataset.themeSetting = setting;
    try {
      localStorage.setItem("framewise-theme", setting);
    } catch {
      // ignore
    }
  };

  let reduced = false;
  try {
    reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  } catch {
    // ignore
  }

  const doc = document as ViewTransitionDocument;
  if (animate && !reduced && typeof doc.startViewTransition === "function") {
    try {
      const transition = doc.startViewTransition(commit);
      transition.ready
        .then(() => {
          const keyframes = {
            clipPath: ["circle(0% at 100% 0%)", "circle(150% at 100% 0%)"],
          };
          const options = {
            duration: 560,
            easing: "cubic-bezier(0.4, 0, 0.2, 1)",
            pseudoElement: "::view-transition-new(root)",
          };
          root.animate(keyframes, options as unknown as KeyframeAnimationOptions);
        })
        .catch(() => {
          // The theme is already committed even if the reveal is interrupted.
        });
      return;
    } catch {
      // Fall through to an instant switch on browsers without support.
    }
  }

  commit();
}

export function watchSystemTheme(): () => void {
  const media = window.matchMedia("(prefers-color-scheme: light)");
  const handler = () => {
    let setting: ThemeSetting = "dark";
    try {
      setting = (localStorage.getItem("framewise-theme") as ThemeSetting) || "dark";
    } catch {
      // ignore
    }
    if (setting === "system") applyTheme("system");
  };
  media.addEventListener("change", handler);
  return () => media.removeEventListener("change", handler);
}
