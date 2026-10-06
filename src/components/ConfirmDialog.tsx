import { Component, onCleanup, onMount } from "solid-js";
import { t } from "../lib/i18n";

interface Props {
  title: string;
  message: string;
  onCancel: () => void;
  onEscape?: () => void;
  onConfirm: () => void;
  confirmLabel?: string;
  cancelLabel?: string;
  showCancel?: boolean;
}

const ConfirmDialog: Component<Props> = (props) => {
  let dialog: HTMLElement | undefined;
  let previousFocus: HTMLElement | null = null;

  onMount(() => {
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog?.querySelector<HTMLElement>("button:not([disabled])")?.focus();
  });

  onCleanup(() => {
    if (previousFocus?.isConnected) previousFocus.focus();
  });

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      (props.onEscape ?? props.onCancel)();
      return;
    }
    if (event.key !== "Tab" || !dialog) return;

    const buttons = [...dialog.querySelectorAll<HTMLButtonElement>("button:not([disabled])")];
    if (buttons.length === 0) {
      event.preventDefault();
      dialog.focus();
      return;
    }
    const first = buttons[0];
    const last = buttons[buttons.length - 1];
    if (event.shiftKey && (document.activeElement === first || document.activeElement === dialog)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  return (
    <div class="confirm-backdrop" role="presentation" onClick={(event) => event.currentTarget === event.target && props.onCancel()}>
      <section
        ref={dialog}
        class="confirm-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-title"
        aria-describedby="confirm-description"
        tabIndex={-1}
        onKeyDown={handleKeyDown}
      >
        <div class="confirm-dialog-icon" aria-hidden="true">♫</div>
        <h2 id="confirm-title">{props.title}</h2>
        <p id="confirm-description">{props.message}</p>
        <div class={`confirm-dialog-actions ${props.showCancel === false ? "single" : ""}`}>
          {props.showCancel !== false && <button type="button" class="confirm-btn secondary" onClick={props.onCancel}>{props.cancelLabel ?? t("dialog.cancel")}</button>}
          <button type="button" class="confirm-btn primary" onClick={props.onConfirm}>{props.confirmLabel ?? t("dialog.turnOffContinue")}</button>
        </div>
      </section>
    </div>
  );
};

export default ConfirmDialog;
