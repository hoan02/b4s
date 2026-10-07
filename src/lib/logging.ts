import { error, warn, info, debug } from "@tauri-apps/plugin-log";
import { isTauri } from "./tauri";

type ConsoleLevel = "log" | "warn" | "error" | "info" | "debug";
type LogSink = (message: string) => Promise<void>;

function format(value: unknown): string {
  if (value instanceof Error) return value.stack || value.message;
  if (typeof value === "string") return value;
  try { return JSON.stringify(value) ?? String(value); }
  catch { return String(value); }
}

/** Keep browser diagnostics while forwarding desktop logs without recursive failures. */
export function forwardConsole(target: Pick<Console, ConsoleLevel>, sinks: Record<ConsoleLevel, LogSink>) {
  for (const level of Object.keys(sinks) as ConsoleLevel[]) {
    const original = target[level].bind(target);
    target[level] = (...values: unknown[]) => {
      original(...values);
      void sinks[level](values.map(format).join(" ")).catch(() => {});
    };
  }
}

let initialized = false;
export function initializeLogging() {
  if (initialized || !isTauri()) return;
  initialized = true;
  forwardConsole(console, { log: info, info, warn, error, debug });
  window.addEventListener("error", event => console.error(event.error ?? event.message));
  window.addEventListener("unhandledrejection", event => console.error(event.reason));
}
