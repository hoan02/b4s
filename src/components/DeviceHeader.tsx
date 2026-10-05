import { Component, createSignal, onCleanup, onMount, Show } from "solid-js";
import type { BatteryData } from "./Battery";
import { resolveDeviceImage } from "../lib/deviceImages";
import type { LinkHealth } from "../lib/ble";
import { t } from "../lib/i18n";

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

function pctClass(p: number | null) {
  if (p === null) return "unk";
  if (p <= 20) return "low";
  if (p <= 50) return "mid";
  return "ok";
}

function fmtPct(p: number | null) {
  if (p === null) return "—";
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
        return t("device.live");
      case "waiting":
        return t("device.waiting");
      case "demo":
        return t("device.demoStatus");
      case "dead":
        return t("device.lost");
      default:
        return isConnected() ? t("device.connected") : t("device.disconnected");
    }
  };

  const linkMessage = () => {
    const health = link();
    if (!health || level() === "offline") return t("linkMessage.offline");
    if (health.mock) return t("linkMessage.demo");
    if (!health.peripheralConnected) return t("linkMessage.dropped");
    if (!health.hasWriteUuid || !health.hasNotifyUuid) {
      return t("linkMessage.serviceMissing");
    }
    if (health.notifyCount > 0) {
      return t("linkMessage.live", {
        notifications: health.notifyCount,
        writes: health.txCount,
      });
    }
    if (health.handshakeOk) return t("linkMessage.waiting");
    return t("linkMessage.handshakeFailed");
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
          <div class="batt-tag">{t("device.left")}</div>
        </div>
        <div class="batt-cell">
          <div class={`batt-pct ${pctClass(props.battery.case)}`}>
            {fmtPct(props.battery.case)}
            {props.battery.caseCharging ? " ⚡" : ""}
          </div>
          <div class="batt-tag">{t("device.case")}</div>
        </div>
        <div class="batt-cell">
          <div class={`batt-pct ${pctClass(props.battery.right)}`}>
            {fmtPct(props.battery.right)}
            {props.battery.rightCharging ? " ⚡" : ""}
          </div>
          <div class="batt-tag">{t("device.right")}</div>
        </div>
      </div>

      <Show when={props.battery.left === null && props.battery.right === null}>
        <div class="battery-warn">
          {t("device.batteryMissing")}
        </div>
      </Show>

      <Show when={isConnected() && link()}>
        <button
          type="button"
          class={`link-banner level-${level()}`}
          onClick={() => setShowDiag(!showDiag())}
        >
          <span class="link-banner-title">
            {level() === "live" && `● ${t("device.receiving")}`}
            {level() === "waiting" && `◐ ${t("device.bleWaiting")}`}
            {level() === "demo" && `◇ ${t("device.demoHardware")}`}
            {level() === "dead" && `✕ ${t("device.controlError")}`}
            {level() === "offline" && `○ ${t("device.offline")}`}
          </span>
          <span class="link-banner-msg">{linkMessage()}</span>
          <span class="link-banner-hint">
            {showDiag() ? t("device.hideDetails") : t("device.connectionDetails")}
          </span>
        </button>
      </Show>

      <Show when={showDiag() && link()}>
        <div class="link-diag">
          <div class="link-row">
            <span>{t("device.mode")}</span>
            <strong class={link()!.mock ? "bad" : "ok"}>
              {link()!.mock ? "DEMO" : "REAL BLE"}
            </strong>
          </div>
          <div class="link-row">
            <span>{t("device.rx")}</span>
            <strong class={link()!.notifyCount > 0 ? "ok" : "warn"}>
              {link()!.notifyCount}
            </strong>
          </div>
          <div class="link-row">
            <span>{t("device.tx")}</span>
            <strong>{link()!.txCount}</strong>
          </div>
          <Show when={link()!.lastRxHex}>
            <div class="link-hex">
              <span>{t("device.lastRx")}</span>
              <code>{link()!.lastRxHex}</code>
            </div>
          </Show>
          <Show when={link()!.lastTxHex}>
            <div class="link-hex">
              <span>{t("device.lastTx")}</span>
              <code>{link()!.lastTxHex}</code>
            </div>
          </Show>
        </div>
      </Show>

      <div class="quick-actions">
        <button class="quick-btn" type="button" onClick={() => props.onFindBuds?.()}>
          <span class="quick-label">{t("device.find")}</span>
        </button>
        <button
          class="quick-btn danger"
          type="button"
          onClick={() => props.onDisconnect?.()}
        >
          <span class="quick-label">{t("home.disconnect")}</span>
        </button>
      </div>
    </div>
  );
};

export default DeviceHeader;
