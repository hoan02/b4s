export type ToastKind = "info" | "success" | "warn" | "error";

export interface ToastItem {
  id: number;
  kind: ToastKind;
  title?: string;
  message: string;
}

let seq = 1;

export function makeToast(
  message: string,
  kind: ToastKind = "info",
  title?: string
): ToastItem {
  return { id: seq++, kind, message, title };
}
