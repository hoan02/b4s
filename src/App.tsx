import { Component, createSignal, Show, onMount, onCleanup } from "solid-js";
import DeviceHeader from "./components/DeviceHeader";
import AncControl, { AncMode } from "./components/AncControl";
import EqCards, { EqPreset } from "./components/EqCards";
import GameModeToggle from "./components/GameModeToggle";
import BlePairing from "./components/BlePairing";
import CaptureStudio from "./components/CaptureStudio";
import type { BatteryData } from "./components/Battery";
import type { BleDevice } from "./lib/ble";
import {
  disconnect as bleDisconnect,
  onDisconnected,
  onConnection,
  getConnection,
} from "./lib/ble";
import {
  fetchBattery,
  setAncMode,
  setEqPreset,
  setGameMode,
  findBuds,
  onBattery,
  onAnc,
  onEq,
  onGameMode,
  toBatteryData,
} from "./lib/device";
import "./styles/main.scss";

type View = "main" | "capture";

const App: Component = () => {
  const [view, setView] = createSignal<View>("main");
  const [connected, setConnected] = createSignal(false);
  const [device, setDevice] = createSignal<BleDevice | null>(null);
  const [battery, setBattery] = createSignal<BatteryData>({
    left: 0,
    right: 0,
    case: 0,
  });
  const [ancMode, setAncModeUi] = createSignal<AncMode>("anc");
  const [ancStrength, setAncStrength] = createSignal(70);
  const [eqActive, setEqActive] = createSignal<EqPreset>("classic");
  const [gameOn, setGameOn] = createSignal(false);

  let unsubs: Array<() => void> = [];

  onMount(async () => {
    try {
      const state = await getConnection();
      if (state.connected && state.device) {
        setDevice(state.device);
        setConnected(true);
        const bat = await fetchBattery();
        setBattery(toBatteryData(bat));
      }
    } catch {
      /* first launch */
    }

    unsubs.push(
      await onDisconnected(() => {
        setConnected(false);
        setDevice(null);
        setBattery({ left: 0, right: 0, case: 0 });
      })
    );

    unsubs.push(
      await onConnection((state) => {
        setConnected(state.connected);
        setDevice(state.device);
        if (!state.connected) {
          setBattery({ left: 0, right: 0, case: 0 });
        }
      })
    );

    unsubs.push(await onBattery((b) => setBattery(toBatteryData(b))));
    unsubs.push(await onAnc((m) => setAncModeUi(m)));
    unsubs.push(
      await onEq((p) => {
        const map: Record<string, EqPreset> = {
          balanced: "classic",
          bassboost: "bass",
          bass: "bass",
          voice: "voice",
          clear: "clear",
        };
        const key = p.toLowerCase().replace(/[^a-z]/g, "");
        setEqActive(map[key] ?? "classic");
      })
    );
    unsubs.push(await onGameMode((on) => setGameOn(on)));
  });

  onCleanup(() => unsubs.forEach((u) => u()));

  const handleConnected = async (dev: BleDevice) => {
    setDevice(dev);
    setConnected(true);
    try {
      const bat = await fetchBattery();
      setBattery(toBatteryData(bat));
    } catch {
      /* mock */
    }
  };

  const handleAncMode = async (mode: AncMode) => {
    setAncModeUi(mode);
    try {
      await setAncMode(mode, ancStrength());
    } catch (e) {
      console.error("[ANC]", e);
    }
  };

  const handleAncStrength = async (value: number) => {
    setAncStrength(value);
    try {
      await setAncMode(ancMode(), value);
    } catch (e) {
      console.error("[ANC strength]", e);
    }
  };

  const handleEq = async (preset: EqPreset) => {
    setEqActive(preset);
    try {
      await setEqPreset(preset);
    } catch (e) {
      console.error("[EQ]", e);
    }
  };

  const handleGameMode = async (enabled: boolean) => {
    setGameOn(enabled);
    try {
      await setGameMode(enabled);
    } catch (e) {
      console.error("[Game]", e);
    }
  };

  const handleFindBuds = async () => {
    try {
      await findBuds();
    } catch (e) {
      console.error("[Find]", e);
    }
  };

  const handleDisconnect = async () => {
    try {
      await bleDisconnect();
    } catch (e) {
      console.error(e);
    }
    setConnected(false);
    setDevice(null);
  };

  return (
    <div class="app">
      <header class="titlebar">
        <div class="titlebar-left">
          <span class="app-logo">baseus</span>
        </div>
        <div class="titlebar-right">
          <button
            class="window-btn"
            type="button"
            title="Capture Studio"
            onClick={() => setView(view() === "capture" ? "main" : "capture")}
            style={{
              color: view() === "capture" ? "#3b82f6" : undefined,
              "font-size": "16px",
            }}
          >
            🔬
          </button>
          <Show when={connected()}>
            <button
              class="window-btn"
              type="button"
              title="Disconnect"
              onClick={handleDisconnect}
            >
              ⏻
            </button>
          </Show>
        </div>
      </header>

      <main class="main-content">
        <Show when={view() === "capture"}>
          <section class="section" style={{ width: "100%", flex: 1 }}>
            <CaptureStudio
              deviceName={device()?.name}
              deviceAddress={device()?.address}
              connected={connected()}
              onClose={() => setView("main")}
            />
          </section>
        </Show>

        <Show when={view() === "main"}>
          <Show
            when={connected()}
            fallback={
              <section class="section">
                <BlePairing onConnected={handleConnected} />
              </section>
            }
          >
            <section class="section">
              <DeviceHeader
              name={device()?.modelName || device()?.name || "Baseus"}
              modelId={device()?.modelId}
              connected={true}
              battery={battery()}
              onFindBuds={handleFindBuds}
              onDisconnect={handleDisconnect}
            />
            </section>

            <section class="section">
              <AncControl
                mode={ancMode()}
                strength={ancStrength()}
                onModeChange={handleAncMode}
                onStrengthChange={handleAncStrength}
              />
            </section>

            <section class="section">
              <GameModeToggle enabled={gameOn()} onChange={handleGameMode} />
            </section>

            <section class="section">
              <EqCards active={eqActive()} onChange={handleEq} />
            </section>
          </Show>
        </Show>
      </main>

      <footer class="app-footer">
        Baseus Desktop ·{" "}
        <Show when={view() === "capture"} fallback={<span>Protocol BP1 Pro</span>}>
          <span style={{ color: "#3b82f6" }}>Capture Studio</span>
        </Show>
        {" · "}
        <Show when={connected()} fallback={<span>Not connected</span>}>
          <span style={{ color: "#22c55e" }}>● Linked</span>
        </Show>
      </footer>
    </div>
  );
};

export default App;
