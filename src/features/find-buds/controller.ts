import { createSignal } from "solid-js";
import { findBuds } from "../../lib/device";
import { formatError, t } from "../../lib/i18n";
import type { ToastItem } from "../../lib/toast";

type Notify = (message: string, kind?: ToastItem["kind"], title?: string) => void;

export function createFindBudsController(notify: Notify) {
  const [active, setActive] = createSignal(false);
  const [confirmationOpen, setConfirmationOpen] = createSignal(false);
  const [dialogMode, setDialogMode] = createSignal<"confirm" | "active">("confirm");

  const start = async () => {
    try {
      await findBuds(true);
      setActive(true);
      setDialogMode("active");
      notify(t("toast.finding"), "info", t("toast.findingTitle"));
    } catch (error) {
      notify(formatError(error), "error");
    }
  };

  const stop = async (errorTitle?: string) => {
    try {
      await findBuds(false);
      setActive(false);
      setConfirmationOpen(false);
      setDialogMode("confirm");
      notify(t("toast.findStopped"), "info", t("toast.stopped"));
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
  };

  return {
    active,
    confirmationOpen,
    dialogMode,
    request,
    start,
    stop,
    reset,
    closeConfirmation: () => setConfirmationOpen(false),
  };
}
