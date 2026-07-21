import { Component, For, Show, createSignal, onCleanup, onMount } from "solid-js";
import {
  BleDevice,
  startScan,
  stopScan,
  connect,
  onScanStatus,
  onConnecting,
  rssiToBars,
  rssiLabel,
  checkAdapter,
} from "../lib/ble";
import { resolveDeviceThumb } from "../lib/deviceImages";

interface Props {
  onConnected: (device: BleDevice) => void;
}

const BlePairing: Component<Props> = (props) => {
  const [scanning, setScanning] = createSignal(false);
  const [devices, setDevices] = createSignal<BleDevice[]>([]);
  const [connectingId, setConnectingId] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [adapterOk, setAdapterOk] = createSignal<boolean | null>(null);
  const [useMock, setUseMock] = createSignal(false);

  let unsubs: Array<() => void> = [];

  onMount(async () => {
    // Check adapter
    const ok = await checkAdapter();
    setAdapterOk(ok);
    if (!ok) {
      setUseMock(true);
    }

    // Listen for scan updates
    const u1 = await onScanStatus((status) => {
      setScanning(status.scanning);
      setDevices(status.devices);
      if (status.error) setError(status.error);
    });
    unsubs.push(u1);

    const u2 = await onConnecting((id) => {
      setConnectingId(id);
    });
    unsubs.push(u2);

    // Auto-start scan
    handleScan();
  });

  onCleanup(() => {
    unsubs.forEach((u) => u());
    stopScan().catch(() => {});
  });

  const handleScan = async () => {
    setError(null);
    setDevices([]);
    try {
      await startScan(useMock());
    } catch (e) {
      setError(String(e));
      // Retry with mock
      try {
        setUseMock(true);
        await startScan(true);
      } catch (e2) {
        setError(String(e2));
      }
    }
  };

  const handleStop = async () => {
    await stopScan();
  };

  const handleConnect = async (device: BleDevice) => {
    if (connectingId()) return;
    setError(null);
    setConnectingId(device.id);
    try {
      const connected = await connect(device.id, useMock() || device.id.startsWith("mock-"));
      props.onConnected(connected);
    } catch (e) {
      setError(String(e));
      setConnectingId(null);
    }
  };

  const baseusDevices = () => devices().filter((d) => d.isBaseus);
  const otherDevices = () => devices().filter((d) => !d.isBaseus);

  return (
    <div class="ble-pairing">
      <div class="ble-hero">
        <div class="ble-radar">
          <div class={`radar-ring ${scanning() ? "active" : ""}`} />
          <div class={`radar-ring delay ${scanning() ? "active" : ""}`} />
          <div class="radar-core">
            <span class="radar-icon">🎧</span>
          </div>
        </div>
        <h2>Connect your earbuds</h2>
        <p class="ble-subtitle">
          <Show
            when={adapterOk() !== false}
            fallback="Bluetooth adapter not found — using demo mode"
          >
            Make sure your Baseus earbuds are out of the case and in pairing mode
          </Show>
        </p>
      </div>

      {/* Scan controls */}
      <div class="ble-controls">
        <Show
          when={!scanning()}
          fallback={
            <button class="ble-btn secondary" type="button" onClick={handleStop}>
              <span class="spinner" />
              Stop scanning
            </button>
          }
        >
          <button class="ble-btn primary" type="button" onClick={handleScan}>
            🔍 Scan for devices
          </button>
        </Show>

        <Show when={useMock()}>
          <span class="mock-badge">Demo mode</span>
        </Show>
      </div>

      {/* Error */}
      <Show when={error()}>
        <div class="ble-error">{error()}</div>
      </Show>

      {/* Device list */}
      <div class="ble-list">
        <Show when={baseusDevices().length > 0}>
          <div class="ble-group-label">Baseus devices</div>
          <For each={baseusDevices()}>
            {(device) => (
              <DeviceRow
                device={device}
                connecting={connectingId() === device.id}
                onConnect={() => handleConnect(device)}
              />
            )}
          </For>
        </Show>

        <Show when={otherDevices().length > 0}>
          <div class="ble-group-label">Other devices</div>
          <For each={otherDevices()}>
            {(device) => (
              <DeviceRow
                device={device}
                connecting={connectingId() === device.id}
                onConnect={() => handleConnect(device)}
              />
            )}
          </For>
        </Show>

        <Show when={!scanning() && devices().length === 0 && !error()}>
          <div class="ble-empty">
            <p>No devices found</p>
            <span>Put earbuds in pairing mode and scan again</span>
          </div>
        </Show>

        <Show when={scanning() && devices().length === 0}>
          <div class="ble-empty scanning">
            <p>Searching nearby...</p>
          </div>
        </Show>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Single device row
// ---------------------------------------------------------------------------

const DeviceRow: Component<{
  device: BleDevice;
  connecting: boolean;
  onConnect: () => void;
}> = (props) => {
  const bars = () => rssiToBars(props.device.rssi);

  return (
    <button
      class={`ble-device ${props.device.isBaseus ? "baseus" : ""} ${props.connecting ? "connecting" : ""}`}
      type="button"
      disabled={props.connecting}
      onClick={() => props.onConnect()}
    >
      <div class="device-icon photo">
        <img
          src={resolveDeviceThumb(props.device.modelId, props.device.name).src}
          alt=""
          draggable={false}
        />
      </div>

      <div class="device-info">
        <div class="device-name-row">
          <span class="name">{props.device.modelName || props.device.name}</span>
          <Show when={props.device.support === "verified"}>
            <span class="baseus-tag verified">Verified</span>
          </Show>
          <Show when={props.device.support === "experimental"}>
            <span class="baseus-tag experimental">Experimental</span>
          </Show>
          <Show when={props.device.isBaseus && !props.device.support}>
            <span class="baseus-tag">Baseus</span>
          </Show>
        </div>
        <div class="device-meta">
          <span class="rssi" title={rssiLabel(props.device.rssi)}>
            <SignalBars level={bars()} />
            {props.device.rssi} dBm
          </span>
          <Show when={props.device.modelName && props.device.modelName !== props.device.name}>
            <span class="address">{props.device.name}</span>
          </Show>
          <span class="address">{props.device.address}</span>
        </div>
      </div>

      <div class="device-action">
        <Show
          when={!props.connecting}
          fallback={<span class="spinner small" />}
        >
          <span class="connect-label">Connect</span>
        </Show>
      </div>
    </button>
  );
};

// Simple signal bars
const SignalBars: Component<{ level: number }> = (props) => (
  <span class="signal-bars" aria-hidden="true">
    {[1, 2, 3, 4].map((i) => (
      <span class={`bar ${i <= props.level ? "on" : ""}`} />
    ))}
  </span>
);

export default BlePairing;
