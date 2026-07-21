/**
 * Product images for Baseus models (mobile-app style).
 * Maps modelId / BLE name → asset URL.
 */

// Vite will hash & bundle these
import imgBp1 from "../assets/devices/bass-bp1-pro.jpg";
import imgBp1White from "../assets/devices/bass-bp1-pro-white.jpg";
import imgBp1Buds from "../assets/devices/bass-bp1-pro-buds.jpg";
import imgMa10 from "../assets/devices/bowie-ma10.jpg";
import imgM2 from "../assets/devices/bowie-m2.jpg";
import imgM2sPro from "../assets/devices/bowie-m2s-pro.jpg";
import imgE3 from "../assets/devices/bowie-e3.jpg";
import imgMc2 from "../assets/devices/bowie-mc2.jpg";
import imgInspire from "../assets/devices/inspire-xp1.jpg";
import imgInspireBuds from "../assets/devices/inspire-xp1-buds.jpg";

export type DeviceVisual = {
  src: string;
  alt: string;
  /** stem | bean | open | case | headset */
  style: "stem" | "bean" | "open" | "case" | "headset";
};

/** Primary image per model id */
const BY_ID: Record<string, DeviceVisual> = {
  "bass-bp1-pro": { src: imgBp1, alt: "Bass BP1 Pro", style: "stem" },
  "bowie-ma10": { src: imgMa10, alt: "Bowie MA10", style: "bean" },
  "bowie-ma10s": { src: imgMa10, alt: "Bowie MA10s", style: "bean" },
  "bowie-ma10-pro": { src: imgMa10, alt: "Bowie MA10 Pro", style: "bean" },
  "bowie-ma20": { src: imgMa10, alt: "Bowie MA20", style: "bean" },
  "bowie-ma20-pro": { src: imgMa10, alt: "Bowie MA20 Pro", style: "bean" },
  "bowie-m2": { src: imgM2, alt: "Bowie M2", style: "stem" },
  "bowie-m2-plus": { src: imgM2, alt: "Bowie M2+", style: "stem" },
  "bowie-m2s": { src: imgM2sPro, alt: "Bowie M2s", style: "stem" },
  "bowie-m2s-pro": { src: imgM2sPro, alt: "Bowie M2s Pro", style: "stem" },
  "bowie-m2s-ultra": { src: imgM2sPro, alt: "M2s Ultra", style: "stem" },
  "bowie-m3": { src: imgM2sPro, alt: "Bowie M3", style: "stem" },
  "bowie-m1": { src: imgM2, alt: "Bowie M1", style: "stem" },
  "bowie-e3": { src: imgE3, alt: "Bowie E3", style: "case" },
  "bowie-e2": { src: imgE3, alt: "Bowie E2", style: "case" },
  "bowie-e5": { src: imgE3, alt: "Bowie E5", style: "case" },
  "bowie-e5x": { src: imgE3, alt: "Bowie E5x", style: "case" },
  "bowie-e8": { src: imgE3, alt: "Bowie E8", style: "case" },
  "bowie-e10": { src: imgE3, alt: "Bowie E10", style: "case" },
  "bowie-e12": { src: imgE3, alt: "Bowie E12", style: "case" },
  "bowie-e13": { src: imgE3, alt: "Bowie E13", style: "case" },
  "baseus-e9": { src: imgE3, alt: "Baseus E9", style: "case" },
  "bowie-ex": { src: imgE3, alt: "Bowie EX", style: "case" },
  "bowie-mc1": { src: imgMc2, alt: "Bowie MC1", style: "open" },
  "bowie-mc2": { src: imgMc2, alt: "Bowie MC2", style: "open" },
  "bowie-mf1": { src: imgMc2, alt: "Bowie MF1", style: "open" },
  "airgo-as01": { src: imgMc2, alt: "AirGo AS01", style: "open" },
  "airgo-1-ring": { src: imgMc2, alt: "AirGo 1 Ring", style: "open" },
  "airgo-ag20": { src: imgMc2, alt: "AirGo AG20", style: "open" },
  "inspire-xp1": { src: imgInspire, alt: "Inspire XP1", style: "stem" },
  "inspire-xh1": { src: imgInspireBuds, alt: "Inspire XH1", style: "stem" },
  "inspire-xc1": { src: imgInspire, alt: "Inspire XC1", style: "stem" },
  "bass-bd1": { src: imgBp1White, alt: "Bass BD1", style: "stem" },
  "bass-1-plus": { src: imgBp1Buds, alt: "Bass 1+", style: "stem" },
  // WM / W bean-ish
  "bowie-wm01": { src: imgMa10, alt: "WM01", style: "bean" },
  "bowie-wm01-plus": { src: imgMa10, alt: "WM01 Plus", style: "bean" },
  "bowie-wm02": { src: imgMa10, alt: "WM02", style: "bean" },
  "bowie-wm02-plus": { src: imgMa10, alt: "WM02+", style: "bean" },
  "bowie-wm03": { src: imgMa10, alt: "WM03", style: "bean" },
  "bowie-wm05": { src: imgMa10, alt: "WM05", style: "bean" },
  "bowie-w04": { src: imgMa10, alt: "W04", style: "bean" },
  "bowie-w04-pro": { src: imgMa10, alt: "W04 Pro", style: "bean" },
  "bowie-w04-plus": { src: imgMa10, alt: "W04 Plus", style: "bean" },
  "bowie-wx5": { src: imgMa10, alt: "WX5", style: "bean" },
  "bowie-mz10": { src: imgMa10, alt: "MZ10", style: "bean" },
  "bowie-ez10": { src: imgMa10, alt: "EZ10", style: "bean" },
  "encok-w04": { src: imgMa10, alt: "Encok W04", style: "bean" },
  "encok-w04-pro": { src: imgMa10, alt: "Encok W04 Pro", style: "bean" },
  "encok-w05-lite": { src: imgMa10, alt: "Encok W05", style: "bean" },
  "encok-w11": { src: imgMa10, alt: "Encok W11", style: "bean" },
  "encok-w12": { src: imgMa10, alt: "Encok W12", style: "bean" },
  // AirNora / Bowie 30
  "airnora": { src: imgBp1, alt: "AirNora", style: "stem" },
  "airnora-2": { src: imgBp1, alt: "AirNora 2", style: "stem" },
  "airnora-3": { src: imgBp1, alt: "AirNora 3", style: "stem" },
  "bowie-30": { src: imgBp1White, alt: "Bowie 30", style: "stem" },
  "bowie-35": { src: imgBp1White, alt: "Bowie 35", style: "stem" },
  "storm-1": { src: imgM2, alt: "Storm 1", style: "stem" },
  "storm-3": { src: imgM2, alt: "Storm 3", style: "stem" },
  // Headset → use stem large as stand-in
  "bowie-h1": { src: imgInspire, alt: "Bowie H1", style: "headset" },
  "bowie-h1-pro": { src: imgInspire, alt: "Bowie H1 Pro", style: "headset" },
  "bowie-h1s": { src: imgInspire, alt: "Bowie H1s", style: "headset" },
  "bowie-h2": { src: imgInspire, alt: "Bowie H2", style: "headset" },
  "bowie-30-max": { src: imgInspire, alt: "Bowie 30 Max", style: "headset" },
  "bowie-10-max": { src: imgInspire, alt: "Bowie 10 Max", style: "headset" },
};

/** Fallback by BLE name keywords */
const BY_NAME_HINT: { test: RegExp; visual: DeviceVisual }[] = [
  { test: /bp1|bass/i, visual: { src: imgBp1, alt: "Baseus Bass", style: "stem" } },
  { test: /ma10|ma20/i, visual: { src: imgMa10, alt: "Bowie MA", style: "bean" } },
  { test: /m2s|m3/i, visual: { src: imgM2sPro, alt: "Bowie M", style: "stem" } },
  { test: /m2|m1/i, visual: { src: imgM2, alt: "Bowie M", style: "stem" } },
  { test: /e[0-9]|ex\b/i, visual: { src: imgE3, alt: "Bowie E", style: "case" } },
  { test: /mc|mf|airgo|open/i, visual: { src: imgMc2, alt: "Open-ear", style: "open" } },
  { test: /inspire|xp1|xh1|xc1/i, visual: { src: imgInspire, alt: "Inspire", style: "stem" } },
  { test: /wm|w0|wx|mz|ez|encok/i, visual: { src: imgMa10, alt: "Baseus", style: "bean" } },
];

const DEFAULT: DeviceVisual = {
  src: imgBp1,
  alt: "Baseus earbuds",
  style: "stem",
};

export function resolveDeviceImage(
  modelId?: string | null,
  bleName?: string | null
): DeviceVisual {
  if (modelId && BY_ID[modelId]) {
    return BY_ID[modelId];
  }
  const name = bleName ?? "";
  for (const { test, visual } of BY_NAME_HINT) {
    if (test.test(name)) return visual;
  }
  return DEFAULT;
}

/** Thumbnail (same asset, UI scales down) */
export function resolveDeviceThumb(
  modelId?: string | null,
  bleName?: string | null
): DeviceVisual {
  return resolveDeviceImage(modelId, bleName);
}
