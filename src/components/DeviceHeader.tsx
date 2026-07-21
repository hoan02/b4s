import { Component, createSignal, onCleanup, onMount, Show } from "solid-js";
import Battery, { BatteryData } from "./Battery";
import { resolveDeviceImage } from "../lib/deviceImages";

interface Props {
  name?: string;
  modelId?: string | null;
  connected?: boolean;
  battery: BatteryData;
  onFindBuds?: () => void;
  onDisconnect?: () => void;
}

const DeviceHeader: Component<Props> = (props) => {
  const [seconds, setSeconds] = createSignal(0);
  const [imgError, setImgError] = createSignal(false);
  let timer: number | undefined;

  onMount(() => {
    if (props.connected !== false) {
      timer = window.setInterval(() => setSeconds((s) => s + 1), 1000);
    }
  });

  onCleanup(() => {
    if (timer) clearInterval(timer);
  });

  const formatTime = (total: number) => {
    const h = Math.floor(total / 3600);
    const m = Math.floor((total % 3600) / 60);
    const s = total % 60;
    if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
    return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  };

  const isConnected = () => props.connected !== false;

  const visual = () =>
    resolveDeviceImage(props.modelId, props.name);

  return (
    <div class="device-header">
      <div class="device-image-wrapper">
        <Show
          when={!imgError()}
          fallback={
            <div class="device-placeholder">
              <div class="earbud left" />
              <div class="earbud right" />
            </div>
          }
        >
          <img
            class={`device-image style-${visual().style}`}
            src={visual().src}
            alt={visual().alt}
            draggable={false}
            onError={() => setImgError(true)}
          />
        </Show>
      </div>

      <h1 class="device-name">{props.name ?? "Baseus Bowie"}</h1>

      <div class="device-status">
        <span class={`status-dot ${isConnected() ? "" : "disconnected"}`} />
        <span>{isConnected() ? "Connected" : "Disconnected"}</span>
      </div>

      <Show when={isConnected()}>
        <div class="session-timer">Session · {formatTime(seconds())}</div>
      </Show>

      <Battery data={props.battery} />

      <div class="quick-actions">
        <button class="quick-btn" type="button" onClick={() => props.onFindBuds?.()}>
          <span class="quick-icon">🔔</span>
          <span class="quick-label">Find Buds</span>
        </button>
        <button class="quick-btn" type="button">
          <span class="quick-icon">👆</span>
          <span class="quick-label">Gestures</span>
        </button>
        <button
          class="quick-btn danger"
          type="button"
          onClick={() => props.onDisconnect?.()}
        >
          <span class="quick-icon">⏻</span>
          <span class="quick-label">Disconnect</span>
        </button>
      </div>
    </div>
  );
};

export default DeviceHeader;
