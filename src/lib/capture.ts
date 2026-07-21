/**
 * Capture Studio API
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export type Direction = "tx" | "rx" | "info";

export interface CaptureEntry {
  id: number;
  tsMs: number;
  direction: Direction;
  hex: string;
  raw: number[];
  label: string | null;
  decoded: string | null;
  charUuid: string | null;
}

export interface GuidedStep {
  id: string;
  title: string;
  instruction: string;
  done: boolean;
  entryIds: number[];
}

export interface GattChar {
  uuid: string;
  properties: string[];
  serviceUuid: string;
}

export interface CaptureSession {
  active: boolean;
  startedAtMs: number | null;
  deviceName: string | null;
  deviceAddress: string | null;
  entries: CaptureEntry[];
  steps: GuidedStep[];
  gattMap: GattChar[];
  currentStepId: string | null;
}

export interface CaptureBundle {
  version: number;
  exportedAtMs: number;
  deviceName: string | null;
  deviceAddress: string | null;
  gattMap: GattChar[];
  entries: CaptureEntry[];
  steps: GuidedStep[];
  notes: string;
}

// Commands
export const startCapture = (deviceName?: string, deviceAddress?: string) =>
  invoke<CaptureSession>("capture_start", { deviceName, deviceAddress });

export const stopCapture = () => invoke<CaptureSession>("capture_stop");

export const clearCapture = () => invoke<CaptureSession>("capture_clear");

export const getCapture = () => invoke<CaptureSession>("capture_get");

export const setStep = (stepId: string) =>
  invoke("capture_set_step", { stepId });

export const markStep = (stepId: string, done: boolean) =>
  invoke("capture_mark_step", { stepId, done });

export const annotateEntry = (entryId: number, label: string) =>
  invoke("capture_annotate", { entryId, label });

export const addNote = (text: string) =>
  invoke("capture_add_note", { text });

export const exportJson = (notes?: string) =>
  invoke<CaptureBundle>("capture_export_json", { notes });

export const exportMarkdown = () =>
  invoke<string>("capture_export_markdown");

export const writeHex = (hex: string) =>
  invoke("capture_write_hex", { hex });

// Events
export const onCaptureSession = (cb: (s: CaptureSession) => void): Promise<UnlistenFn> =>
  listen<CaptureSession>("capture://session", (e) => cb(e.payload));

export const onCaptureEntry = (cb: (e: CaptureEntry) => void): Promise<UnlistenFn> =>
  listen<CaptureEntry>("capture://entry", (e) => cb(e.payload));

// Download helpers (browser-side)
export function downloadText(filename: string, content: string, mime = "text/plain") {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export function downloadJson(filename: string, data: unknown) {
  downloadText(filename, JSON.stringify(data, null, 2), "application/json");
}
