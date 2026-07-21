import { Component, createSignal, Show, onMount, onCleanup } from "solid-js";
import type { AncMode, EqPresetId, SpatialMode } from "./lib/device";
import BlePairing from "./components/BlePairing";
import HomePanel from "./components/HomePanel";
import MorePanel from "./components/MorePanel";
import EqPanel from "./components/EqPanel";
import Settings from "./components/Settings";
import ToastHost from "./components/ToastHost";
import type { BatteryData } from "./components/Battery";
import type { BleDevice, LinkHealth } from "./lib/ble";
import {
  disconnect as bleDisconnect,
  onDisconnected,
  onConnection,
  onLinkHealth,
  getConnection,
  getLinkHealth,
  emptyLink,
} from "./lib/ble";
import {
  fetchBattery,
  queryBattery,
  setAncMode,
  setEqPreset,
  setGameMode,
  setSpatialMode,
  setBassBoost,
  findBuds,
  onBattery,
  onAnc,
  onEq,
  onGameMode,
  toBatteryData,
} from "./lib/device";
import { EQ_LABEL, defaultCustomBands } from "./lib/eq";
import { getAppInfo } from "./lib/app";
import {
  applyTheme,
  getStoredTheme,
  toggleTheme,
  type ThemeMode,
} from "./lib/theme";
import { makeToast, type ToastItem } from "./lib/toast";
import { IconBack } from "./components/Icons";
import "./styles/main.scss";

type View = "home" | "more" | "eq" | "settings";

const App: Component = () => {
  const [view, setView] = createSignal<View>("home");
  const [theme, setTheme] = createSignal<ThemeMode>("dark");
  const [toasts, setToasts] = createSignal<ToastItem[]>([]);
  const [appVersion, setAppVersion] = createSignal("…");
  const [connected, setConnected] = createSignal(false);
  const [device, setDevice] = createSignal<BleDevice | null>(null);
  const [battery, setBattery] = createSignal<BatteryData>({
    left: 0,
    right: 0,
    case: 0,
  });
  const [ancMode, setAncModeUi] = createSignal<AncMode>("off");
  const [ancStrength, setAncStrength] = createSignal(70);
  const [eqActive, setEqActive] = createSignal<EqPresetId>("classic");
  const [eqCustomBands, setEqCustomBands] = createSignal(defaultCustomBands());
  const [eqCustomActive, setEqCustomActive] = createSignal(false);
  const [gameOn, setGameOn] = createSignal(false);
  const [spatialOn, setSpatialOn] = createSignal(false);
  const [spatialMode, setSpatialModeUi] = createSignal<SpatialMode>("music");
  const [bassBoost, setBassBoostUi] = createSignal(0);
  const [ldac, setLdac] = createSignal(false);
  const [hearingProtect, setHearingProtect] = createSignal(false);
  const [link, setLink] = createSignal<LinkHealth>(emptyLink());
  const [controlError, setControlError] = createSignal<string | null>(null);

  let unsubs: Array<() => void> = [];
  let linkPoll: number | undefined;
  let toastTimers = new Map<number, number>();

  const applyLink = (l: LinkHealth) => setLink(l);

  const notify = (
    message: string,
    kind: ToastItem["kind"] = "info",
    title?: string
  ) => {
    const t = makeToast(message, kind, title);
    setToasts((prev) => [...prev.slice(-2), t]);
    const id = window.setTimeout(() => dismissToast(t.id), 2800);
    toastTimers.set(t.id, id);
  };

  const dismissToast = (id: number) => {
    setToasts((prev) => prev.filter((x) => x.id !== id));
    const tm = toastTimers.get(id);
    if (tm) {
      window.clearTimeout(tm);
      toastTimers.delete(id);
    }
  };

  const startLinkPoll = () => {
    if (linkPoll) window.clearInterval(linkPoll);
    linkPoll = window.setInterval(async () => {
      try {
        applyLink(await getLinkHealth());
      } catch {
        /* */
      }
    }, 2000);
  };
  const stopLinkPoll = () => {
    if (linkPoll) {
      window.clearInterval(linkPoll);
      linkPoll = undefined;
    }
  };

  onMount(async () => {
    const t = getStoredTheme();
    applyTheme(t);
    setTheme(t);

    try {
      const info = await getAppInfo();
      setAppVersion(info.version);
    } catch {
      /* */
    }
    try {
      const state = await getConnection();
      if (state.link) applyLink(state.link);
      if (state.connected && state.device) {
        setDevice(state.device);
        setConnected(true);
        startLinkPoll();
        try {
          setBattery(toBatteryData(await queryBattery()));
        } catch {
          setBattery(toBatteryData(await fetchBattery()));
        }
      }
    } catch {
      /* */
    }

    try {
      unsubs.push(
        await onDisconnected(() => {
          setConnected(false);
          setDevice(null);
          setBattery({ left: 0, right: 0, case: 0 });
          setLink(emptyLink());
          setControlError(null);
          setView("home");
          stopLinkPoll();
          notify("Đã ngắt kết nối", "info");
        })
      );
      unsubs.push(
        await onConnection((state) => {
          setConnected(state.connected);
          setDevice(state.device);
          if (state.link) applyLink(state.link);
          if (!state.connected) {
            setBattery({ left: 0, right: 0, case: 0 });
            setLink(emptyLink());
            stopLinkPoll();
          } else startLinkPoll();
        })
      );
      unsubs.push(await onLinkHealth((l) => applyLink(l)));
      unsubs.push(await onBattery((b) => setBattery(toBatteryData(b))));
      unsubs.push(await onAnc((m) => setAncModeUi(m)));
      unsubs.push(
        await onEq((p) => {
          const map: Record<string, EqPresetId> = {
            balanced: "classic",
            classic: "classic",
            bassboost: "bass",
            bass: "bass",
            voice: "voice",
            clear: "clear",
            hifilive: "hifi",
            pop: "pop",
            jazzrock: "jazz",
            classical: "classical",
            acoustic: "acoustic",
          };
          const key = p.toLowerCase().replace(/[^a-z]/g, "");
          setEqActive(map[key] ?? "classic");
        })
      );
      unsubs.push(await onGameMode((on) => setGameOn(on)));
    } catch (e) {
      console.warn("[App] events", e);
    }
  });

  onCleanup(() => {
    unsubs.forEach((u) => u());
    stopLinkPoll();
    toastTimers.forEach((id) => window.clearTimeout(id));
  });

  const handleConnected = async (dev: BleDevice) => {
    setDevice(dev);
    setConnected(true);
    setControlError(null);
    setView("home");
    startLinkPoll();
    notify(`Đã kết nối ${dev.modelName || dev.name}`, "success");
    try {
      const state = await getConnection();
      if (state.link) applyLink(state.link);
      try {
        setBattery(toBatteryData(await queryBattery()));
      } catch {
        setBattery(toBatteryData(await fetchBattery()));
      }
    } catch {
      /* */
    }
  };

  const handleAncMode = async (mode: AncMode) => {
    setAncModeUi(mode);
    setControlError(null);
    try {
      await setAncMode(mode, ancStrength());
      applyLink(await getLinkHealth());
    } catch (e) {
      setControlError(String(e));
      notify(String(e), "error", "Lỗi");
    }
  };

  const handleAncStrength = async (value: number) => {
    setAncStrength(value);
    try {
      await setAncMode(ancMode(), value);
    } catch (e) {
      setControlError(String(e));
    }
  };

  const handleEq = async (preset: EqPresetId) => {
    setEqActive(preset);
    setEqCustomActive(false);
    try {
      await setEqPreset(preset);
      applyLink(await getLinkHealth());
      notify(`EQ · ${EQ_LABEL[preset] ?? preset}`, "success");
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleApplyCustomEq = async () => {
    setEqCustomActive(true);
    // Official Self-Define uses multi-band frames; desktop best-effort:
    // reset path BA43 00 then stay on custom UI (curve kept locally).
    try {
      await setEqPreset("classic");
      notify(
        "Đã lưu đường cong tùy chỉnh (BLE custom full còn best-effort)",
        "warn",
        "EQ custom"
      );
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleResetCustomEq = async () => {
    setEqCustomBands(defaultCustomBands());
    setEqCustomActive(false);
    try {
      await setEqPreset("classic");
      notify("Đã đặt lại EQ", "info");
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleGameMode = async (enabled: boolean) => {
    setGameOn(enabled);
    try {
      await setGameMode(enabled);
      notify(enabled ? "Game mode bật" : "Game mode tắt", "info");
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleSpatialOn = async (on: boolean) => {
    setSpatialOn(on);
    try {
      await setSpatialMode(on ? spatialMode() : "off");
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleSpatialMode = async (m: SpatialMode) => {
    setSpatialModeUi(m);
    setSpatialOn(true);
    try {
      await setSpatialMode(m);
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleBassBoost = async (level: number) => {
    setBassBoostUi(level);
    try {
      await setBassBoost(level);
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleFindBuds = async () => {
    try {
      await findBuds();
      notify("Đang phát âm tìm tai nghe", "info", "Tìm tai");
    } catch (e) {
      notify(String(e), "error");
    }
  };

  const handleRefreshBattery = async () => {
    try {
      const b = toBatteryData(await queryBattery());
      setBattery(b);
      applyLink(await getLinkHealth());
      if (b.left === 0 && b.right === 0 && b.case === 0) {
        notify("Chưa có % pin — mở nắp hộp / đeo tai", "warn", "Pin");
      } else {
        notify(`L ${b.left}% · R ${b.right}% · Hộp ${b.case}%`, "success", "Pin");
      }
    } catch (e) {
      notify(String(e), "error");
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
    setLink(emptyLink());
    setControlError(null);
    setView("home");
    stopLinkPoll();
  };

  const handleTheme = () => {
    const next = toggleTheme(theme());
    setTheme(next);
    notify(next === "dark" ? "Giao diện tối" : "Giao diện sáng", "info");
  };

  return (
    <div class="app">
      <ToastHost items={toasts()} onDismiss={dismissToast} />

      <main class="main-content">
        {/* —— Settings (app) —— */}
        <Show when={view() === "settings"}>
          <section class="section section-scroll">
            <div class="screen-nav">
              <button
                type="button"
                class="screen-back"
                onClick={() => setView("home")}
              >
                <IconBack size={20} />
                Xong
              </button>
              <span class="screen-title">Cài đặt</span>
              <div class="screen-nav-spacer" />
            </div>
            <Settings
              theme={theme()}
              onToggleTheme={handleTheme}
              onNotify={notify}
            />
          </section>
        </Show>

        {/* —— EQ (full screen, separate from “Âm thanh khác”) —— */}
        <Show when={view() === "eq" && connected()}>
          <section class="section section-scroll">
            <EqPanel
              eqActive={eqActive()}
              customBands={eqCustomBands()}
              customActive={eqCustomActive()}
              onBack={() => setView("home")}
              onEq={handleEq}
              onCustomBands={setEqCustomBands}
              onApplyCustom={handleApplyCustomEq}
              onResetCustom={handleResetCustomEq}
            />
          </section>
        </Show>

        {/* —— More sound (bass / LDAC / hearing only) —— */}
        <Show when={view() === "more" && connected()}>
          <section class="section section-scroll">
            <MorePanel
              bassBoost={bassBoost()}
              ldac={ldac()}
              hearingProtect={hearingProtect()}
              onBack={() => setView("home")}
              onBassBoost={handleBassBoost}
              onLdac={(on) => {
                setLdac(on);
                notify(on ? "LDAC (UI)" : "LDAC off", "info");
              }}
              onHearingProtect={(on) => {
                setHearingProtect(on);
                notify(
                  on ? "Hearing protection (UI)" : "Hearing protection off",
                  "info"
                );
              }}
            />
          </section>
        </Show>

        {/* —— Home / Pair —— */}
        <Show when={view() === "home"}>
          <Show
            when={connected()}
            fallback={
              <section class="section section-pair">
                <BlePairing
                  onConnected={handleConnected}
                  onOpenSettings={() => setView("settings")}
                  appVersion={appVersion()}
                />
              </section>
            }
          >
            <section class="section section-scroll">
              <Show when={controlError()}>
                <div class="control-error" style={{ "margin-bottom": "12px" }}>
                  {controlError()}
                </div>
              </Show>
              <HomePanel
                name={device()?.modelName || device()?.name || "Device"}
                modelId={device()?.modelId}
                battery={battery()}
                link={link()}
                ancMode={ancMode()}
                ancStrength={ancStrength()}
                gameMode={gameOn()}
                spatialOn={spatialOn()}
                spatialMode={spatialMode()}
                eqLabel={
                  eqCustomActive()
                    ? "Tùy chỉnh"
                    : EQ_LABEL[eqActive()] ?? eqActive()
                }
                onAncMode={handleAncMode}
                onAncStrength={handleAncStrength}
                onGameMode={handleGameMode}
                onFindBuds={handleFindBuds}
                onRefreshBattery={handleRefreshBattery}
                onOpenMore={() => setView("more")}
                onOpenSettings={() => setView("settings")}
                onDisconnect={handleDisconnect}
                onOpenEq={() => setView("eq")}
                onSpatialOn={handleSpatialOn}
                onSpatialMode={handleSpatialMode}
                onSoundFit={() =>
                  notify(
                    "Cần quy trình test thính lực trên app mobile",
                    "warn",
                    "SoundFit"
                  )
                }
              />
            </section>
          </Show>
        </Show>
      </main>
    </div>
  );
};

export default App;
