import { Component, createSignal, Show, onMount, onCleanup } from "solid-js";
import type { AncMode } from "./lib/device";
import BlePairing from "./components/BlePairing";
import HomePanel from "./components/HomePanel";
import MorePanel from "./components/MorePanel";
import EqPanel from "./components/EqPanel";
import GesturePanel from "./components/GesturePanel";
import Settings from "./components/Settings";
import ToastHost from "./components/ToastHost";
import ConfirmDialog from "./components/ConfirmDialog";
import { getDeviceSnapshot, type DeviceSnapshot } from "./bridge/deviceSnapshot";
import { createEqualizerController } from "./features/equalizer/controller";
import { subscribeDeviceRuntime } from "./features/devices/runtimeSubscriptions";
import { createFindBudsController } from "./features/find-buds/controller";
import { createListeningController } from "./features/listening/controller";
import { createSoundController } from "./features/sound/controller";
import { createGameModeController } from "./features/game-mode/controller";
import { createSpatialController } from "./features/spatial/controller";
import { createGestureController } from "./features/gestures/controller";
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
import { queryBattery } from "./lib/device";
import { readDesktopPreferences, writeAutoReconnect, writeExperimentalMode } from "./lib/desktopPreferences";
import { migrateModelIdsOnce } from "./lib/modelIdMigration";
import { getAppInfo, setExperimentalMode } from "./lib/app";
import {
  applyTheme,
  getStoredTheme,
  type ThemeMode,
} from "./lib/theme";
import { makeToast, type ToastItem } from "./lib/toast";
import { formatError, t } from "./lib/i18n";
import { IconBack } from "./components/Icons";
import "./styles/main.scss";

type View = "home" | "more" | "eq" | "settings" | "gestures";
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
  const [experimentalMode, setExperimentalModeEnabled] = createSignal(savedDesktopPreferences.experimentalMode);
  const [experimentalModeReady, setExperimentalModeReady] = createSignal(false);
  const [savingExperimentalMode, setSavingExperimentalMode] = createSignal(false);
  const [device, setDevice] = createSignal<BleDevice | null>(null);
  const [battery, setBattery] = createSignal<BatteryData>({
    left: null,
    right: null,
    case: null,
  });
  const [ancMode, setAncModeUi] = createSignal<AncMode | null>(null);
  const [modelProfiles, setModelProfiles] = createSignal<ModelProfile[]>([]);
  const modelEq = () => modelProfiles().find((profile) => profile.id === device()?.modelId)?.eq;
  const modelGesture = () => modelProfiles().find((profile) => profile.id === device()?.modelId)?.gesture ?? null;
  const modelInEar = () => modelProfiles().find((profile) => profile.id === device()?.modelId)?.inEar ?? null;
  const [gameOn, setGameOn] = createSignal<boolean | null>(null);
  const [spatialOn, setSpatialOn] = createSignal<boolean | null>(null);
  const [bassBoost, setBassBoostUi] = createSignal<number | null>(null);
  const [ldac, setLdac] = createSignal<boolean | null>(null);
  const [hearingThreshold, setHearingThreshold] = createSignal<number | null>(null);
  const [hearingProtect, setHearingProtect] = createSignal<boolean | null>(null);
  const [inEarOn, setInEarOn] = createSignal<boolean | null>(null);
  const [multipointOn, setMultipointOn] = createSignal<boolean | null>(null);
  const [gestureState, setGestureState] = createSignal<Array<{ layout: number; left: number; right: number }>>([]);
  const [link, setLink] = createSignal<LinkHealth>(emptyLink());
  const [controlError, setControlError] = createSignal<string | null>(null);
  let latestLinkSession = -1;
  let latestLinkRevision = -1;
  const noiseCaps = () => device()?.deviceProfile.noise;

  const applySnapshot = (snapshot: DeviceSnapshot | null) => {
    setBattery({
      left: snapshot?.battery.left?.percentage ?? null,
      right: snapshot?.battery.right?.percentage ?? null,
      case: snapshot?.battery.case?.percentage ?? null,
      leftCharging: snapshot?.battery.left?.charging,
      rightCharging: snapshot?.battery.right?.charging,
      caseCharging: snapshot?.battery.case?.charging,
    });
    equalizer.observeSnapshot(snapshot?.eqIndex ?? null, snapshot?.eq ?? null);
    if (!snapshot) {
      equalizer.reset();
      sound.reset();
      spatialController.reset();
      gameModeController.reset();
      setBassBoostUi(null);
      setHearingProtect(null);
      setHearingThreshold(null);
      setInEarOn(null);
      setMultipointOn(null);
      setGestureState([]);
      setSpatialOn(null);
      setAncModeUi(null);
      listening.reset();
      gestures.reset();
      setGameOn(null);
      setLdac(null);
      return;
    }
    setAncModeUi(snapshot.anc?.mode ?? null);
    listening.observeSnapshot(snapshot.anc);
    setGameOn(snapshot.game ?? null);
    setSpatialOn(snapshot.spatialEnabled ?? null);
    setLdac(snapshot.ldac ?? null);
    setBassBoostUi(snapshot.bassBoost ?? null);
    setHearingProtect(snapshot.hearing?.enabled ?? null);
    setHearingThreshold(snapshot.hearing?.level ?? null);
    setInEarOn(snapshot.inEar?.enabled ?? null);
    setMultipointOn(snapshot.multipoint?.enabled ?? null);
    setGestureState(snapshot.gesture.map((value) => ({ layout: value.layout, left: value.left, right: value.right })));
  };
  const session = createDeviceSession(applySnapshot);
  const refreshSnapshot = async () => {
    const generation = session.capture();
    const snapshot = await getDeviceSnapshot();
    if (session.isCurrent(generation)) session.accept(snapshot);
  };
  let disposed = false;
  let stopRuntimeSubscriptions: (() => void) | undefined;
  let linkPoll: number | undefined;
  let toastTimers = new Map<number, number>();

  const applyLink = (value: LinkHealth): boolean => {
    if (value.sessionId < latestLinkSession ||
      (value.sessionId === latestLinkSession && value.revision < latestLinkRevision)) return false;
    latestLinkSession = value.sessionId;
    latestLinkRevision = value.revision;
    setLink(value);
    return true;
  };

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

  const equalizer = createEqualizerController({
    session,
    refreshSnapshot,
    deviceId: () => device()?.id,
    model: () => modelEq() ?? null,
    customEqSupported: () => device()?.deviceProfile.capabilities.customEq ?? false,
    isDemo: () => link().mock,
    spatialOn,
    spatialSupported: () => device()?.deviceProfile.capabilities.spatial ?? false,
    setSpatialOffInDemo: () => setSpatialOn(false),
    formatError,
    notify,
  });

  const listening = createListeningController({
    mode: ancMode,
    session,
    refreshSnapshot,
    noiseCapabilities: noiseCaps,
    clearError: () => setControlError(null),
    setError: setControlError,
    refreshLink: async () => {
      applyLink(await getLinkHealth());
    },
    notify,
  });
  const sound = createSoundController({
    session,
    refreshSnapshot,
    isDemo: () => link().mock,
    hearingThreshold,
    setBassBoost: setBassBoostUi,
    setLdac,
    setHearingProtection: setHearingProtect,
    formatError,
    notifyError: (message) => notify(message, "error"),
  });
  const gameModeController = createGameModeController({
    session,
    refreshSnapshot,
    isDemo: () => link().mock,
    setGameMode: setGameOn,
    formatError,
    notifyChanged: (enabled) => notify(enabled ? t("toast.gameOn") : t("toast.gameOff"), "info"),
    notifyError: (message) => notify(message, "error"),
  });
  const spatialController = createSpatialController({
    session,
    refreshSnapshot,
    isDemo: () => link().mock,
    setSpatialEnabled: setSpatialOn,
    formatError,
    notifyError: (message) => notify(message, "error"),
  });
  const gestures = createGestureController({
    session,
    refreshSnapshot,
    formatError,
    notifyError: (message) => notify(message, "error"),
  });

  const experimentalUnlocked = (key: string): boolean => {
    const profile = device()?.deviceProfile;
    if (!profile) return false;
    const experimental = profile.experimentalFeatures ?? [];
    return !experimental.includes(key) || experimentalMode();
  };
  const inEarSupported = () =>
    (device()?.deviceProfile.capabilities.inEar ?? false) &&
    !!modelInEar() &&
    experimentalUnlocked("inEar");
  const gestureSupported = () =>
    (device()?.deviceProfile.capabilities.gesture ?? false) &&
    !!modelGesture() &&
    experimentalUnlocked("gesture");
  const multipointSupported = () =>
    (device()?.deviceProfile.capabilities.multipoint ?? false) &&
    experimentalUnlocked("multipoint");

  const findController = createFindBudsController(notify);

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
    try {
      await setExperimentalMode(savedDesktopPreferences.experimentalMode);
      setExperimentalModeReady(true);
    } catch {
      setExperimentalModeEnabled(false);
      writeExperimentalMode(false);
      notify(t("settings.preferenceSaveFailed"), "error");
    }

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
      const current = !state.link || applyLink(state.link);
      if (current && state.connected && state.device) {
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
          if (state.link && !applyLink(state.link)) return;
          session.selectDevice(state.connected ? state.device?.id ?? null : null);
          if (state.connected) void refreshSnapshot().catch(() => {});
          setConnected(state.connected);
          setDevice(state.device);
          if (!state.connected) {
            setControlError(null);
            setView("home");
            setBattery({ left: null, right: null, case: null });
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
    findController.reset();
    gestures.reset();
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

  const handleDisconnect = async () => {
    setAutoReconnectAvailable(false);
    try {
      await bleDisconnect();
    } catch (e) {
      console.error(e);
    }
    setConnected(false);
    setDevice(null);
    findController.reset();
    gestures.reset();
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

  const handleExperimentalModeChange = async (enabled: boolean) => {
    if (!experimentalModeReady() || savingExperimentalMode()) return;
    if (!writeExperimentalMode(enabled)) {
      notify(t("settings.preferenceSaveFailed"), "error");
      return;
    }
    setSavingExperimentalMode(true);
    try {
      await setExperimentalMode(enabled);
      setExperimentalModeEnabled(enabled);
    } catch {
      writeExperimentalMode(false);
      setExperimentalModeEnabled(false);
      notify(t("settings.preferenceSaveFailed"), "error");
    } finally {
      setSavingExperimentalMode(false);
    }
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
              experimentalMode={experimentalMode()}
              experimentalModeDisabled={!experimentalModeReady() || savingExperimentalMode()}
              onExperimentalModeChange={handleExperimentalModeChange}
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
              customSupported={device()?.deviceProfile.capabilities.customEq ?? false}
              presets={(modelEq()?.presets ?? []).map((preset) => ({ ...preset, sub: preset.description }))}
              eqActive={equalizer.active()}
              pending={equalizer.pending()}
              error={equalizer.error()}
              customBands={equalizer.customBands()}
              customActive={equalizer.customActive()}
              storageKey={`${device()?.address ?? "default"}.${device()?.modelId ?? "unknown"}.${modelEq()?.bands.join("-") ?? "none"}`}
              onBack={() => setView("home")}
              onEq={equalizer.selectPreset}
              onCustomBands={equalizer.updateCustomBands}
              onApplyCustom={equalizer.applyCustom}
              onResetCustom={equalizer.resetCustom}
            />
          </section>
        </Show>

        {/* —— More sound (bass / LDAC / hearing only) —— */}
        <Show when={view() === "more" && connected()}>
          <section class="section section-scroll">
            <MorePanel
              bassSupported={device()?.deviceProfile.capabilities.bassBoost ?? false}
              ldacSupported={device()?.deviceProfile.capabilities.ldac ?? false}
              pending={sound.pending()}
              error={sound.error()}
              hearingSupported={device()?.deviceProfile.capabilities.hearingProtection ?? false}
              bassBoost={bassBoost()}
              ldac={ldac()}
              hearingProtect={hearingProtect()}
              onBack={() => setView("home")}
              onBassBoost={sound.setBassBoost}
              onLdac={sound.setLdac}
              onHearingProtect={sound.setHearingProtection}
            />
          </section>
        </Show>

        {/* —— Gestures / in-ear (experimental capability) —— */}
        <Show when={view() === "gestures" && connected()}>
          <section class="section section-scroll">
            <GesturePanel
              dualButton={modelGesture()?.dualButton ?? false}
              layouts={modelGesture()?.layouts ?? []}
              gestureState={gestureState()}
              inEarSupported={inEarSupported()}
              inEarOn={inEarOn()}
              pending={gestures.pending()}
              error={gestures.error()}
              experimental={device()?.deviceProfile.experimentalFeatures?.includes("gesture") ?? false}
              onBack={() => setView("home")}
              onInEar={gestures.setInEar}
              onGesture={gestures.setGesture}
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
                <div class="control-error" role="alert" aria-live="assertive" style={{ "margin-bottom": "12px" }}>
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
                ancPending={listening.pending()}
                ancError={listening.error()}
                transparencyMode={listening.transparencyMode()}
                adaptiveNoise={listening.adaptiveNoise()}
                noiseEnvironment={listening.noiseEnvironment()}
                noiseLevel={listening.noiseLevel()}
                listeningSupported={device()?.deviceProfile.capabilities.anc ?? false}
                noiseMaxLevel={listening.noiseProfile().maxLevel}
                noiseSupported={(noiseCaps()?.maxCustomLevel ?? 0) > 0}
                adaptiveSupported={noiseCaps()?.supportsAdaptive ?? false}
                transparencyVoiceSupported={noiseCaps()?.supportsTransparencyVoice ?? false}
                gameMode={gameOn()}
                gamePending={gameModeController.pending()}
                gameError={gameModeController.error()}
                findActive={findController.active()}
                spatialSupported={device()?.deviceProfile.capabilities.spatial ?? false}
                gameSupported={device()?.deviceProfile.capabilities.gameMode ?? false}
                eqSupported={device()?.deviceProfile.capabilities.eq ?? false}
                findSupported={device()?.deviceProfile.capabilities.findBuds ?? false}
                gestureSupported={gestureSupported()}
                inEarSupported={inEarSupported()}
                inEarOn={inEarOn()}
                inEarPending={gestures.pending()}
                inEarError={gestures.error()}
                multipointSupported={multipointSupported()}
                multipointOn={multipointOn()}
                multipointPending={gestures.pending()}
                multipointError={gestures.error()}
                moreSupported={Boolean(device()?.deviceProfile.capabilities.bassBoost || device()?.deviceProfile.capabilities.ldac || device()?.deviceProfile.capabilities.hearingProtection)}
                spatialPending={spatialController.pending()}
                spatialError={spatialController.error()}
                spatialOn={spatialOn()}
                spatialMode={spatialController.mode()}
                eqLabel={
                  equalizer.customActive()
                    ? t("eq.customize")
                    : modelEq()?.presets.find((preset) => preset.id === equalizer.active())?.label ?? "—"
                }
                onAncMode={listening.setMode}
                onTransparencyMode={listening.setTransparencyMode}
                onAdaptiveNoise={listening.setAdaptiveNoise}
                onNoiseEnvironment={listening.setNoiseEnvironment}
                onNoiseLevel={listening.setNoiseLevel}
                onGameMode={gameModeController.setMode}
                onFindBuds={findController.request}
                onOpenMore={() => setView("more")}
                onOpenSettings={() => setView("settings")}
                onDisconnect={handleDisconnect}
                onOpenEq={() => setView("eq")}
                onOpenGestures={() => setView("gestures")}
                onInEar={gestures.setInEar}
                onMultipoint={gestures.setMultipoint}
                onSpatialOn={spatialController.setEnabled}
                onSpatialMode={spatialController.selectMode}
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
      <Show when={equalizer.pendingAction()}>
        <ConfirmDialog
          title={t("dialog.turnOffSpatialTitle")}
          message={t("dialog.turnOffSpatialMessage")}
          onCancel={equalizer.clearPendingAction}
          onConfirm={equalizer.confirmPendingAction}
        />
      </Show>
      <Show when={findController.confirmationOpen() || findController.active()}>
        <ConfirmDialog
          title={t("dialog.loudSoundTitle")}
          message={t("dialog.loudSoundMessage")}
          showCancel={findController.dialogMode() === "confirm"}
          confirmLabel={findController.dialogMode() === "active" ? t("dialog.stopFinding") : t("dialog.ready")}
          onCancel={findController.closeConfirmation}
          onEscape={() => findController.dialogMode() === "active" ? findController.stop() : findController.closeConfirmation()}
          onConfirm={() => (findController.dialogMode() === "active" ? findController.stop() : findController.start())}
        />
      </Show>
    </div>
  );
};

export default App;
