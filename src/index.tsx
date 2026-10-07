/* @refresh reload */
import { render } from "solid-js/web";
import { initializeLogging } from "./lib/logging";
import { initializeStorage } from "./lib/storage";

initializeLogging();

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html?"
  );
}

async function start() {
  await initializeStorage();
  // Import preference consumers only after the native store is hydrated.
  const { applyTheme, getStoredTheme } = await import("./lib/theme");
  applyTheme(getStoredTheme());
  const { i18nReady } = await import("./lib/i18n");
  await i18nReady;
  const { default: App } = await import("./App");
  render(() => <App />, root!);
}
void start().catch(error => {
  console.error("Application startup failed", error);
  if (root) root.textContent = "Không thể tải cấu hình. Vui lòng khởi động lại B4S. / Unable to load settings. Please restart B4S.";
});
