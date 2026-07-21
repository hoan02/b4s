import { Component, createSignal, onCleanup, onMount, Show } from "solid-js";
import type { BatteryData } from "./Battery";
import { resolveDeviceImage } from "../lib/deviceImages";
import type { LinkHealth } from "../lib/ble";

interface Props {
  name?: string;
  modelId?: string | null;
  connected?: boolean;
  battery: BatteryData;
  link?: LinkHealth | null;
  rssi?: number;
  onFindBuds?: () => void;
  onDisconnect?: () => void;
}

function pctClass(p: number) {
  if (p <= 0) return "unk";
  if (p <= 20) return "low";
  if (p <= 50) return "mid";
  return "ok";
}

function fmtPct(p: number) {
  if (p <= 0) return "—";
  return `${Math.min(100, p)}%`;
}

function signalBars(link?: LinkHealth | null, rssi?: number): number {
  if (link?.level === "live") return 4;
  if (link?.level === "waiting") return 2;
  if (link?.level === "demo") return 3;
  if (link?.level === "dead") return 1;
  if (rssi != null) {
    if (rssi >= -50) return 4;
    if (rssi >= -60) return 3;
    if (rssi >= -70) return 2;
    return 1;
  }
  return 0;
}

const DeviceHeader: Component<Props> = (props) => {
  const [seconds, setSeconds] = createSignal(0);
  const [imgError, setImgError] = createSignal(false);
  const [showDiag, setShowDiag] = createSignal(false);
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
    const m = Math.floor(total / 60);
    const s = total % 60;
    return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  };

  const isConnected = () => props.connected !== false;
  const link = () => props.link;
  const level = () =>
    (link()?.level ?? (isConnected() ? "waiting" : "offline")).toString();
  const bars = () => signalBars(link(), props.rssi);
  const visual = () => resolveDeviceImage(props.modelId, props.name);

  const statusLabel = () => {
    switch (level()) {
      case "live":
        return "Đang kết nối · Live";
      case "waiting":
        return "Đã ghép · chờ dữ liệu";
      case "demo":
        return "Demo (không phải tai nghe thật)";
      case "dead":
        return "Mất liên kết điều khiển";
      default:
        return isConnected() ? "Đã kết nối" : "Ngắt kết nối";
    }
  };

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
            class="device-image"
            src={visual().src}
            alt={visual().alt}
            draggable={false}
            onError={() => setImgError(true)}
          />
        </Show>
      </div>

      <h1 class="device-name">{props.name ?? "Baseus"}</h1>

      <div class="device-signal">
        <span class="sig-bars" aria-hidden="true">
          {[1, 2, 3, 4].map((i) => (
            <span class={i <= bars() ? "on" : ""} />
          ))}
        </span>
        <span class={`status-dot level-${level()}`} />
        <span>{statusLabel()}</span>
        <Show when={isConnected()}>
          <span>· {formatTime(seconds())}</span>
        </Show>
      </div>

      {/* L / Case / R — Baseus app style */}
      <div class="device-batt-strip">
        <div class="batt-cell">
          <div class={`batt-pct ${pctClass(props.battery.left)}`}>
            {fmtPct(props.battery.left)}
            {props.battery.leftCharging ? " ⚡" : ""}
          </div>
          <div class="batt-tag">Trái</div>
        </div>
        <div class="batt-cell">
          <div class={`batt-pct ${pctClass(props.battery.case)}`}>
            {fmtPct(props.battery.case)}
            {props.battery.caseCharging ? " ⚡" : ""}
          </div>
          <div class="batt-tag">Hộp</div>
        </div>
        <div class="batt-cell">
          <div class={`batt-pct ${pctClass(props.battery.right)}`}>
            {fmtPct(props.battery.right)}
            {props.battery.rightCharging ? " ⚡" : ""}
          </div>
          <div class="batt-tag">Phải</div>
        </div>
      </div>

      <Show when={props.battery.left === 0 && props.battery.right === 0}>
        <div class="battery-warn">
          Chưa nhận % pin — mở nắp hộp / đeo tai nghe để thiết bị gửi lại trạng thái
        </div>
      </Show>

      <Show when={isConnected() && link()}>
        <button
          type="button"
          class={`link-banner level-${level()}`}
          onClick={() => setShowDiag(!showDiag())}
        >
          <span class="link-banner-title">
            {level() === "live" && "● Đang nhận dữ liệu từ tai nghe"}
            {level() === "waiting" && "◐ BLE OK — chờ notify/pin"}
            {level() === "demo" && "◇ Demo — không phải hardware"}
            {level() === "dead" && "✕ Link điều khiển lỗi"}
            {level() === "offline" && "○ Offline"}
          </span>
          <span class="link-banner-msg">{link()!.message}</span>
          <span class="link-banner-hint">
            {showDiag() ? "Ẩn chi tiết ▴" : "Chi tiết kết nối ▾"}
          </span>
        </button>
      </Show>

      <Show when={showDiag() && link()}>
        <div class="link-diag">
          <div class="link-row">
            <span>Mode</span>
            <strong class={link()!.mock ? "bad" : "ok"}>
              {link()!.mock ? "DEMO" : "REAL BLE"}
            </strong>
          </div>
          <div class="link-row">
            <span>RX notifies</span>
            <strong class={link()!.notifyCount > 0 ? "ok" : "warn"}>
              {link()!.notifyCount}
            </strong>
          </div>
          <div class="link-row">
            <span>TX writes</span>
            <strong>{link()!.txCount}</strong>
          </div>
          <Show when={link()!.lastRxHex}>
            <div class="link-hex">
              <span>Last RX</span>
              <code>{link()!.lastRxHex}</code>
            </div>
          </Show>
          <Show when={link()!.lastTxHex}>
            <div class="link-hex">
              <span>Last TX</span>
              <code>{link()!.lastTxHex}</code>
            </div>
          </Show>
        </div>
      </Show>

      <div class="quick-actions">
        <button class="quick-btn" type="button" onClick={() => props.onFindBuds?.()}>
          <span class="quick-label">Tìm tai nghe</span>
        </button>
        <button
          class="quick-btn danger"
          type="button"
          onClick={() => props.onDisconnect?.()}
        >
          <span class="quick-label">Ngắt kết nối</span>
        </button>
      </div>
    </div>
  );
};

export default DeviceHeader;
