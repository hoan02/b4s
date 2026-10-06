import { createSignal } from "solid-js";
import type { SpatialMode } from "../../lib/device";
import { setSpatialMode } from "../../lib/device";
import { createConfirmedOperation } from "../shared/confirmedOperation";
import type { createDeviceSession } from "../../stores/deviceSession";

type DeviceSession = ReturnType<typeof createDeviceSession>;

interface Dependencies {
  session: DeviceSession;
  refreshSnapshot(): Promise<void>;
  isDemo(): boolean;
  setSpatialEnabled(value: boolean): void;
  formatError(error: unknown): string;
  notifyError(message: string): void;
}

/** Owns spatial mode intent and confirmation for enable/disable commands. */
export function createSpatialController(dependencies: Dependencies) {
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [mode, setMode] = createSignal<SpatialMode>("music");
  const operation = createConfirmedOperation({
    session: dependencies.session,
    refresh: dependencies.refreshSnapshot,
    pending: setPending,
    error: setError,
    formatError: dependencies.formatError,
  });

  return {
    pending,
    error,
    mode,
    reset: operation.reset,
    setEnabled(enabled: boolean) {
      return operation.run(() => setSpatialMode(enabled ? mode() : "off"), () => {
        if (dependencies.isDemo()) dependencies.setSpatialEnabled(enabled);
      }, dependencies.notifyError);
    },
    selectMode(next: SpatialMode) {
      return operation.run(() => setSpatialMode(next), () => {
        setMode(next);
        if (dependencies.isDemo()) dependencies.setSpatialEnabled(next !== "off");
      }, dependencies.notifyError);
    },
  };
}
