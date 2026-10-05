import { Component, createSignal, Show, onMount, onCleanup } from "solid-js";
import type { AncMode, NoiseEnvironment, EqPresetId, SpatialMode, TransparencyMode } from "./lib/device";
import BlePairing from "./components/BlePairing";
import HomePanel from "./components/HomePanel";
import MorePanel from "./components/MorePanel";
import EqPanel from "./components/EqPanel";
import Settings from "./components/Settings";
import ToastHost from "./components/ToastHost";
import ConfirmDialog from "./components/ConfirmDialog";
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
  setListeningState,
  setEqIndex,
  setCustomEq,
  setGameMode,
  setSpatialMode,
  setBassBoost,
  setLdac as sendLdac,
  setHearingProtection as sendHearingProtection,
  findBuds,
  profileNoise,
  onBattery,
  onAnc,
  onEq,
  onGameMode,
  onBassBoost,
  onLdac,
  onHearingProtection,
  toBatteryData,
} from "./lib/device";
import { EQ_BANDS, EQ_LABEL, defaultCustomBands, presetSort } from "./lib/eq";
import { getAppInfo } from "./lib/app";
import {
  applyTheme,
  getStoredTheme,
  toggleTheme,
  type ThemeMode,
} from "./lib/theme";
import { makeToast, type ToastItem } from "./lib/toast";
import { formatError, t } from "./lib/i18n";
import { IconBack } from "./components/Icons";
import "./styles/main.scss";

type View = "home" | "more" | "eq" | "settings";
type PendingEqAction =
  | { kind: "preset"; preset: EqPresetId }
  | { kind: "customBands"; bands: number[] }
  | { kind: "applyCustom" }
  | { kind: "resetCustom" };

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
  const [transparencyMode, setTransparencyMode] = createSignal<TransparencyMode>("full");
  const [adaptiveNoise, setAdaptiveNoise] = createSignal(true);
  const [noiseEnvironment, setNoiseEnvironment] = createSignal<NoiseEnvironment>(102);
  const [noiseLevel, setNoiseLevel] = createSignal(3);
  const [eqActive, setEqActive] = createSignal<EqPresetId>("classic");
  const [eqCustomBands, setEqCustomBands] = createSignal(defaultCustomBands());
  const [eqCustomActive, setEqCustomActive] = createSignal(false);
  const [gameOn, setGameOn] = createSignal(false);
  const [findActive, setFindActive] = createSignal(false);
  const [findConfirmOpen, setFindConfirmOpen] = createSignal(false);
  const [findDialogMode, setFindDialogMode] = createSignal<"confirm" | "active">("confirm");
  const [spatialOn, setSpatialOn] = createSignal(false);
  const [spatialMode, setSpatialModeUi] = createSignal<SpatialMode>("music");
  const [bassBoost, setBassBoostUi] = createSignal(0);
  const [ldac, setLdac] = createSignal(false);
  const [hearingProtect, setHearingProtect] = createSignal(false);
  const [pendingEqAction, setPendingEqAction] = createSignal<PendingEqAction | null>(null);
  const [link, setLink] = createSignal<LinkHealth>(emptyLink());
  const [controlError, setControlError] = createSignal<string | null>(null);
  const noiseCaps = () => device()?.deviceProfile?.noise;
  const noiseProfile = () => profileNoise(noiseCaps());

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
    const storedTheme = getStoredTheme();
    applyTheme(storedTheme);
    setTheme(storedTheme);

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
          notify(t("toast.disconnected"), "info");
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
      unsubs.push(await onBassBoost((level) => setBassBoostUi(level)));
      unsubs.push(await onLdac((on) => setLdac(on)));
      unsubs.push(await onHearingProtection((state) => setHearingProtect(state.enabled)));
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
    setFindActive(false);
    setControlError(null);
    setView("home");
    startLinkPoll();
    notify(t("toast.connected", { name: dev.modelName || dev.name }), "success");
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
      await setListeningState({
        mode,
        transparencyMode: transparencyMode(),
        adaptive: adaptiveNoise(),
        environment: noiseEnvironment(),
        level: noiseLevel(),
      });
      applyLink(await getLinkHealth());
    } catch (e) {
      setControlError(formatError(e));
      notify(formatError(e), "error", t("toast.error"));
    }
  };

  const handleAncStrength = async (value: number) => {
    setAncStrength(value);
    try {
      await setListeningState({
        mode: ancMode(),
        transparencyMode: transparencyMode(),
        adaptive: adaptiveNoise(),
        environment: noiseEnvironment(),
        level: noiseLevel(),
      });
    } catch (e) {
      setControlError(formatError(e));
    }
  };

  const applyEqPreset = async (preset: EqPresetId) => {
    setEqActive(preset);
    setEqCustomActive(false);
    try {
      await setEqIndex(presetSort(preset));
      applyLink(await getLinkHealth());
      notify(`EQ · ${t(`eqPreset.${preset}`)}`, "success");
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const requestEqAction = (action: PendingEqAction): boolean => {
    if (!spatialOn()) return true;
    setPendingEqAction(action);
    return false;
  };

  const handleEq = async (preset: EqPresetId) => {
    if (!requestEqAction({ kind: "preset", preset })) return;
    await applyEqPreset(preset);
  };

  const handleCustomBands = (bands: number[]) => {
    if (!requestEqAction({ kind: "customBands", bands })) return false;
    setEqCustomBands(bands);
    return true;
  };

  const handleApplyCustomEq = async (bands = eqCustomBands(), label = t("eq.customize")) => {
    if (!requestEqAction({ kind: "applyCustom" })) return;
    setEqCustomActive(true);
    const customLabel = label.trim() || t("eq.customize");
    // Official Self-Define uses multi-band frames; desktop best-effort:
    // reset path BA43 00 then stay on custom UI (curve kept locally).
    try {
      await setCustomEq(
        bands.map((gain, index) => ({
          frequency: EQ_BANDS[index].frequency,
          qValue: 1,
          gain,
          filter: 1,
        }))
      );
      notify(
        t("toast.customEqSaved"),
        "success",
        `EQ custom · ${customLabel}`
      );
    } catch (e) {
      notify(formatError(e), "error", customLabel);
    }
  };

  const handleResetCustomEq = async () => {
    if (!requestEqAction({ kind: "resetCustom" })) return;
    setEqCustomBands(defaultCustomBands());
    setEqCustomActive(false);
    try {
      await setEqIndex(0);
      notify(t("toast.resetEq"), "info");
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const handleGameMode = async (enabled: boolean) => {
    setGameOn(enabled);
    try {
      await setGameMode(enabled);
      notify(enabled ? t("toast.gameOn") : t("toast.gameOff"), "info");
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const handleSpatialOn = async (on: boolean) => {
    setSpatialOn(on);
    try {
      await setSpatialMode(on ? spatialMode() : "off");
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const handleSpatialMode = async (m: SpatialMode) => {
    setSpatialModeUi(m);
    setSpatialOn(true);
    try {
      await setSpatialMode(m);
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const handleBassBoost = async (level: number) => {
    setBassBoostUi(level);
    try {
      await setBassBoost(level);
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const startFindBuds = async () => {
    try {
      await findBuds(true);
      setFindActive(true);
      setFindDialogMode("active");
      notify(t("toast.finding"), "info", t("toast.findingTitle"));
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const stopFindBuds = async () => {
    try {
      await findBuds(false);
      setFindActive(false);
      setFindConfirmOpen(false);
      setFindDialogMode("confirm");
      notify(t("toast.findStopped"), "info", t("toast.stopped"));
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const handleFindBuds = async () => {
    if (!findActive()) {
      setFindDialogMode("confirm");
      setFindConfirmOpen(true);
      return;
    }
    const start = !findActive();
    try {
      await findBuds(start);
      setFindActive(start);
      if (!start) setFindConfirmOpen(false);
      notify(
        start ? t("toast.finding") : t("toast.findStopped"),
        "info",
        start ? t("toast.findingTitle") : t("toast.stopped")
      );
    } catch (e) {
      notify(formatError(e), "error", t("home.find"));
    }
  };

  const applyNoiseParameter = async (mode: AncMode, parameter: number) => {
    setControlError(null);
    try {
      await setListeningState({
        mode,
        transparencyMode: transparencyMode(),
        adaptive: adaptiveNoise(),
        environment: noiseEnvironment(),
        level: mode === "anc" && parameter < 100 ? parameter : noiseLevel(),
      });
    } catch (e) {
      setControlError(formatError(e));
      notify(formatError(e), "error", t("toast.controlError"));
    }
  };

  const confirmEqAction = async () => {
    const action = pendingEqAction();
    setPendingEqAction(null);
    if (!action) return;
    try {
      if (spatialOn()) {
        setSpatialOn(false);
        await setSpatialMode("off");
      }
      if (action.kind === "preset") await applyEqPreset(action.preset);
      if (action.kind === "customBands") setEqCustomBands(action.bands);
      if (action.kind === "applyCustom") await handleApplyCustomEq();
      if (action.kind === "resetCustom") await handleResetCustomEq();
    } catch (e) {
      notify(formatError(e), "error");
    }
  };

  const handleTransparencyMode = async (mode: TransparencyMode) => {
    setTransparencyMode(mode);
    if (ancMode() === "transparency") await applyNoiseParameter("transparency", mode === "voice" ? 1 : 0xff);
  };

  const handleAdaptiveNoise = async (on: boolean) => {
    setAdaptiveNoise(on);
    if (ancMode() === "anc") await applyNoiseParameter("anc", on ? noiseEnvironment() : noiseLevel());
  };

  const handleNoiseEnvironment = async (value: NoiseEnvironment) => {
    setNoiseEnvironment(value);
    if (ancMode() === "anc" && adaptiveNoise()) await applyNoiseParameter("anc", value);
  };

  const handleNoiseLevel = async (value: number) => {
    setNoiseLevel(value);
    if (ancMode() === "anc" && !adaptiveNoise()) await applyNoiseParameter("anc", value);
  };

  const handleLdac = async (enabled: boolean) => {
    setLdac(enabled);
    try {
      await sendLdac(enabled);
    } catch (e) {
      setLdac(!enabled);
      notify(formatError(e), "error");
    }
  };

  const handleHearingProtection = async (enabled: boolean) => {
    setHearingProtect(enabled);
    try {
      await sendHearingProtection(enabled, 1);
    } catch (e) {
      setHearingProtect(!enabled);
      notify(formatError(e), "error");
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
    setFindActive(false);
    setLink(emptyLink());
    setControlError(null);
    setView("home");
    stopLinkPoll();
  };

  const handleTheme = () => {
    const next = toggleTheme(theme());
    setTheme(next);
    notify(next === "dark" ? t("theme.dark") : t("theme.light"), "info");
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
                aria-label={t("nav.back")}
                onClick={() => setView("home")}
              >
                <IconBack size={20} />
              </button>
              <span class="screen-title">{t("nav.settings")}</span>
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
              storageKey={device()?.address || device()?.modelId || "default"}
              onBack={() => setView("home")}
              onEq={handleEq}
              onCustomBands={handleCustomBands}
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
              onLdac={handleLdac}
              onHearingProtect={handleHearingProtection}
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
                imageUrl={device()?.imageUrl}
                battery={battery()}
                link={link()}
                ancMode={ancMode()}
                ancStrength={ancStrength()}
                transparencyMode={transparencyMode()}
                adaptiveNoise={adaptiveNoise()}
                noiseEnvironment={noiseEnvironment()}
                noiseLevel={noiseLevel()}
                noiseMaxLevel={noiseProfile().maxLevel}
                noiseSupported={(noiseCaps()?.maxCustomLevel ?? 0) > 0}
                adaptiveSupported={noiseCaps()?.supportsAdaptive ?? false}
                transparencyVoiceSupported={noiseCaps()?.supportsTransparencyVoice ?? false}
                gameMode={gameOn()}
                findActive={findActive()}
                spatialOn={spatialOn()}
                spatialMode={spatialMode()}
                eqLabel={
                  eqCustomActive()
                    ? t("eq.customize")
                    : EQ_LABEL[eqActive()] ?? eqActive()
                }
                onAncMode={handleAncMode}
                onAncStrength={handleAncStrength}
                onTransparencyMode={handleTransparencyMode}
                onAdaptiveNoise={handleAdaptiveNoise}
                onNoiseEnvironment={handleNoiseEnvironment}
                onNoiseLevel={handleNoiseLevel}
                onGameMode={handleGameMode}
                onFindBuds={handleFindBuds}
                onOpenMore={() => setView("more")}
                onOpenSettings={() => setView("settings")}
                onDisconnect={handleDisconnect}
                onOpenEq={() => setView("eq")}
                onSpatialOn={handleSpatialOn}
                onSpatialMode={handleSpatialMode}
                onSoundFit={() =>
                  notify(
                    t("toast.soundFitUnavailable"),
                    "warn",
                    "SoundFit"
                  )
                }
              />
            </section>
          </Show>
        </Show>
      </main>
      <Show when={pendingEqAction()}>
        <ConfirmDialog
          title={t("dialog.turnOffSpatialTitle")}
          message={t("dialog.turnOffSpatialMessage")}
          onCancel={() => setPendingEqAction(null)}
          onConfirm={confirmEqAction}
        />
      </Show>
      <Show when={findConfirmOpen() || findActive()}>
        <ConfirmDialog
          title={t("dialog.loudSoundTitle")}
          message={t("dialog.loudSoundMessage")}
          showCancel={findDialogMode() === "confirm"}
          confirmLabel={findDialogMode() === "active" ? t("dialog.stopFinding") : t("dialog.ready")}
          onCancel={() => setFindConfirmOpen(false)}
          onConfirm={() => (findDialogMode() === "active" ? stopFindBuds() : startFindBuds())}
        />
      </Show>
      <Show when={false}>
        <ConfirmDialog
          title={t("dialog.loudSoundTitle")}
          message={t("dialog.loudSoundMessage")}
          confirmLabel={t("dialog.ready")}
          onCancel={() => setFindConfirmOpen(false)}
          onConfirm={startFindBuds}
        />
      </Show>
      <Show when={false}>
        <ConfirmDialog
          title={t("dialog.findingTitle")}
          message={t("dialog.findingMessage")}
          cancelLabel={t("dialog.continue")}
          confirmLabel={t("dialog.stopFinding")}
          onCancel={() => undefined}
          onConfirm={handleFindBuds}
        />
      </Show>
    </div>
  );
};

export default App;
