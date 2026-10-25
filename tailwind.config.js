/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,js,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter Variable", "Inter", "Segoe UI", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono Variable", "JetBrains Mono", "Cascadia Mono", "monospace"],
      },
      colors: {
        canvas: "var(--bg)",
        surface: "var(--surface)",
        surface2: "var(--surface-2)",
        line: "var(--border)",
        line2: "var(--border-strong)",
        ink: "var(--text)",
        muted: "var(--muted)",
        accent: "var(--accent)",
        accenttext: "var(--accent-text)",
      },
      borderRadius: {
        DEFAULT: "6px",
        md: "8px",
      },
    },
  },
  plugins: [],
};
