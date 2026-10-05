import { Component, For, Show, createSignal, onCleanup, onMount } from "solid-js";
import {
  BleDevice,
  startScan,
  stopScan,
  connect,
  onScanStatus,
  onConnecting,
  rssiToBars,
  checkAdapter,
  getScanStatus,
} from "../lib/ble";
import { findRememberedDevice, readRememberedDevice } from "../lib/reconnect";
import { resolveDeviceThumb } from "../lib/deviceImages";
import { formatError, t } from "../lib/i18n";

interface Props {
  onConnected: (device: BleDevice) => void;
  onOpenSettings?: () => void;
  appVersion?: string;
  autoReconnect: boolean;
  onAutoReconnectAttempt: () => void;
}

const BlePairing: Component<Props> = (props) => {
  const [scanning, setScanning] = createSignal(false);
  const [devices, setDevices] = createSignal<BleDevice[]>([]);
  const [connectingId, setConnectingId] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [adapterOk, setAdapterOk] = createSignal<boolean | null>(null);
  const [useMock, setUseMock] = createSignal(false);
  const savedDevice = props.autoReconnect ? readRememberedDevice() : null;
  const [autoSearching, setAutoSearching] = createSignal(!!savedDevice);
  const reconnectAbort = new AbortController();
  let reconnectTask: Promise<void> | undefined;

  let unsubs: Array<() => void> = [];
  let checkingAdapter = false;
  let disposed = false;
  let adapterPoll: number | undefined;
  let handleFocus: (() => void) | undefined;
  let handleVisibility: (() => void) | undefined;

  const refreshAdapter = async () => {
    if (useMock() || checkingAdapter || scanning() || autoSearching() || connectingId()) return;
    checkingAdapter = true;
    try {
      const ok = await checkAdapter();
      setAdapterOk(ok);
      if (!ok) {
        if (scanning()) await stopScan();
        setScanning(false);
        setDevices([]);
        setError(t("pair.bluetoothHint"));
      } else if (!scanning()) {
        setError(null);
      }
    } finally {
      checkingAdapter = false;
    }
  };

  onMount(() => {
    props.onAutoReconnectAttempt();
    handleFocus = () => { void refreshAdapter(); };
    handleVisibility = () => {
      if (document.visibilityState === "visible") void refreshAdapter();
    };
    window.addEventListener("focus", handleFocus);
    document.addEventListener("visibilitychange", handleVisibility);
    adapterPoll = window.setInterval(() => { void refreshAdapter(); }, 3000);
    void refreshAdapter();

    void (async () => {
      const scanUnsub = await onScanStatus((status) => {
        setScanning(status.scanning);
        setDevices(status.devices);
        if (status.error) setError(status.error);
      });
      if (disposed) scanUnsub();
      else {
        unsubs.push(scanUnsub);
        if (savedDevice) reconnectTask = runAutomaticReconnect();
      }
    })().catch((e) => {
      if (!disposed) {
        setAutoSearching(false);
        setError(formatError(e));
      }
    });

    void (async () => {
      const connectingUnsub = await onConnecting((id) => setConnectingId(id));
      if (disposed) connectingUnsub();
      else unsubs.push(connectingUnsub);
    })();
  });

  onCleanup(() => {
    disposed = true;
    reconnectAbort.abort();
    if (handleFocus) window.removeEventListener("focus", handleFocus);
    if (handleVisibility) {
      document.removeEventListener("visibilitychange", handleVisibility);
    }
    if (adapterPoll !== undefined) window.clearInterval(adapterPoll);
    unsubs.forEach((u) => u());
    stopScan().catch(() => {});
  });

  const runAutomaticReconnect = async () => {
    if (!savedDevice) return;
    try {
      const candidate = await findRememberedDevice(savedDevice, {
        checkAdapter, startScan, stopScan, getScanStatus, onScanStatus,
      }, reconnectAbort.signal);
      if (!disposed && !reconnectAbort.signal.aborted && candidate) {
        setAutoSearching(false);
        await handleConnect(candidate);
      }
    } catch (e) {
      if (!disposed && !reconnectAbort.signal.aborted) setError(formatError(e));
    } finally {
      if (!disposed) setAutoSearching(false);
    }
  };

  const cancelAutoReconnect = async () => {
    reconnectAbort.abort();
    await reconnectTask;
    setAutoSearching(false);
  };

  const handleScan = async () => {
    if (connectingId()) return;
    if (autoSearching()) await cancelAutoReconnect();
    setError(null);
    setDevices([]);
    if (!useMock()) {
      const ok = await checkAdapter();
      setAdapterOk(ok);
      if (!ok) {
        setError(t("pair.bluetoothHint"));
        return;
      }
    }
    try {
      await startScan(useMock());
    } catch (e) {
      setError(formatError(e));
    }
  };

  const enableDemo = async () => {
    if (connectingId()) return;
    if (autoSearching()) await cancelAutoReconnect();
    setUseMock(true);
    setError(null);
    setDevices([]);
    try {
      await startScan(true);
    } catch (e) {
      setError(formatError(e));
    }
  };

  const handleConnect = async (device: BleDevice) => {
    if (connectingId()) return;
    if (autoSearching()) await cancelAutoReconnect();
    if (disposed || connectingId()) return;
    setError(null);
    setConnectingId(device.id);
    try {
      const isMock = useMock() || device.id.startsWith("mock-");
      if (!isMock) {
        const ok = await checkAdapter();
        if (!ok) {
          setError(t("pair.bluetoothDisabled"));
          setConnectingId(null);
          return;
        }
      }
      props.onConnected(await connect(device.id, isMock));
    } catch (e) {
      setError(formatError(e));
      setConnectingId(null);
    }
  };

  const matched = () => {
    const list = devices().filter((d) => d.isBaseus);
    return [...list].sort((a, b) => {
      const rank = (s?: string | null) =>
        s === "verified" ? 0 : s === "experimental" ? 1 : 2;
      const r = rank(a.support) - rank(b.support);
      return r !== 0 ? r : b.rssi - a.rssi;
    });
  };
  const others = () => devices().filter((d) => !d.isBaseus);
  const hasDual = () => {
    const names = matched().map((d) => d.name.toLowerCase());
    return names.some((n, i) => names.indexOf(n) !== i);
  };

  return (
    <div class="pair-shell">
      {/* Scrollable middle */}
      <div class="pair-scroll">
        <div class="ble-pairing">
          <div class={`ble-hero ${adapterOk() === false && !useMock() ? "bluetooth-off" : ""}`}>
            <img class="ble-app-logo" src="/b4s-logo.png" alt="B4S" />
            <h2>{t("pair.title")}</h2>
            <p class="ble-subtitle">
              {adapterOk() === false && !useMock()
                ? t("pair.bluetoothOff")
                : useMock()
                  ? t("pair.demoSub")
                  : autoSearching()
                    ? t("pair.reconnecting", { name: savedDevice?.name })
                    : t("pair.nearby")}
            </p>
          </div>

          <div class={`ble-controls ${adapterOk() === false && !useMock() ? "bluetooth-off" : ""}`}>
            <Show
              when={!scanning()}
              fallback={
                <button
                  class="ble-btn secondary"
                  type="button"
                  disabled={!!connectingId()}
                  onClick={() => autoSearching() ? cancelAutoReconnect() : stopScan()}
                >
                  <span class="spinner" />
                  {t("pair.stop")}
                </button>
              }
            >
              <button class="ble-btn primary" type="button" disabled={!!connectingId() || autoSearching()} onClick={handleScan}>
                {t("pair.scan")}
              </button>
            </Show>
          </div>

          <Show when={error()}>
            <div class="ble-error">{error()}</div>
          </Show>

          <Show when={adapterOk() === false && !useMock()}>
            <div class="ble-bt-off">
              <p>{t("pair.bluetoothWarning")}</p>
              <button class="ble-btn secondary" type="button" onClick={enableDemo}>
                {t("pair.openDemo")}
              </button>
            </div>
          </Show>

          <Show when={hasDual() && !useMock()}>
            <p class="ble-tip">
              {t("pair.dualHint")}
            </p>
          </Show>

          <div class="ble-list">
            <Show when={matched().length > 0}>
              <div class="ble-group-label">
                {useMock() ? t("pair.demo") : t("pair.devices")}
              </div>
              <For each={matched()}>
                {(device) => (
                  <DeviceRow
                    device={device}
                    connecting={connectingId() === device.id}
                    onConnect={() => handleConnect(device)}
                  />
                )}
              </For>
            </Show>

            <Show when={others().length > 0 && !useMock()}>
              <div class="ble-group-label">{t("pair.other")}</div>
              <For each={others()}>
                {(device) => (
                  <DeviceRow
                    device={device}
                    connecting={connectingId() === device.id}
                    onConnect={() => handleConnect(device)}
                  />
                )}
              </For>
            </Show>

            <Show
              when={
                !scanning() &&
                devices().length === 0 &&
                !error() &&
                adapterOk() !== false
              }
            >
              <div class="ble-empty">
                <p>{t("pair.empty")}</p>
                <span>{t("pair.emptyHint")}</span>
              </div>
            </Show>

            <Show when={scanning() && devices().length === 0}>
              <div class="ble-empty scanning">
                <div class="ble-scan-status" role="status" aria-live="polite">
                  <span class="scan-bars" aria-hidden="true"><i /><i /><i /></span>
                  <span>{t("pair.scanning")}</span>
                </div>
                <p>{t("pair.searching")}</p>
                <span>{t("pair.scanHint")}</span>
              </div>
            </Show>
          </div>
        </div>
      </div>

      {/* Fixed footer */}
      <footer class="pair-footer">
        <button
          type="button"
          class="pair-footer-settings"
          onClick={() => props.onOpenSettings?.()}
        >
          {t("nav.settings")}
        </button>
        <span class="pair-footer-ver">B4S v{props.appVersion ?? "…"}</span>
      </footer>
    </div>
  );
};

const DeviceRow: Component<{
  device: BleDevice;
  connecting: boolean;
  onConnect: () => void;
}> = (props) => {
  const bars = () => rssiToBars(props.device.rssi);
  const title = () => props.device.modelName || props.device.name;

  return (
    <button
      class={`ble-device ${props.device.isBaseus ? "matched" : ""} ${props.connecting ? "connecting" : ""}`}
      type="button"
      disabled={props.connecting}
      onClick={() => props.onConnect()}
    >
      <div class="device-icon photo">
        <img
          src={resolveDeviceThumb(props.device.modelId, props.device.name, props.device.imageUrl).src}
          alt=""
          draggable={false}
        />
      </div>
      <div class="device-info">
        <div class="device-name-row">
          <span class="name" title={title()}>
            {title()}
          </span>
          <Show when={props.device.support === "verified"}>
            <span class="tag ok">{t("pair.verified")}</span>
          </Show>
          <Show when={props.device.support === "experimental"}>
            <span class="tag">{t("pair.experimental")}</span>
          </Show>
        </div>
        <div class="device-meta">
          <span class="rssi">
            <SignalBars level={bars()} />
            {props.device.rssi}
          </span>
          <Show when={props.device.hint}>
            <span class="hint-inline">{t("pair.deviceCount", { count: 2 })}</span>
          </Show>
        </div>
      </div>
      <div class="device-action">
        <Show when={!props.connecting} fallback={<span class="spinner small" />}>
            <span class="connect-label">{t("pair.connect")}</span>
        </Show>
      </div>
    </button>
  );
};

const SignalBars: Component<{ level: number }> = (props) => (
  <span class="signal-bars" aria-hidden="true">
    {[1, 2, 3, 4].map((i) => (
      <span class={`bar ${i <= props.level ? "on" : ""}`} />
    ))}
  </span>
);

export default BlePairing;
