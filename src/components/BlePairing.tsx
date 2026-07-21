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
} from "../lib/ble";
import { resolveDeviceThumb } from "../lib/deviceImages";

interface Props {
  onConnected: (device: BleDevice) => void;
  onOpenSettings?: () => void;
  appVersion?: string;
}

const BlePairing: Component<Props> = (props) => {
  const [scanning, setScanning] = createSignal(false);
  const [devices, setDevices] = createSignal<BleDevice[]>([]);
  const [connectingId, setConnectingId] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [adapterOk, setAdapterOk] = createSignal<boolean | null>(null);
  const [useMock, setUseMock] = createSignal(false);

  let unsubs: Array<() => void> = [];
  let checkingAdapter = false;
  let disposed = false;
  let adapterPoll: number | undefined;
  let handleFocus: (() => void) | undefined;
  let handleVisibility: (() => void) | undefined;

  const refreshAdapter = async () => {
    if (useMock() || checkingAdapter) return;
    checkingAdapter = true;
    try {
      const ok = await checkAdapter();
      setAdapterOk(ok);
      if (!ok) {
        if (scanning()) await stopScan();
        setScanning(false);
        setDevices([]);
        setError("Bật Bluetooth rồi quét lại.");
      } else if (!scanning()) {
        setError(null);
      }
    } finally {
      checkingAdapter = false;
    }
  };

  onMount(() => {
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
      else unsubs.push(scanUnsub);
    })();

    void (async () => {
      const connectingUnsub = await onConnecting((id) => setConnectingId(id));
      if (disposed) connectingUnsub();
      else unsubs.push(connectingUnsub);
    })();
  });

  onCleanup(() => {
    disposed = true;
    if (handleFocus) window.removeEventListener("focus", handleFocus);
    if (handleVisibility) {
      document.removeEventListener("visibilitychange", handleVisibility);
    }
    if (adapterPoll !== undefined) window.clearInterval(adapterPoll);
    unsubs.forEach((u) => u());
    stopScan().catch(() => {});
  });

  const handleScan = async () => {
    setError(null);
    setDevices([]);
    if (!useMock()) {
      const ok = await checkAdapter();
      setAdapterOk(ok);
      if (!ok) {
        setError("Bật Bluetooth rồi quét lại.");
        return;
      }
    }
    try {
      await startScan(useMock());
    } catch (e) {
      setError(String(e));
    }
  };

  const enableDemo = async () => {
    setUseMock(true);
    setError(null);
    setDevices([]);
    try {
      await startScan(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleConnect = async (device: BleDevice) => {
    if (connectingId()) return;
    setError(null);
    setConnectingId(device.id);
    try {
      const isMock = useMock() || device.id.startsWith("mock-");
      if (!isMock) {
        const ok = await checkAdapter();
        if (!ok) {
          setError("Bluetooth đang tắt.");
          setConnectingId(null);
          return;
        }
      }
      props.onConnected(await connect(device.id, isMock));
    } catch (e) {
      setError(String(e));
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
            <h2>Kết nối</h2>
            <p class="ble-subtitle">
              {adapterOk() === false && !useMock()
                ? "Bật Bluetooth để quét"
                : useMock()
                  ? "Demo — thiết bị giả"
                  : "Tai nghe gần / mở nắp hộp"}
            </p>
          </div>

          <div class={`ble-controls ${adapterOk() === false && !useMock() ? "bluetooth-off" : ""}`}>
            <Show
              when={!scanning()}
              fallback={
                <button
                  class="ble-btn secondary"
                  type="button"
                  onClick={() => stopScan()}
                >
                  <span class="spinner" />
                  Dừng
                </button>
              }
            >
              <button class="ble-btn primary" type="button" onClick={handleScan}>
                Quét thiết bị
              </button>
            </Show>
          </div>

          <Show when={error()}>
            <div class="ble-error">{error()}</div>
          </Show>

          <Show when={adapterOk() === false && !useMock()}>
            <div class="ble-bt-off">
              <p>Bluetooth tắt — không điều khiển được tai nghe thật.</p>
              <button class="ble-btn secondary" type="button" onClick={enableDemo}>
                Mở demo UI
              </button>
            </div>
          </Show>

          <Show when={hasDual() && !useMock()}>
            <p class="ble-tip">
              Cùng tên 2 dòng: chọn RSSI mạnh trước (thường là BLE control).
            </p>
          </Show>

          <div class="ble-list">
            <Show when={matched().length > 0}>
              <div class="ble-group-label">
                {useMock() ? "Demo" : "Thiết bị"}
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
              <div class="ble-group-label">Khác</div>
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
                <p>Chưa thấy thiết bị</p>
                <span>Đưa tai vào chế độ ghép · quét lại</span>
              </div>
            </Show>

            <Show when={scanning() && devices().length === 0}>
              <div class="ble-empty scanning">
                <div class="ble-scan-status" role="status" aria-live="polite">
                  <span class="scan-bars" aria-hidden="true"><i /><i /><i /></span>
                  <span>Đang quét thiết bị</span>
                </div>
                <p>Đang tìm tai nghe ở gần</p>
                <span>Đưa tai vào chế độ ghép nối và mở nắp hộp</span>
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
          Cài đặt
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
            <span class="tag ok">OK</span>
          </Show>
          <Show when={props.device.support === "experimental"}>
            <span class="tag">Beta</span>
          </Show>
        </div>
        <div class="device-meta">
          <span class="rssi">
            <SignalBars level={bars()} />
            {props.device.rssi}
          </span>
          <Show when={props.device.hint}>
            <span class="hint-inline">2 entry</span>
          </Show>
        </div>
      </div>
      <div class="device-action">
        <Show when={!props.connecting} fallback={<span class="spinner small" />}>
          <span class="connect-label">Kết nối</span>
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
