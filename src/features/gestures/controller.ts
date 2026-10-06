import { createSignal } from "solid-js";
import { setGesture, setInEar, setMultipoint } from "../../lib/device";
import { createConfirmedOperation } from "../shared/confirmedOperation";
import type { createDeviceSession } from "../../stores/deviceSession";

type DeviceSession = ReturnType<typeof createDeviceSession>;

interface Dependencies {
  session: DeviceSession;
  refreshSnapshot(): Promise<void>;
  formatError(error: unknown): string;
  notifyError(message: string): void;
}

/**
 * Owns gesture-mapping and in-ear command admission. Wire confirmation stays in
 * the backend; demo snapshots are published by the backend mock path.
 */
export function createGestureController(dependencies: Dependencies) {
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
    setInEar(enabled: boolean) {
      return operation.run(
        () => setInEar(enabled),
        () => {},
        dependencies.notifyError
      );
    },
    setMultipoint(enabled: boolean) {
      return operation.run(
        () => setMultipoint(enabled),
        () => {},
        dependencies.notifyError
      );
    },
    setGesture(layout: number, left: number | null, right: number | null) {
      return operation.run(
        () => setGesture(layout, left, right),
        () => {},
        dependencies.notifyError
      );
    },
  };
}
