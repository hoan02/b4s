import { createSignal } from "solid-js";
import type { Accessor } from "solid-js";
import type { AncReading } from "../../bridge/deviceSnapshot";
import type { AncMode, NoiseEnvironment, TransparencyMode } from "../../lib/device";
import { profileNoise, setListeningState } from "../../lib/device";
import { formatError, t } from "../../lib/i18n";
import { createConfirmedOperation } from "../shared/confirmedOperation";

interface Dependencies {
  mode: Accessor<AncMode | null>;
  session: { capture(): number; isCurrent(token: number): boolean };
  refreshSnapshot(): Promise<void>;
  noiseCapabilities: Accessor<{
    supportsAdaptive: boolean;
    maxCustomLevel: number;
    environments: number[];
  } | undefined>;
  refreshLink(): Promise<void>;
  notify(message: string, kind: "error", title: string): void;
}

/** Owns listening preferences and translates UI intent into the device command. */
export function createListeningController(dependencies: Dependencies) {
  const [transparencyMode, setTransparencyMode] = createSignal<TransparencyMode | null>(null);
  const [adaptiveNoise, setAdaptiveNoise] = createSignal<boolean | null>(null);
  const [noiseEnvironment, setNoiseEnvironment] = createSignal<NoiseEnvironment | null>(null);
  const [noiseLevel, setNoiseLevel] = createSignal<number | null>(null);
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const operation = createConfirmedOperation({
    session: dependencies.session,
    refresh: dependencies.refreshSnapshot,
    pending: setPending,
    error: (value) => {
      setError(value);
    },
    formatError,
  });

  const apply = async (
    mode: AncMode,
    parameter: number,
    overrides: Partial<{
      transparencyMode: TransparencyMode;
      adaptive: boolean;
      environment: NoiseEnvironment;
      level: number;
    }> = {},
  ) => {
    setError(null);
    await operation.run(
      () => setListeningState({
        mode,
        transparencyMode: overrides.transparencyMode ?? transparencyMode() ?? "full",
        adaptive: overrides.adaptive ?? adaptiveNoise() ?? true,
        environment: (mode === "anc" && parameter >= 100 && parameter !== 0xff
          ? parameter
          : overrides.environment ?? noiseEnvironment() ?? 102) as NoiseEnvironment,
        level: overrides.level ?? (mode === "anc" && parameter < 100
          ? parameter
          : noiseLevel() ?? 3),
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
    reset() {
      operation.reset();
      setTransparencyMode(null);
      setAdaptiveNoise(null);
      setNoiseEnvironment(null);
      setNoiseLevel(null);
      setError(null);
    },
    observeSnapshot(reading: AncReading | null) {
      if (!reading) {
        setTransparencyMode(null);
        setAdaptiveNoise(null);
        setNoiseEnvironment(null);
        setNoiseLevel(null);
        return;
      }
      if (reading.mode === "transparency") {
        setTransparencyMode(reading.parameter === 0xff
          ? "full"
          : reading.parameter === 1 ? "voice" : null);
        return;
      }
      if (reading.mode !== "anc") return;
      const noise = dependencies.noiseCapabilities();
      if (noise?.supportsAdaptive && noise.environments.includes(reading.parameter)) {
        setAdaptiveNoise(true);
        setNoiseEnvironment(reading.parameter as NoiseEnvironment);
        setNoiseLevel(null);
      } else if (noise && reading.parameter >= 1 && reading.parameter <= noise.maxCustomLevel) {
        setAdaptiveNoise(false);
        setNoiseLevel(reading.parameter);
        setNoiseEnvironment(null);
      } else {
        setAdaptiveNoise(null);
        setNoiseEnvironment(null);
        setNoiseLevel(null);
      }
    },
    noiseProfile: () => profileNoise(dependencies.noiseCapabilities()),
    async setMode(mode: AncMode) {
      setError(null);
      await apply(mode, 0xff);
    },
    async setTransparencyMode(mode: TransparencyMode) {
      if (dependencies.mode() === "transparency") {
        await apply("transparency", mode === "voice" ? 1 : 0xff, {
          transparencyMode: mode,
        });
      }
    },
    async setAdaptiveNoise(enabled: boolean) {
      if (dependencies.mode() === "anc") {
        await apply("anc", enabled ? noiseEnvironment() ?? 102 : noiseLevel() ?? 3, {
          adaptive: enabled,
        });
      }
    },
    async setNoiseEnvironment(value: NoiseEnvironment) {
      if (dependencies.mode() === "anc" && adaptiveNoise()) {
        await apply("anc", value, { environment: value });
      }
    },
    async setNoiseLevel(value: number) {
      if (dependencies.mode() === "anc" && !adaptiveNoise()) {
        await apply("anc", value, { level: value, adaptive: false });
      }
    },
  };
}
