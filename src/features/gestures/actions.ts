/**
 * Source-traced gesture layout and function labels (Baseus 2.17.0.1).
 * `layout` is the wire click byte; `functions` are the wire function IDs
 * exposed by the reviewed profile. Labels resolve through i18n.
 */

export const GESTURE_LAYOUT_LABELS: Record<number, string> = {
  0: "gesture.doubleClick",
  1: "gesture.tripleClick",
  2: "gesture.longPress",
  3: "gesture.singleClick",
  4: "gesture.singlePress",
  5: "gesture.pentaClick",
};

export const GESTURE_FUNCTION_LABELS: Record<number, string> = {
  0: "gesture.fn.none",
  1: "gesture.fn.playPause",
  2: "gesture.fn.previous",
  3: "gesture.fn.next",
  4: "gesture.fn.voiceAssistant",
  5: "gesture.fn.lowLatency",
  6: "gesture.fn.noiseControl",
  7: "gesture.fn.spatial",
  8: "gesture.fn.gameSound",
  9: "gesture.fn.rapidMode",
  10: "gesture.fn.lightEffect",
  11: "gesture.fn.volumeUp",
  12: "gesture.fn.volumeDown",
  13: "gesture.fn.bassBoost",
  14: "gesture.fn.sleepHelper",
  15: "gesture.fn.listenXimalaya",
  16: "gesture.fn.dynamicSound",
  17: "gesture.fn.awakenAi",
  18: "gesture.fn.quickPhoto",
  19: "gesture.fn.listenXimalaya",
  27: "gesture.fn.switchConnection",
  28: "gesture.fn.microphone",
};

export function gestureLayoutLabelKey(layout: number): string {
  return GESTURE_LAYOUT_LABELS[layout] ?? "gesture.unknownLayout";
}

export function gestureFunctionLabelKey(functionId: number): string {
  return GESTURE_FUNCTION_LABELS[functionId] ?? "gesture.unknownFunction";
}
