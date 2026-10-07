import { Show } from "solid-js";
import { t } from "../lib/i18n";

export default function OperationStatus(props: { pending?: boolean; error?: string | null }) {
  // Progress stays accessible without adding a row; controllers publish errors as toasts.
  return <span class="sr-only" role="status" aria-live="polite" aria-atomic="true">
    <Show when={props.pending}>{t("device.applying")}</Show>
  </span>;
}
