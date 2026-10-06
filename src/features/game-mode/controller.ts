import { createSignal } from "solid-js";
import { setGameMode } from "../../lib/device";
import { createConfirmedOperation } from "../shared/confirmedOperation";
import type { createDeviceSession } from "../../stores/deviceSession";

type DeviceSession = ReturnType<typeof createDeviceSession>;

interface Dependencies {
  session: DeviceSession;
  refreshSnapshot(): Promise<void>;
  isDemo(): boolean;
  setGameMode(value: boolean): void;
  formatError(error: unknown): string;
  notifyChanged(enabled: boolean): void;
  notifyError(message: string): void;
}

/** Owns game-mode command admission and its confirmed-operation UI state. */
export function createGameModeController(dependencies: Dependencies) {
  const [pending, setPending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
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
    reset: operation.reset,
    setMode(enabled: boolean) {
      return operation.run(() => setGameMode(enabled), () => {
        if (dependencies.isDemo()) dependencies.setGameMode(enabled);
        dependencies.notifyChanged(enabled);
      }, dependencies.notifyError);
    },
  };
}
