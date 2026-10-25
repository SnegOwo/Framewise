const root = document.documentElement;
const themeToggle = document.getElementById("theme-toggle");
const metaTheme = document.querySelector('meta[name="theme-color"]');

function currentTheme() {
  return root.classList.contains("light") ? "light" : "dark";
}

const HERO_SHOTS = {
  dashboard: {
    dark: "assets/screens/dashboard-dark.png",
    light: "assets/screens/dashboard-light.png",
  },
  projects: {
    dark: "assets/screens/projects-dark.png",
    light: "assets/screens/projects-light.png",
  },
  reports: {
    dark: "assets/screens/reports-dark.png",
    light: "assets/screens/reports-light.png",
  },
};

const heroImage = document.getElementById("hero-shot");
const heroTabs = [...document.querySelectorAll(".hero-tabs button")];
let heroShot = "dashboard";
let swapTimer;

function showHeroShot(name) {
  const shot = HERO_SHOTS[name];
  if (!shot || !heroImage) return;
  heroShot = name;
  const theme = currentTheme();
  const next = theme === "light" ? shot.light : shot.dark;
  heroImage.dataset.dark = shot.dark;
  heroImage.dataset.light = shot.light;
  const frame = heroImage.closest(".app-frame");
  frame?.classList.add("swapping");
  clearTimeout(swapTimer);
  swapTimer = setTimeout(() => {
    heroImage.src = next;
    frame?.classList.remove("swapping");
  }, 130);
  for (const tab of heroTabs) {
    const active = tab.dataset.shot === name;
    tab.classList.toggle("active", active);
    tab.setAttribute("aria-selected", String(active));
  }
}

heroTabs.forEach((tab) => {
  tab.addEventListener("click", () => showHeroShot(tab.dataset.shot));
});

function applyTheme(theme, animate) {
  const commit = () => {
    root.classList.toggle("light", theme === "light");
    root.classList.toggle("dark", theme === "dark");
    document.querySelectorAll("img[data-dark]").forEach((img) => {
      const next = theme === "light" ? img.dataset.light : img.dataset.dark;
      if (next && img.getAttribute("src") !== next) img.setAttribute("src", next);
    });
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

// Reveal sections as they enter the viewport.
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

// Scroll progress + active nav.
const progress = document.querySelector(".scroll-progress span");
const navLinks = [...document.querySelectorAll(".nav a")];
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
  let current = null;
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

// Subtle pointer tilt on the hero frame.
const heroVisual = document.querySelector(".hero-visual");
const heroFrame = document.querySelector(".app-frame");
const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const finePointer = window.matchMedia("(pointer: fine)").matches;

if (heroVisual && heroFrame && finePointer && !reduced) {
  heroVisual.addEventListener("pointermove", (event) => {
    const rect = heroFrame.getBoundingClientRect();
    const x = (event.clientX - rect.left) / rect.width - 0.5;
    const y = (event.clientY - rect.top) / rect.height - 0.5;
    heroFrame.style.setProperty("--ry", `${x * 4.5}deg`);
    heroFrame.style.setProperty("--rx", `${-y * 3.5}deg`);
  });

  heroVisual.addEventListener("pointerleave", () => {
    heroFrame.style.removeProperty("--rx");
    heroFrame.style.removeProperty("--ry");
  });
}

// Lightbox with keyboard / arrow navigation.
const lightbox = document.getElementById("lightbox");
const lightboxImage = document.getElementById("lightbox-img");
const lightboxCount = document.getElementById("lightbox-count");
const lightboxClose = lightbox?.querySelector(".lightbox-close");
const lightboxPrev = lightbox?.querySelector(".lightbox-nav.prev");
const lightboxNext = lightbox?.querySelector(".lightbox-nav.next");
const screenCards = [...document.querySelectorAll(".screen-card")];
let lightboxIndex = 0;

function showLightbox(index) {
  if (!lightboxImage || screenCards.length === 0) return;
  lightboxIndex = (index + screenCards.length) % screenCards.length;
  const img = screenCards[lightboxIndex].querySelector("img");
  if (!img) return;
  lightboxImage.src = img.currentSrc || img.src;
  lightboxImage.alt = img.alt;
  if (lightboxCount) {
    lightboxCount.textContent = `${lightboxIndex + 1} / ${screenCards.length}`;
  }
}

for (const [index, card] of screenCards.entries()) {
  card.addEventListener("click", () => {
    if (!lightbox) return;
    showLightbox(index);
    lightbox.showModal();
  });
}

lightboxPrev?.addEventListener("click", () => showLightbox(lightboxIndex - 1));
lightboxNext?.addEventListener("click", () => showLightbox(lightboxIndex + 1));
lightboxClose?.addEventListener("click", () => lightbox?.close());
lightbox?.addEventListener("click", (event) => {
  if (event.target === lightbox) lightbox.close();
});

window.addEventListener("keydown", (event) => {
  if (!lightbox?.open) return;
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    showLightbox(lightboxIndex - 1);
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    showLightbox(lightboxIndex + 1);
  }
});

const year = document.getElementById("year");
if (year) year.textContent = String(new Date().getFullYear());

const brandVideo = document.getElementById("brand-video");
brandVideo?.addEventListener("playing", () => brandVideo.classList.add("playing"));
