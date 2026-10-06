import { createSignal } from "solid-js";
import type { Accessor } from "solid-js";
import type { AncMode, NoiseEnvironment, TransparencyMode } from "../../lib/device";
import { profileNoise, setListeningState } from "../../lib/device";
import { formatError, t } from "../../lib/i18n";

interface Dependencies {
  mode: Accessor<AncMode>;
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

  const applyNoiseParameter = async (mode: AncMode, parameter: number) => {
    dependencies.clearError();
    try {
      await setListeningState({
        mode,
        transparencyMode: transparencyMode(),
        adaptive: adaptiveNoise(),
        environment: noiseEnvironment(),
        level: mode === "anc" && parameter < 100 ? parameter : noiseLevel(),
      });
    } catch (error) {
      const message = formatError(error);
      dependencies.setError(message);
      dependencies.notify(message, "error", t("toast.controlError"));
    }
  };

  return {
    transparencyMode,
    adaptiveNoise,
    noiseEnvironment,
    noiseLevel,
    noiseProfile: () => profileNoise(dependencies.noiseCapabilities()),
    async setMode(mode: AncMode) {
      dependencies.clearError();
      try {
        await setListeningState({
          mode,
          transparencyMode: transparencyMode(),
          adaptive: adaptiveNoise(),
          environment: noiseEnvironment(),
          level: noiseLevel(),
        });
        await dependencies.refreshLink();
      } catch (error) {
        const message = formatError(error);
        dependencies.setError(message);
        dependencies.notify(message, "error", t("toast.error"));
      }
    },
    async setTransparencyMode(mode: TransparencyMode) {
      setTransparencyMode(mode);
      if (dependencies.mode() === "transparency") {
        await applyNoiseParameter("transparency", mode === "voice" ? 1 : 0xff);
      }
    },
    async setAdaptiveNoise(enabled: boolean) {
      setAdaptiveNoise(enabled);
      if (dependencies.mode() === "anc") {
        await applyNoiseParameter("anc", enabled ? noiseEnvironment() : noiseLevel());
      }
    },
    async setNoiseEnvironment(value: NoiseEnvironment) {
      setNoiseEnvironment(value);
      if (dependencies.mode() === "anc" && adaptiveNoise()) {
        await applyNoiseParameter("anc", value);
      }
    },
    async setNoiseLevel(value: number) {
      setNoiseLevel(value);
      if (dependencies.mode() === "anc" && !adaptiveNoise()) {
        await applyNoiseParameter("anc", value);
      }
    },
  };
}
