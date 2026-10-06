/**
 * App metadata + auto-update helpers
 */

import { invoke } from "./tauri";

export interface AppInfo {
  name: string;
  version: string;
  identifier: string;
  tauriVersion: string;
  os: string;
  debug: boolean;
}

export interface UpdateCheckResult {
  available: boolean;
  currentVersion: string;
  version?: string | null;
  body?: string | null;
  date?: string | null;
  error?: string | null;
}

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function getStartAtLogin(): Promise<boolean> {
  return invoke<boolean>("get_start_at_login");
}

export async function setStartAtLogin(enabled: boolean): Promise<void> {
  await invoke("set_start_at_login", { enabled });
}

export async function setExperimentalMode(enabled: boolean): Promise<void> {
  await invoke("set_experimental_mode", { enabled });
}

export async function checkForUpdates(): Promise<UpdateCheckResult> {
  return invoke<UpdateCheckResult>("check_for_updates");
}

export async function installUpdate(): Promise<void> {
  await invoke("install_update");
}

export async function openExternal(url: string): Promise<void> {
  try {
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
  } catch {
    window.open(url, "_blank");
  }
}
