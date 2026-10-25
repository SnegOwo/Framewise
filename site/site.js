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
const heroTabsWrap = document.querySelector(".hero-tabs");
const heroVisual = document.querySelector(".hero-visual");
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const HERO_ORDER = ["dashboard", "projects", "reports"];
const TAB_MS = 7000;
const SETTLE_MS = 1500;
let heroShot = "dashboard";
let swapTimer;
let rotateTimer;
let settleTimer;
let tabHeld = false;

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

function setCounting(on) {
  for (const tab of heroTabs) tab.classList.toggle("counting", on && tab.classList.contains("active"));
}

function clearHeroCycle() {
  clearInterval(rotateTimer);
  clearTimeout(settleTimer);
  rotateTimer = undefined;
  settleTimer = undefined;
  setCounting(false);
}

function beginHeroCycle() {
  clearInterval(rotateTimer);
  if (reducedMotion || tabHeld || document.hidden) return;
  setCounting(true);
  rotateTimer = setInterval(() => {
    if (tabHeld || document.hidden) return;
    const index = HERO_ORDER.indexOf(heroShot);
    showHeroShot(HERO_ORDER[(index + 1) % HERO_ORDER.length]);
    setCounting(true);
  }, TAB_MS);
}

// A quick click should not reset the preview timer. Wait for the tab to settle,
// then start a full cycle.
function armHeroCycle(delay = SETTLE_MS) {
  clearHeroCycle();
  if (reducedMotion || tabHeld) return;
  settleTimer = setTimeout(() => {
    settleTimer = undefined;
    if (!tabHeld && !document.hidden) beginHeroCycle();
  }, delay);
}

heroTabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    const name = tab.dataset.shot;
    if (!name || name === heroShot) return;
    showHeroShot(name);
    armHeroCycle();
  });
});

heroTabsWrap?.addEventListener("pointerenter", () => {
  tabHeld = true;
  clearHeroCycle();
});
heroTabsWrap?.addEventListener("pointerleave", () => {
  tabHeld = false;
  armHeroCycle();
});
heroTabsWrap?.addEventListener("focusin", () => {
  tabHeld = true;
  clearHeroCycle();
});
heroTabsWrap?.addEventListener("focusout", () => {
  tabHeld = false;
  armHeroCycle();
});

document.addEventListener("visibilitychange", () => {
  if (document.hidden) clearHeroCycle();
  else armHeroCycle();
});

showHeroShot("dashboard");
armHeroCycle(700);

// Interactive mini timer preview.
const miniDemo = document.getElementById("mini-demo");
const miniClock = document.getElementById("mini-clock");
const miniPause = document.getElementById("mini-pause");
const miniPin = document.getElementById("mini-pin");
const miniState = document.getElementById("mini-state");
const miniFocus = document.getElementById("mini-focus");
let miniSeconds = 1 * 3600 + 42 * 60 + 18;
let miniRunning = true;

function renderMini() {
  if (!miniClock) return;
  const hours = Math.floor(miniSeconds / 3600);
  const minutes = Math.floor((miniSeconds % 3600) / 60);
  const seconds = miniSeconds % 60;
  miniClock.textContent = [hours, minutes, seconds]
    .map((part) => String(part).padStart(2, "0"))
    .join(":");
}

function renderMiniState() {
  if (!miniState) return;
  if (!miniRunning) miniState.textContent = "Paused · click play to resume";
  else if (miniPin?.getAttribute("aria-pressed") === "true") miniState.textContent = "Pinned · stays open";
  else miniState.textContent = "Live demo · click pause or pin";
}

renderMini();
renderMiniState();

window.setInterval(() => {
  if (document.hidden || !miniRunning) return;
  miniSeconds += 1;
  renderMini();
}, 1000);

miniPause?.addEventListener("click", () => {
  miniRunning = !miniRunning;
  miniDemo?.classList.toggle("paused", !miniRunning);
  miniPause.setAttribute("aria-pressed", String(!miniRunning));
  miniPause.setAttribute("aria-label", miniRunning ? "Pause demo timer" : "Resume demo timer");
  renderMiniState();
});

miniPin?.addEventListener("click", () => {
  const pinned = miniPin.getAttribute("aria-pressed") === "true";
  miniPin.setAttribute("aria-pressed", String(!pinned));
  miniPin.setAttribute("aria-label", pinned ? "Pin demo timer" : "Unpin demo timer");
  miniDemo?.classList.toggle("pinned", !pinned);
  renderMiniState();
});

miniFocus?.addEventListener("click", () => {
  const on = miniFocus.getAttribute("aria-pressed") === "true";
  miniFocus.setAttribute("aria-pressed", String(!on));
  miniFocus.classList.toggle("on", !on);
  miniFocus.classList.toggle("off", on);
  miniFocus.textContent = on ? "Off tool" : "On tool";
  miniFocus.setAttribute(
    "aria-label",
    on ? "Mark demo app as a selected tool" : "Mark demo app as another app",
  );
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
  .filter((link) => link.getAttribute("href")?.startsWith("#"))
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

// Subtle pointer tilt on the hero frame + parallax on the live preview.
const heroFrame = document.querySelector(".app-frame");
const finePointer = window.matchMedia("(pointer: fine)").matches;

if (heroVisual && heroFrame && finePointer && !reducedMotion) {
  heroVisual.addEventListener("pointermove", (event) => {
    const rect = heroFrame.getBoundingClientRect();
    const x = (event.clientX - rect.left) / rect.width - 0.5;
    const y = (event.clientY - rect.top) / rect.height - 0.5;
    heroVisual.style.setProperty("--mx", x.toFixed(3));
    heroVisual.style.setProperty("--my", y.toFixed(3));
    heroFrame.style.setProperty("--ry", `${x * 4.5}deg`);
    heroFrame.style.setProperty("--rx", `${-y * 3.5}deg`);
  });

  heroVisual.addEventListener("pointerleave", () => {
    heroVisual.style.removeProperty("--mx");
    heroVisual.style.removeProperty("--my");
    heroFrame.style.removeProperty("--rx");
    heroFrame.style.removeProperty("--ry");
  });
}

// Pointer spotlight on cards.
if (finePointer) {
  for (const card of document.querySelectorAll(".feature, .step")) {
    card.addEventListener("pointermove", (event) => {
      const rect = card.getBoundingClientRect();
      card.style.setProperty("--px", `${event.clientX - rect.left}px`);
      card.style.setProperty("--py", `${event.clientY - rect.top}px`);
    });
  }
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

// Launch film: custom poster/play overlay, native controls after the first play.
const trailerFrame = document.getElementById("trailer-frame");
const trailerVideo = document.getElementById("trailer-video");
const trailerPlay = document.getElementById("trailer-play");

trailerPlay?.addEventListener("click", () => {
  if (!trailerVideo) return;
  trailerPlay.classList.add("hidden");
  trailerFrame?.classList.add("playing");
  trailerVideo.play().catch(() => {
    trailerPlay.classList.remove("hidden");
    trailerFrame?.classList.remove("playing");
  });
});

trailerVideo?.addEventListener("ended", () => {
  trailerPlay?.classList.remove("hidden");
  trailerFrame?.classList.remove("playing");
  trailerVideo.currentTime = 0;
});
