import { createEffect, createSignal } from "solid-js";
import type { EqPresetId } from "../../lib/device";
import { setCustomEq, setEqIndex, setEqPreset, setSpatialMode } from "../../lib/device";
import { defaultCustomBands } from "../../lib/eq";
import type { ModelProfile } from "../../lib/ble";
import { t } from "../../lib/i18n";
import { resolveEqSelection } from "./selection";
import { createConfirmedOperation } from "../shared/confirmedOperation";
import type { createDeviceSession } from "../../stores/deviceSession";

type DeviceSession = ReturnType<typeof createDeviceSession>;
type PendingAction =
  | { kind: "preset"; preset: EqPresetId }
  | { kind: "customBands"; bands: number[] }
  | { kind: "applyCustom" }
  | { kind: "resetCustom" };

interface Dependencies {
  session: DeviceSession;
  refreshSnapshot(): Promise<void>;
  deviceId(): string | undefined;
  model(): ModelProfile["eq"];
  customEqSupported(): boolean;
  isDemo(): boolean;
  spatialOn(): boolean | null;
  spatialSupported(): boolean;
  setSpatialOffInDemo(): void;
  formatError(error: unknown): string;
  notify(message: string, kind: "success" | "info" | "error", title?: string): void;
}

/** Owns EQ draft, spatial conflict confirmation, and confirmed device writes. */
export function createEqualizerController(dependencies: Dependencies) {
  const [wireIndex, setWireIndex] = createSignal<number | null>(null);
  const [active, setActive] = createSignal<EqPresetId>("classic");
  const [customBands, setCustomBands] = createSignal<number[]>([]);
  const [customActive, setCustomActive] = createSignal(false);
  const [pendingAction, setPendingAction] = createSignal<PendingAction | null>(null);
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const operation = createConfirmedOperation({
    session: dependencies.session,
    refresh: dependencies.refreshSnapshot,
    pending: setPending,
    error: setError,
    formatError: dependencies.formatError,
  });

  createEffect(() => {
    if (dependencies.isDemo()) return;
    const selection = resolveEqSelection(wireIndex(), dependencies.model()?.presets ?? [],
      dependencies.customEqSupported());
    setCustomActive(selection.kind === "custom");
    setActive(selection.kind === "preset" ? selection.id : "");
  });

  let draftLayout = "";
  createEffect(() => {
    const model = dependencies.model();
    const layout = `${dependencies.deviceId() ?? ""}:${model?.bands.join(",") ?? ""}`;
    if (layout === draftLayout) return;
    draftLayout = layout;
    setCustomBands(defaultCustomBands(model?.bands.length ?? 0));
  });

  const requestAction = (action: PendingAction): boolean => {
    if (dependencies.spatialOn() === false || !dependencies.spatialSupported()) return true;
    setPendingAction(action);
    return false;
  };

  const performPreset = async (preset: EqPresetId) => operation.run(
    () => setEqPreset(preset),
    () => {
      if (dependencies.isDemo()) setCustomActive(false);
    },
    (message) => dependencies.notify(message, "error"),
  );

  const performCustom = async (bands: number[], label: string) => {
    const customLabel = label.trim() || t("eq.customize");
    await operation.run(() => {
      const schema = dependencies.model();
      if (!schema || bands.length !== schema.bands.length) {
        throw new Error("Custom EQ draft does not match the model schema");
      }
      return setCustomEq(bands.map((gain, index) => ({
        frequency: schema.bands[index], qValue: schema.qValues?.[index] ?? 1, gain, filter: 1,
      })), 101, false);
    }, () => {
      if (dependencies.isDemo()) setCustomActive(true);
    }, (message) => dependencies.notify(message, "error", customLabel));
  };

  const performReset = async () => operation.run(
    () => setEqIndex(0),
    () => {
      setCustomBands(defaultCustomBands(dependencies.model()?.bands.length ?? 0));
      if (dependencies.isDemo()) setCustomActive(false);
    },
    (message) => dependencies.notify(message, "error"),
  );

  return {
    active,
    customBands,
    customActive,
    pending,
    error,
    pendingAction,
    observeSnapshot(index: number | null, preset: string | null) {
      setWireIndex(index);
      const ids: Record<string, string> = {
        balanced: "classic", bassBoost: "bass", voice: "voice", clear: "clear",
        hifiLive: "hifi", pop: "pop", jazzRock: "jazz", classical: "classical", acoustic: "acoustic",
      };
      if (preset !== null && ids[preset]) setActive(ids[preset]);
    },
    reset() {
      operation.reset();
      setCustomActive(false);
      setCustomBands(defaultCustomBands(dependencies.model()?.bands.length ?? 0));
      setActive("classic");
      setPendingAction(null);
    },
    clearPendingAction: () => setPendingAction(null),
    selectPreset(preset: EqPresetId) {
      if (!requestAction({ kind: "preset", preset })) return;
      return performPreset(preset);
    },
    updateCustomBands(bands: number[]) {
      if (!requestAction({ kind: "customBands", bands })) return false;
      setCustomBands(bands);
      return true;
    },
    applyCustom(bands = customBands(), label = t("eq.customize")) {
      if (!requestAction({ kind: "applyCustom" })) return;
      return performCustom(bands, label);
    },
    resetCustom() {
      if (!requestAction({ kind: "resetCustom" })) return;
      return performReset();
    },
    async confirmPendingAction() {
      const action = pendingAction();
      setPendingAction(null);
      if (!action) return;
      try {
        if (dependencies.spatialOn() !== false && dependencies.spatialSupported()) {
          const generation = dependencies.session.capture();
          await setSpatialMode("off");
          if (!dependencies.session.isCurrent(generation)) return;
          await dependencies.refreshSnapshot();
          if (!dependencies.session.isCurrent(generation)) return;
          if (dependencies.isDemo()) dependencies.setSpatialOffInDemo();
        }
        if (action.kind === "preset") await performPreset(action.preset);
        if (action.kind === "customBands") setCustomBands(action.bands);
        if (action.kind === "applyCustom") await performCustom(customBands(), t("eq.customize"));
        if (action.kind === "resetCustom") await performReset();
      } catch (error) {
        dependencies.notify(dependencies.formatError(error), "error");
      }
    },
  };
}
