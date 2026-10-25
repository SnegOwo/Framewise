(function () {
  try {
    var t = localStorage.getItem("framewise-theme") || "dark";
    if (t === "system") {
      t = window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
    }
    document.documentElement.classList.toggle("dark", t !== "light");
    document.documentElement.dataset.theme = t;
  } catch (_) {}
})();
