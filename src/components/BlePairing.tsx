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
  type ScanStatus,
  type AudioTarget,
  getAudioTarget,
  prepareAudioTarget,
} from "../lib/ble";
import { findReconnectTarget, readRememberedDevice } from "../lib/reconnect";
import { resolveDeviceThumb } from "../lib/deviceImages";
import { formatError, t } from "../lib/i18n";
import { isTauri } from "../lib/tauri";

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
  const [audioTarget, setAudioTarget] = createSignal<AudioTarget | null>(null);
  const [audioBusy, setAudioBusy] = createSignal(false);
  let checkingAudio = false;
  let lastAudioRefresh = 0;
  const savedDevice = props.autoReconnect ? readRememberedDevice() : null;
  const [autoSearching, setAutoSearching] = createSignal(!!savedDevice);
  const reconnectAbort = new AbortController();
  let reconnectTask: Promise<void> | undefined;

  let unsubs: Array<() => void> = [];
  let checkingAdapter = false;
  let disposed = false;
  let latestScanGeneration = -1;
  let latestScanRevision = -1;
  let latestConnectingSession = -1;
  let adapterPoll: number | undefined;
  let handleFocus: (() => void) | undefined;
  let handleVisibility: (() => void) | undefined;

  const refreshAdapter = async () => {
    if (useMock() || checkingAdapter || scanning() || autoSearching() || connectingId() || audioBusy()) return;
    checkingAdapter = true;
    try {
      const ok = await checkAdapter();
      setAdapterOk(ok);
      if (!ok) {
        if (scanning()) await stopScan();
        setScanning(false);
        setDevices([]);
        setError(t("pair.bluetoothHint"));
      } else if (!scanning() && error() === t("pair.bluetoothHint")) {
        setError(null);
      }
    } finally {
      checkingAdapter = false;
    }
  };

  const refreshAudio = async () => {
    if (!isTauri() || checkingAudio || disposed || useMock() || audioBusy() || connectingId() || scanning() || autoSearching()) return;
    checkingAudio = true;
    lastAudioRefresh = Date.now();
    try {
      const target = await getAudioTarget();
      if (!disposed) setAudioTarget(target);
    } catch (error) {
      if (!disposed) setAudioTarget(null);
      console.warn("Audio output lookup failed", error);
    } finally { checkingAudio = false; }
  };

  const connectAudio = async () => {
    const target = audioTarget();
    if (!target || disposed || audioBusy() || connectingId() || scanning() || autoSearching()) return;
    setAudioBusy(true);
    setError(null);
    try {
      const device = await prepareAudioTarget(target.endpointId);
      if (!disposed) await handleConnect(device, target.endpointId);
    } catch (error) {
      if (!disposed) setError(formatError(error));
    } finally {
      if (!disposed) { setAudioBusy(false); void refreshAudio(); }
    }
  };

  onMount(() => {
    props.onAutoReconnectAttempt();
    handleFocus = () => { void refreshAdapter(); void refreshAudio(); };
    handleVisibility = () => {
      if (document.visibilityState === "visible") { void refreshAdapter(); void refreshAudio(); }
    };
    window.addEventListener("focus", handleFocus);
    document.addEventListener("visibilitychange", handleVisibility);
    adapterPoll = window.setInterval(() => {
      void refreshAdapter();
      if (Date.now() - lastAudioRefresh >= 15_000) void refreshAudio();
    }, 3000);
    void refreshAdapter();
    void refreshAudio();

    void (async () => {
      const scanUnsub = await onScanStatus((status) => {
        acceptScanStatus(status);
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
      const connectingUnsub = await onConnecting((state) => {
        if (state.sessionId < latestConnectingSession) return;
        latestConnectingSession = state.sessionId;
        setConnectingId(state.deviceId);
      });
      if (disposed) connectingUnsub();
      else unsubs.push(connectingUnsub);
    })();
  });

  const acceptScanStatus = (status: ScanStatus) => {
    if (status.generation < latestScanGeneration ||
      (status.generation === latestScanGeneration && status.revision < latestScanRevision)) return;
    latestScanGeneration = status.generation;
    latestScanRevision = status.revision;
    setScanning(status.scanning);
    setDevices(status.devices);
    if (status.error) setError(status.error);
  };

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
      const candidate = await findReconnectTarget(savedDevice, {
        checkAdapter, startScan, stopScan, getScanStatus, onScanStatus, getAudioTarget, prepareAudioTarget,
      }, reconnectAbort.signal);
      if (!disposed && !reconnectAbort.signal.aborted && candidate) {
        setAutoSearching(false);
        await handleConnect(candidate.device, candidate.audioEndpointId);
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
    if (connectingId() || audioBusy()) return;
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
    if (connectingId() || audioBusy()) return;
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

  const handleConnect = async (device: BleDevice, audioEndpointId?: string) => {
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
      props.onConnected(await connect(device.id, isMock, audioEndpointId));
    } catch (e) {
      setError(formatError(e));
      setConnectingId(null);
    }
  };

  const advertisesControl = (device: BleDevice): boolean => {
    const serviceUuid = device.deviceProfile.connection?.serviceUuid;
    return !!serviceUuid &&
      device.advertisedServices.some((uuid) => uuid.toLowerCase() === serviceUuid.toLowerCase());
  };

  const matched = () => {
    const list = devices().filter((d) => d.isBaseus && d.headphoneCandidate !== false);
    return [...list].sort((a, b) => {
      const rank = (s?: string | null) =>
        s === "verified" ? 0 : s === "experimental" ? 1 : 2;
      const r = rank(a.support) - rank(b.support);
      if (r !== 0) return r;
      const control = Number(advertisesControl(b)) - Number(advertisesControl(a));
      return control !== 0 ? control : b.rssi - a.rssi;
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
                  disabled={!!connectingId() || audioBusy()}
                  onClick={() => autoSearching() ? cancelAutoReconnect() : stopScan()}
                >
                  <span class="spinner" />
                  {t("pair.stop")}
                </button>
              }
            >
              <button class="ble-btn primary" type="button" disabled={!!connectingId() || autoSearching() || audioBusy()} onClick={handleScan}>
                {t("pair.scan")}
              </button>
            </Show>
          </div>

          <Show when={audioTarget() && !useMock()}>
            <button class="ble-device matched audio-quick-connect" type="button"
              aria-busy={audioBusy()}
              aria-label={`${t("pair.quickConnect")}: ${audioTarget()?.name}`}
              disabled={audioBusy() || !!connectingId() || scanning() || autoSearching()}
              onClick={connectAudio}>
              <div class="device-icon photo">
                <img src={resolveDeviceThumb(null, audioTarget()?.candidates[0]?.name ?? "", null).src} alt="" draggable={false} />
              </div>
              <div class="device-info">
                <div class="device-name-row"><span class="name">{audioTarget()?.name}</span></div>
                <div class="device-meta"><span>{t("pair.activeAudio")}</span></div>
              </div>
              <div class="device-action">
                <Show when={!audioBusy()} fallback={<span class="spinner small" />}>
                  <span class="connect-label">{t("pair.quickConnect")}</span>
                </Show>
              </div>
            </button>
          </Show>

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
                    disabled={audioBusy() || !!connectingId()}
                    control={advertisesControl(device)}
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
                    disabled={audioBusy() || !!connectingId()}
                    connecting={connectingId() === device.id}
                    onConnect={() => handleConnect(device)}
                  />
                )}
              </For>
            </Show>

            <Show
              when={
                !scanning() &&
                matched().length === 0 &&
                others().length === 0 &&
                !error() &&
                adapterOk() !== false
              }
            >
              <div class="ble-empty">
                <p>{t("pair.empty")}</p>
                <span>{t("pair.emptyHint")}</span>
              </div>
            </Show>

            <Show when={scanning() && matched().length === 0 && others().length === 0}>
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
  control?: boolean;
  connecting: boolean;
  disabled?: boolean;
  onConnect: () => void;
}> = (props) => {
  const bars = () => rssiToBars(props.device.rssi);
  const title = () => props.device.modelName || props.device.name;

  return (
    <button
      class={`ble-device ${props.device.isBaseus ? "matched" : ""} ${props.connecting ? "connecting" : ""}`}
      type="button"
      disabled={props.connecting || props.disabled}
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
          <Show when={props.control}>
            <span class="tag ok">{t("pair.control")}</span>
          </Show>
        </div>
        <div class="device-meta">
          <span class="entry-address">{props.device.address}</span>
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
