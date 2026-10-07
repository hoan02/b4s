/**
 * Safe Tauri helpers - no-op outside the Tauri webview
 * (e.g. plain `vite` / browser preview).
 */

import { invoke as tauriInvoke, isTauri as hasTauri } from "@tauri-apps/api/core";
import { listen as tauriListen, type UnlistenFn, type EventCallback, type EventName } from "@tauri-apps/api/event";

/** True when running inside a Tauri webview with IPC available. */
export function isTauri(): boolean {
  return typeof window !== "undefined" && hasTauri();
}

const noopUnlisten: UnlistenFn = () => {};

/**
 * Invoke a Tauri command. Throws if not in Tauri (callers should catch)
 * or if the command fails.
 */
export async function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (!isTauri()) {
    throw new Error(`Tauri unavailable (invoke "${cmd}")`);
  }
  return tauriInvoke<T>(cmd, args);
}

/**
 * Listen to a Tauri event. Returns a no-op unlisten when not in Tauri
 * so UI can still mount in the browser.
 */
export async function listen<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return noopUnlisten;
  }
  try {
    return await tauriListen<T>(event, handler);
  } catch (e) {
    console.warn(`[tauri] listen("${event}") failed:`, e);
    return noopUnlisten;
  }
}

export type { UnlistenFn };
