import { createSignal } from "solid-js";
import type { Accessor } from "solid-js";
import type { AncMode, NoiseEnvironment, TransparencyMode } from "../../lib/device";
import { profileNoise, setListeningState } from "../../lib/device";
import { formatError, t } from "../../lib/i18n";
import { createConfirmedOperation } from "../shared/confirmedOperation";

interface Dependencies {
  mode: Accessor<AncMode | null>;
  session: { capture(): number; isCurrent(token: number): boolean };
  refreshSnapshot(): Promise<void>;
  noiseCapabilities: Accessor<{ supportsAdaptive: boolean; maxCustomLevel: number } | undefined>;
  clearError(): void;
  setError(error: string | null): void;
  refreshLink(): Promise<void>;
  notify(message: string, kind: "error", title: string): void;
}

/** Owns listening preferences and translates UI intent into the device command. */
export function createListeningController(dependencies: Dependencies) {
  const [transparencyMode, setTransparencyMode] = createSignal<TransparencyMode>("full");
  const [adaptiveNoise, setAdaptiveNoise] = createSignal(true);
  const [noiseEnvironment, setNoiseEnvironment] = createSignal<NoiseEnvironment>(102);
  const [noiseLevel, setNoiseLevel] = createSignal(3);
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const operation = createConfirmedOperation({
    session: dependencies.session,
    refresh: dependencies.refreshSnapshot,
    pending: setPending,
    error: (value) => {
      setError(value);
      dependencies.setError(value);
    },
    formatError,
  });

  const apply = async (mode: AncMode, parameter: number) => {
    dependencies.clearError();
    setError(null);
    await operation.run(
      () => setListeningState({
        mode,
        transparencyMode: transparencyMode(),
        adaptive: adaptiveNoise(),
        environment: noiseEnvironment(),
        level: mode === "anc" && parameter < 100 ? parameter : noiseLevel(),
      }),
      () => { void dependencies.refreshLink(); },
      (message) => dependencies.notify(message, "error", t("toast.controlError")),
    );
  };

  return {
    transparencyMode,
    adaptiveNoise,
    noiseEnvironment,
    noiseLevel,
    pending,
    error,
    noiseProfile: () => profileNoise(dependencies.noiseCapabilities()),
    async setMode(mode: AncMode) {
      dependencies.clearError();
      setError(null);
      await apply(mode, 0xff);
    },
    async setTransparencyMode(mode: TransparencyMode) {
      setTransparencyMode(mode);
      if (dependencies.mode() === "transparency") {
        await apply("transparency", mode === "voice" ? 1 : 0xff);
      }
    },
    async setAdaptiveNoise(enabled: boolean) {
      setAdaptiveNoise(enabled);
      if (dependencies.mode() === "anc") {
        await apply("anc", enabled ? noiseEnvironment() : noiseLevel());
      }
    },
    async setNoiseEnvironment(value: NoiseEnvironment) {
      setNoiseEnvironment(value);
      if (dependencies.mode() === "anc" && adaptiveNoise()) {
        await apply("anc", value);
      }
    },
    async setNoiseLevel(value: number) {
      setNoiseLevel(value);
      if (dependencies.mode() === "anc" && !adaptiveNoise()) {
        await apply("anc", value);
      }
    },
  };
}
