import { Component, createEffect, createSignal, Show, onMount, onCleanup } from "solid-js";
import type { AncMode, NoiseEnvironment, EqPresetId, SpatialMode, TransparencyMode } from "./lib/device";
import BlePairing from "./components/BlePairing";
import HomePanel from "./components/HomePanel";
import MorePanel from "./components/MorePanel";
import EqPanel from "./components/EqPanel";
import Settings from "./components/Settings";
import ToastHost from "./components/ToastHost";
import ConfirmDialog from "./components/ConfirmDialog";
import { getDeviceSnapshot, type DeviceSnapshot } from "./bridge/deviceSnapshot";
import { createConfirmedOperation } from "./features/shared/confirmedOperation";
import { resolveEqSelection } from "./features/equalizer/selection";
import { subscribeDeviceRuntime } from "./features/devices/runtimeSubscriptions";
import { createDeviceSession } from "./stores/deviceSession";
import type { BatteryData } from "./components/Battery";
import type { BleDevice, LinkHealth, ModelProfile } from "./lib/ble";
import {
  disconnect as bleDisconnect,
  getConnection,
  getLinkHealth,
  emptyLink,
  listModelProfiles,
} from "./lib/ble";
import {
  queryBattery,
  setListeningState,
  setEqIndex,
  setEqPreset,
  setCustomEq,
  setGameMode,
  setSpatialMode,
  setBassBoost,
  setLdac as sendLdac,
  setHearingProtection as sendHearingProtection,
  findBuds,
  profileNoise,
} from "./lib/device";
import { defaultCustomBands } from "./lib/eq";
import { readDesktopPreferences, writeAutoReconnect } from "./lib/desktopPreferences";
import { migrateModelIdsOnce } from "./lib/modelIdMigration";
import { getAppInfo } from "./lib/app";
import {
  applyTheme,
  getStoredTheme,
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
  migrateModelIdsOnce();
  const savedDesktopPreferences = readDesktopPreferences();
  const [view, setView] = createSignal<View>("home");
  const [settingsSubpage, setSettingsSubpage] = createSignal<"language" | "appearance" | null>(null);
  const [theme, setTheme] = createSignal<ThemeMode>("dark");
  const [toasts, setToasts] = createSignal<ToastItem[]>([]);
  const [appVersion, setAppVersion] = createSignal("…");
  const [connected, setConnected] = createSignal(false);
  const [connectionReady, setConnectionReady] = createSignal(false);
  const [autoReconnectEnabled, setAutoReconnectEnabled] = createSignal(savedDesktopPreferences.autoReconnect);
  const [autoReconnectThisLaunch, setAutoReconnectThisLaunch] = createSignal(savedDesktopPreferences.autoReconnect);
  const [autoReconnectAvailable, setAutoReconnectAvailable] = createSignal(true);
  const [device, setDevice] = createSignal<BleDevice | null>(null);
  const [battery, setBattery] = createSignal<BatteryData>({
    left: null,
    right: null,
    case: null,
  });
  const [ancMode, setAncModeUi] = createSignal<AncMode>("off");
  const [transparencyMode, setTransparencyMode] = createSignal<TransparencyMode>("full");
  const [adaptiveNoise, setAdaptiveNoise] = createSignal(true);
  const [noiseEnvironment, setNoiseEnvironment] = createSignal<NoiseEnvironment>(102);
  const [noiseLevel, setNoiseLevel] = createSignal(3);
  const [modelProfiles, setModelProfiles] = createSignal<ModelProfile[]>([]);
  const [eqWireIndex, setEqWireIndex] = createSignal<number | null>(null);
  const modelEq = () => modelProfiles().find((profile) => profile.id === device()?.modelId)?.eq;
  const [eqActive, setEqActive] = createSignal<EqPresetId>("classic");
  const [eqCustomBands, setEqCustomBands] = createSignal<number[]>([]);
  const [eqCustomActive, setEqCustomActive] = createSignal(false);
  const [eqPending, setEqPending] = createSignal(false);
  const [gamePending, setGamePending] = createSignal(false);
  const [eqError, setEqError] = createSignal<string | null>(null);
  const [gameError, setGameError] = createSignal<string | null>(null);
  const [gameOn, setGameOn] = createSignal<boolean | null>(null);
  const [findActive, setFindActive] = createSignal(false);
  const [findConfirmOpen, setFindConfirmOpen] = createSignal(false);
  const [findDialogMode, setFindDialogMode] = createSignal<"confirm" | "active">("confirm");
  const [spatialPending, setSpatialPending] = createSignal(false);
  const [spatialError, setSpatialError] = createSignal<string | null>(null);
  const [spatialOn, setSpatialOn] = createSignal<boolean | null>(null);
  const [spatialMode, setSpatialModeUi] = createSignal<SpatialMode>("music");
  const [bassBoost, setBassBoostUi] = createSignal<number | null>(null);
  const [ldac, setLdac] = createSignal<boolean | null>(null);
  const [soundPending, setSoundPending] = createSignal(false);
  const [soundError, setSoundError] = createSignal<string | null>(null);
  const [hearingThreshold, setHearingThreshold] = createSignal<number | null>(null);
  const [hearingProtect, setHearingProtect] = createSignal<boolean | null>(null);
  const [pendingEqAction, setPendingEqAction] = createSignal<PendingEqAction | null>(null);
  const [link, setLink] = createSignal<LinkHealth>(emptyLink());
  const [controlError, setControlError] = createSignal<string | null>(null);
  const noiseCaps = () => device()?.deviceProfile?.noise;
  const noiseProfile = () => profileNoise(noiseCaps());

  const applySnapshot = (snapshot: DeviceSnapshot | null) => {
    setBattery({
      left: snapshot?.battery.left?.percentage ?? null,
      right: snapshot?.battery.right?.percentage ?? null,
      case: snapshot?.battery.case?.percentage ?? null,
      leftCharging: snapshot?.battery.left?.charging,
      rightCharging: snapshot?.battery.right?.charging,
      caseCharging: snapshot?.battery.case?.charging,
    });
    setEqWireIndex(snapshot?.eqIndex ?? null);
    if (!snapshot) {
      equalizer.reset();
      advancedSound.reset();
      spatialOperation.reset();
      gameOperation.reset();
      setBassBoostUi(null);
      setHearingProtect(null);
      setHearingThreshold(null);
      setSpatialOn(null);
      setEqCustomActive(false);
      setEqCustomBands(defaultCustomBands(modelEq()?.bands.length ?? 0));
      setAncModeUi("off");
      setEqActive("classic");
      setGameOn(null);
      setLdac(null);
      return;
    }
    if (snapshot.anc !== null) setAncModeUi(snapshot.anc);
    setGameOn(snapshot.game ?? null);
    setSpatialOn(snapshot.spatialEnabled ?? null);
    setLdac(snapshot.ldac ?? null);
    setBassBoostUi(snapshot.bassBoost ?? null);
    setHearingProtect(snapshot.hearing?.enabled ?? null);
    setHearingThreshold(snapshot.hearing?.level ?? null);
    const eqIds: Record<string, string> = {
      balanced: "classic", bassBoost: "bass", voice: "voice", clear: "clear",
      hifiLive: "hifi", pop: "pop", jazzRock: "jazz", classical: "classical", acoustic: "acoustic",
    };
    if (snapshot.eq !== null && eqIds[snapshot.eq]) setEqActive(eqIds[snapshot.eq]);
  };
  createEffect(() => {
    if (link().mock) return;
    const selection = resolveEqSelection(eqWireIndex(), modelEq()?.presets ?? [],
      device()?.deviceProfile?.capabilities.customEq ?? false);
    setEqCustomActive(selection.kind === "custom");
    setEqActive(selection.kind === "preset" ? selection.id : "");
  });
  let eqDraftLayout = "";
  createEffect(() => {
    const layout = `${device()?.id ?? ""}:${modelEq()?.bands.join(",") ?? ""}`;
    if (layout === eqDraftLayout) return;
    eqDraftLayout = layout;
    setEqCustomBands(defaultCustomBands(modelEq()?.bands.length ?? 0));
  });
  const session = createDeviceSession(applySnapshot);
  const refreshSnapshot = async () => {
    const generation = session.capture();
    const snapshot = await getDeviceSnapshot();
    if (session.isCurrent(generation)) session.accept(snapshot);
  };
  const equalizer = createConfirmedOperation({
    session, refresh: refreshSnapshot, pending: setEqPending, error: setEqError, formatError,
  });
  const advancedSound = createConfirmedOperation({
    session, refresh: refreshSnapshot, pending: setSoundPending, error: setSoundError, formatError,
  });
  const spatialOperation = createConfirmedOperation({
    session, refresh: refreshSnapshot, pending: setSpatialPending, error: setSpatialError, formatError,
  });
  const gameOperation = createConfirmedOperation({
    session, refresh: refreshSnapshot, pending: setGamePending, error: setGameError, formatError,
  });
  let disposed = false;
  let stopRuntimeSubscriptions: (() => void) | undefined;
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

    try { setModelProfiles(await listModelProfiles()); } catch { /* unavailable outside Tauri */ }
    try {
      const info = await getAppInfo();
      setAppVersion(info.version);
    } catch {
      /* */
    }
    try {
      const state = await getConnection();
      if (disposed) return;
      if (state.link) applyLink(state.link);
      if (state.connected && state.device) {
        setAutoReconnectAvailable(false);
        session.selectDevice(state.device.id);
        setDevice(state.device);
        setConnected(true);
        startLinkPoll();
        try {
          await queryBattery();
          await refreshSnapshot();
        } catch {
          await refreshSnapshot();
        }
      }
    } catch {
      /* */
    }

    try {
      stopRuntimeSubscriptions = await subscribeDeviceRuntime({
        connection: (state) => {
          session.selectDevice(state.connected ? state.device?.id ?? null : null);
          if (state.connected) void refreshSnapshot().catch(() => {});
          setConnected(state.connected);
          setDevice(state.device);
          if (state.link) applyLink(state.link);
          if (!state.connected) {
            setControlError(null);
            setView("home");
            setBattery({ left: null, right: null, case: null });
            setLink(emptyLink());
            stopLinkPoll();
            notify(t("toast.disconnected"), "info");
          } else startLinkPoll();
        },
        link: applyLink,
        snapshot: (snapshot) => session.accept(snapshot),
      }, () => !disposed);
      await refreshSnapshot();
    } catch (e) {
      console.warn("[App] events", e);
    }
    if (!disposed) setConnectionReady(true);
  });

  onCleanup(() => {
    disposed = true;
    session.selectDevice(null);
    stopRuntimeSubscriptions?.();
    stopLinkPoll();
    toastTimers.forEach((id) => window.clearTimeout(id));
  });

  const handleConnected = async (dev: BleDevice) => {
    session.selectDevice(dev.id);
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
        await queryBattery();
          await refreshSnapshot();
      } catch {
        await refreshSnapshot();
      }
    } catch {
      /* */
    }
  };

  const handleAncMode = async (mode: AncMode) => {
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

  const applyEqPreset = async (preset: EqPresetId) => {
    await equalizer.run(() => setEqPreset(preset), () => {
      if (link().mock) setEqCustomActive(false);
      const label = modelEq()?.presets.find((item) => item.id === preset)?.label ?? preset;
      notify(`EQ · ${label}`, "success");
    }, (message) => notify(message, "error"));
  };

  const requestEqAction = (action: PendingEqAction): boolean => {
    if (spatialOn() === false || !device()?.deviceProfile?.capabilities.spatial) return true;
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
    const customLabel = label.trim() || t("eq.customize");
    await equalizer.run(() => {
      const schema = modelEq();
      if (!schema || bands.length !== schema.bands.length) {
        throw new Error("Custom EQ draft does not match the model schema");
      }
      return setCustomEq(bands.map((gain, index) => ({
        frequency: schema.bands[index], qValue: 1, gain, filter: 1,
      })), 101, false);
    }, () => {
      if (link().mock) setEqCustomActive(true);
      notify(t("toast.customEqSaved"), "success", `EQ custom · ${customLabel}`);
    }, (message) => notify(message, "error", customLabel));
  };

  const handleResetCustomEq = async () => {
    if (!requestEqAction({ kind: "resetCustom" })) return;
    await equalizer.run(() => setEqIndex(0), () => {
      setEqCustomBands(defaultCustomBands(modelEq()?.bands.length ?? 0));
      if (link().mock) setEqCustomActive(false);
      notify(t("toast.resetEq"), "info");
    }, (message) => notify(message, "error"));
  };

  const handleGameMode = async (enabled: boolean) => {
    await gameOperation.run(() => setGameMode(enabled), () => {
      if (link().mock) setGameOn(enabled);
      notify(enabled ? t("toast.gameOn") : t("toast.gameOff"), "info");
    }, (message) => notify(message, "error"));
  };

  const handleSpatialOn = async (on: boolean) => {
    await spatialOperation.run(() => setSpatialMode(on ? spatialMode() : "off"), () => {
      if (link().mock) setSpatialOn(on);
    }, (message) => notify(message, "error"));
  };

  const handleSpatialMode = async (mode: SpatialMode) => {
    await spatialOperation.run(() => setSpatialMode(mode), () => {
      // Requested mode is a local choice; AA42 only confirms enabled state.
      setSpatialModeUi(mode);
      if (link().mock) setSpatialOn(mode !== "off");
    }, (message) => notify(message, "error"));
  };

  const handleBassBoost = async (level: number) => {
    await advancedSound.run(() => setBassBoost(level), () => {
      if (link().mock) setBassBoostUi(level);
    }, (message) => notify(message, "error"));
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
      if (spatialOn() !== false && device()?.deviceProfile?.capabilities.spatial) {
        const generation = session.capture();
        await setSpatialMode("off");
        if (!session.isCurrent(generation)) return;
        await refreshSnapshot();
        if (!session.isCurrent(generation)) return;
        if (link().mock) setSpatialOn(false);
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
    await advancedSound.run(() => sendLdac(enabled), () => {
      if (link().mock) setLdac(enabled);
    }, (message) => notify(message, "error"));
  };

  const handleHearingProtection = async (enabled: boolean) => {
    await advancedSound.run(() => {
      const threshold = hearingThreshold();
      if (threshold === null) throw new Error(t("control.unknown"));
      return sendHearingProtection(enabled, threshold);
    }, () => {
      if (link().mock) setHearingProtect(enabled);
    }, (message) => notify(message, "error"));
  };

  const handleDisconnect = async () => {
    setAutoReconnectAvailable(false);
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

  const handleTheme = (mode: ThemeMode) => {
    applyTheme(mode);
    setTheme(mode);
  };

  const handleAutoReconnectChange = (enabled: boolean) => {
    if (!writeAutoReconnect(enabled)) {
      notify(t("settings.preferenceSaveFailed"), "error");
      return;
    }
    setAutoReconnectEnabled(enabled);
    if (!enabled) setAutoReconnectThisLaunch(false);
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
                onClick={() => settingsSubpage() ? setSettingsSubpage(null) : setView("home")}
              >
                <IconBack size={20} />
              </button>
              <span class="screen-title">{settingsSubpage() === "language" ? t("settings.language") : settingsSubpage() === "appearance" ? t("settings.interface") : t("nav.settings")}</span>
              <div class="screen-nav-spacer" />
            </div>
            <Settings
              theme={theme()}
              onSelectTheme={handleTheme}
              autoReconnect={autoReconnectEnabled()}
              onAutoReconnectChange={handleAutoReconnectChange}
              onNotify={notify}
              activeSubpage={settingsSubpage()}
              onNavigate={setSettingsSubpage}
            />
          </section>
        </Show>

        {/* —— EQ (full screen, separate from “Âm thanh khác”) —— */}
        <Show when={view() === "eq" && connected()}>
          <section class="section section-scroll">
            <EqPanel
              frequencies={modelEq()?.bands ?? []}
              minGain={modelEq()?.minGain ?? -12}
              maxGain={modelEq()?.maxGain ?? 12}
              customSupported={device()?.deviceProfile?.capabilities.customEq ?? false}
              presets={(modelEq()?.presets ?? []).map((preset) => ({ ...preset, sub: preset.description }))}
              eqActive={eqActive()}
              pending={eqPending()}
              error={eqError()}
              customBands={eqCustomBands()}
              customActive={eqCustomActive()}
              storageKey={`${device()?.address ?? "default"}.${device()?.modelId ?? "unknown"}.${modelEq()?.bands.join("-") ?? "none"}`}
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
              bassSupported={device()?.deviceProfile?.capabilities.bassBoost ?? false}
              ldacSupported={device()?.deviceProfile?.capabilities.ldac ?? false}
              pending={soundPending()}
              error={soundError()}
              hearingSupported={device()?.deviceProfile?.capabilities.hearingProtection ?? false}
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
                <Show when={connectionReady()}>
                  <BlePairing
                    onConnected={handleConnected}
                    onOpenSettings={() => setView("settings")}
                    appVersion={appVersion()}
                    autoReconnect={autoReconnectThisLaunch() && autoReconnectAvailable()}
                    onAutoReconnectAttempt={() => setAutoReconnectAvailable(false)}
                  />
                </Show>
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
                transparencyMode={transparencyMode()}
                adaptiveNoise={adaptiveNoise()}
                noiseEnvironment={noiseEnvironment()}
                noiseLevel={noiseLevel()}
                noiseMaxLevel={noiseProfile().maxLevel}
                noiseSupported={(noiseCaps()?.maxCustomLevel ?? 0) > 0}
                adaptiveSupported={noiseCaps()?.supportsAdaptive ?? false}
                transparencyVoiceSupported={noiseCaps()?.supportsTransparencyVoice ?? false}
                gameMode={gameOn()}
                gamePending={gamePending()}
                gameError={gameError()}
                findActive={findActive()}
                spatialSupported={device()?.deviceProfile?.capabilities.spatial ?? false}
                gameSupported={device()?.deviceProfile?.capabilities.gameMode ?? false}
                eqSupported={device()?.deviceProfile?.capabilities.eq ?? false}
                findSupported={device()?.deviceProfile?.capabilities.findBuds ?? false}
                moreSupported={Boolean(device()?.deviceProfile?.capabilities.bassBoost || device()?.deviceProfile?.capabilities.ldac || device()?.deviceProfile?.capabilities.hearingProtection)}
                spatialPending={spatialPending()}
                spatialError={spatialError()}
                spatialOn={spatialOn()}
                spatialMode={spatialMode()}
                eqLabel={
                  eqCustomActive()
                    ? t("eq.customize")
                    : modelEq()?.presets.find((preset) => preset.id === eqActive())?.label ?? "—"
                }
                onAncMode={handleAncMode}
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
    </div>
  );
};

export default App;
