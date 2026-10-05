interface OperationSession<T> {
  capture(): T;
  isCurrent(token: T): boolean;
}

interface Dependencies<T> {
  session: OperationSession<T>;
  refresh(): Promise<void>;
  pending(value: boolean): void;
  error(value: string | null): void;
  formatError(error: unknown): string;
}

/** Owns admission and UI publication; backend commands own wire confirmation. */
export function createEqualizerController<T>(dependencies: Dependencies<T>) {
  let epoch = 0;
  let busy = false;
  return {
    reset() {
      epoch++;
      busy = false;
      dependencies.pending(false);
      dependencies.error(null);
    },
    async run(command: () => Promise<void>, confirmed: () => void, failed: (message: string) => void) {
      if (busy) return;
      const operationEpoch = epoch;
      const token = dependencies.session.capture();
      const current = () => operationEpoch === epoch && dependencies.session.isCurrent(token);
      busy = true;
      dependencies.pending(true);
      dependencies.error(null);
      try {
        await command();
        if (!current()) return;
        await dependencies.refresh();
        if (!current()) return;
        confirmed();
      } catch (error) {
        if (!current()) return;
        const message = dependencies.formatError(error);
        dependencies.error(message);
        failed(message);
      } finally {
        if (operationEpoch === epoch) {
          busy = false;
          dependencies.pending(false);
        }
      }
    },
  };
}
