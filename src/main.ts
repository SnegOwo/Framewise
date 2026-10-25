import "@fontsource-variable/hanken-grotesk";
import "@fontsource-variable/jetbrains-mono";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { applyTheme, type ThemeSetting } from "./lib/theme";

let saved: ThemeSetting = "dark";
try {
  saved = (localStorage.getItem("framewise-theme") as ThemeSetting) || "dark";
} catch {
  // ignore
}
applyTheme(saved, false);

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
