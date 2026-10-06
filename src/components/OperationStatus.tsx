import { Show } from "solid-js";
import { t } from "../lib/i18n";

export default function OperationStatus(props: { pending?: boolean; error?: string | null }) {
  return <>
    <Show when={props.pending}><span role="status">{t("device.applying")}</span></Show>
    <Show when={props.error}><span role="alert">{props.error}</span></Show>
  </>;
}
