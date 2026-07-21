/**
 * Device visuals — official app loads per-model photos from CDN.
 * APK only ships a generic ear placeholder (default_ear_pic.png).
 */

import defaultEar from "../assets/devices/default_ear_pic.png";

export type DeviceVisual = {
  src: string;
  alt: string;
};

/** Generic earbud image asset. */
export function resolveDeviceImage(
  _modelId?: string | null,
  bleName?: string | null
): DeviceVisual {
  return {
    src: defaultEar,
    alt: bleName?.trim() || "Earbuds",
  };
}

export function resolveDeviceThumb(
  modelId?: string | null,
  bleName?: string | null
): DeviceVisual {
  return resolveDeviceImage(modelId, bleName);
}
