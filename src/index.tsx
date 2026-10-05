/* @refresh reload */
import { render } from "solid-js/web";
import { i18nReady } from "./lib/i18n";
import App from "./App";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html?"
  );
}

i18nReady.then(() => render(() => <App />, root!));
