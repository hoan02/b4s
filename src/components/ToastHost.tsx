import { Component, For, Show } from "solid-js";
import type { ToastItem } from "../lib/toast";

interface Props {
  items: ToastItem[];
  onDismiss: (id: number) => void;
}

const ToastHost: Component<Props> = (props) => (
  <div class="toast-host" aria-live="polite">
    <For each={props.items}>
      {(t) => (
        <button
          type="button"
          class={`toast-item toast-${t.kind}`}
          onClick={() => props.onDismiss(t.id)}
        >
          <Show when={t.title}>
            <strong class="toast-title">{t.title}</strong>
          </Show>
          <span class="toast-msg">{t.message}</span>
        </button>
      )}
    </For>
  </div>
);

export default ToastHost;
