const root = document.documentElement;
const themeToggle = document.getElementById("theme-toggle");
const metaTheme = document.querySelector('meta[name="theme-color"]');

function currentTheme() {
  return root.classList.contains("light") ? "light" : "dark";
}

function applyTheme(theme, animate) {
  const commit = () => {
    root.classList.toggle("light", theme === "light");
    root.classList.toggle("dark", theme === "dark");
    if (metaTheme) metaTheme.setAttribute("content", theme === "light" ? "#fbfbfb" : "#0a0a0a");
    themeToggle?.setAttribute(
      "aria-label",
      theme === "light" ? "Switch to dark mode" : "Switch to light mode",
    );
    try {
      localStorage.setItem("framewise-site-theme", theme);
    } catch {
      // ignore
    }
  };

  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (animate && !reduced && typeof document.startViewTransition === "function") {
    const transition = document.startViewTransition(commit);
    transition.ready
      .then(() => {
        root.animate(
          { clipPath: ["circle(0% at 100% 0%)", "circle(150% at 100% 0%)"] },
          {
            duration: 560,
            easing: "cubic-bezier(0.4, 0, 0.2, 1)",
            pseudoElement: "::view-transition-new(root)",
          },
        );
      })
      .catch(() => {});
    return;
  }
  commit();
}

applyTheme(currentTheme(), false);

themeToggle?.addEventListener("click", () => {
  applyTheme(currentTheme() === "light" ? "dark" : "light", true);
});

const revealItems = document.querySelectorAll(".reveal");
if ("IntersectionObserver" in window) {
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          entry.target.classList.add("in");
          observer.unobserve(entry.target);
        }
      }
    },
    { threshold: 0.12, rootMargin: "0px 0px -8% 0px" },
  );
  revealItems.forEach((item) => observer.observe(item));
} else {
  revealItems.forEach((item) => item.classList.add("in"));
}

const progress = document.querySelector(".scroll-progress span");
const navLinks = [...document.querySelectorAll(".docs-nav a")];
const navSections = navLinks
  .map((link) => ({ link, element: document.querySelector(link.getAttribute("href")) }))
  .filter((item) => item.element);

let ticking = false;

function updateScrollUi() {
  ticking = false;
  const max = document.documentElement.scrollHeight - window.innerHeight;
  if (progress) {
    const ratio = max > 0 ? window.scrollY / max : 0;
    progress.style.transform = `scaleX(${Math.min(1, Math.max(0, ratio))})`;
  }
  let current = navSections[0]?.link ?? null;
  for (const section of navSections) {
    if (section.element.getBoundingClientRect().top <= 150) current = section.link;
  }
  for (const link of navLinks) link.classList.toggle("active", link === current);
}

window.addEventListener(
  "scroll",
  () => {
    if (!ticking) {
      ticking = true;
      requestAnimationFrame(updateScrollUi);
    }
  },
  { passive: true },
);
updateScrollUi();

const year = document.getElementById("year");
if (year) year.textContent = String(new Date().getFullYear());
