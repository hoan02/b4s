/**
 * Device visuals — official app loads per-model photos from CDN.
 * APK only ships a generic ear placeholder (default_ear_pic.png).
 */

import defaultEar from "../assets/devices/default_ear_pic.png";
import { createResource, createSignal } from "solid-js";
import { convertFileSrc } from "@tauri-apps/api/core";
import { invoke, isTauri } from "./tauri";

const pending = new Map<string, Promise<string>>();
const resolved = new Map<string, string>();
const failedAt = new Map<string, number>();
const [thumbnails, setThumbnails] = createSignal<Record<string, string>>({});

export function registerDeviceImages(models: { imageUrl: string | null; presentation: {
  imageUrl: string | null; largeImageUrl: string | null;
} | null }[]) {
  const entries: Record<string, string> = {};
  for (const model of models) {
    if (model.imageUrl && model.imageUrl === model.presentation?.largeImageUrl && model.presentation.imageUrl) {
      entries[model.imageUrl] = model.presentation.imageUrl;
    }
  }
  setThumbnails(entries);
}

export async function loadDeviceImage(url: string): Promise<string> {
  if (!url.startsWith("https://") || !isTauri()) return url;
  const cached = resolved.get(url);
  if (cached) return cached;
  if (Date.now() - (failedAt.get(url) ?? 0) < 60_000) return defaultEar;
  const existing = pending.get(url);
  if (existing) return existing;
  const request = invoke<string>("cache_product_image", { url })
    .then(path => {
      const src = convertFileSrc(path);
      resolved.set(url, src);
      failedAt.delete(url);
      return src;
    })
    .catch(() => { failedAt.set(url, Date.now()); return defaultEar; })
    .finally(() => pending.delete(url));
  pending.set(url, request);
  return request;
}

/** Cache asynchronously; never let the previous model's photo bleed into a new selection. */
export function createDeviceVisual(url: () => string | null | undefined, name: () => string | null | undefined, thumbnail = false) {
  const requestedUrl = () => {
    const value = url()?.trim();
    return value && thumbnail ? thumbnails()[value] || value : value;
  };
  const [image] = createResource(() => requestedUrl() || null,
    async value => ({ url: value, src: await loadDeviceImage(value) }));
  return (): DeviceVisual => {
    const value = requestedUrl();
    const loaded = image();
    return {
      src: value && resolved.get(value) || (loaded && loaded.url === value ? loaded.src : value && !isTauri() ? value : defaultEar),
      alt: name()?.trim() || "Earbuds",
      sourceUrl: value,
    };
  };
}

export type DeviceVisual = {
  src: string;
  alt: string;
  sourceUrl?: string;
};

/** Generic earbud image asset. */
export function resolveDeviceImage(
  _modelId?: string | null,
  bleName?: string | null,
  imageUrl?: string | null
): DeviceVisual {
  return {
    src: imageUrl?.trim() || defaultEar,
    alt: bleName?.trim() || "Earbuds",
  };
}

/** Keep an unavailable CDN image from rendering a broken-image icon. */
export function handleDeviceImageError(event: Event & { currentTarget: HTMLImageElement }) {
  const url = event.currentTarget.dataset.imageUrl;
  if (url) resolved.delete(url);
  event.currentTarget.onerror = null;
  event.currentTarget.src = defaultEar;
}

export function resolveDeviceThumb(
  modelId?: string | null,
  bleName?: string | null,
  imageUrl?: string | null
): DeviceVisual {
  return resolveDeviceImage(modelId, bleName, imageUrl);
}
