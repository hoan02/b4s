import { createSignal } from "solid-js";
import { findBuds } from "../../lib/device";
import type { FindSide } from "../../lib/device";
import { formatError, t } from "../../lib/i18n";
import type { ToastItem } from "../../lib/toast";

type Notify = (message: string, kind?: ToastItem["kind"], title?: string) => void;

export function createFindBudsController(notify: Notify) {
  const [active, setActive] = createSignal(false);
  const [confirmationOpen, setConfirmationOpen] = createSignal(false);
  const [dialogMode, setDialogMode] = createSignal<"confirm" | "active">("confirm");
  // The app stops the alert on the same side it started (BA10 <side> 00).
  const [side, setSide] = createSignal<FindSide>("both");

  const start = async () => {
    try {
      const result = await findBuds(true, side());
      setActive(true);
      setDialogMode("active");
      notify(
        t(result.disposition === "simulated" ? "toast.findSimulated" : "toast.finding"),
        "info",
        t("toast.findingTitle")
      );
    } catch (error) {
      notify(formatError(error), "error");
    }
  };

  const stop = async (errorTitle?: string) => {
    try {
      const result = await findBuds(false, side());
      setActive(false);
      setConfirmationOpen(false);
      setDialogMode("confirm");
      notify(
        t(result.disposition === "simulated" ? "toast.findStopSimulated" : "toast.findStopped"),
        "info",
        t("toast.stopped")
      );
    } catch (error) {
      notify(formatError(error), "error", errorTitle);
    }
  };

  const request = async () => {
    if (!active()) {
      setDialogMode("confirm");
      setConfirmationOpen(true);
      return;
    }
    await stop(t("home.find"));
  };

  const reset = () => {
    setActive(false);
    setConfirmationOpen(false);
    setDialogMode("confirm");
    setSide("both");
  };

  return {
    active,
    confirmationOpen,
    dialogMode,
    side,
    /** Side can only change before the alert starts. */
    selectSide: (value: FindSide) => {
      if (!active()) setSide(value);
    },
    request,
    start,
    stop,
    reset,
    closeConfirmation: () => setConfirmationOpen(false),
  };
}
