import { createSignal } from "solid-js";
import { createConfirmedOperation } from "../shared/confirmedOperation";
import type { createDeviceSession } from "../../stores/deviceSession";
import {
  setBassBoost,
  setHearingProtection,
  setLdac,
} from "../../lib/device";
import { t } from "../../lib/i18n";

type DeviceSession = ReturnType<typeof createDeviceSession>;

interface Dependencies {
  session: DeviceSession;
  refreshSnapshot(): Promise<void>;
  isDemo(): boolean;
  hearingThreshold(): number | null;
  hearingEnabled(): boolean | null;
  setBassBoost(value: number): void;
  setLdac(value: boolean): void;
  setHearingProtection(value: boolean): void;
  formatError(error: unknown): string;
  notifyError(message: string): void;
}

/** Owns confirmed sound-setting operations and their pending/error state. */
export function createSoundController(dependencies: Dependencies) {
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
    setBassBoost(value: number) {
      return operation.run(() => setBassBoost(value), () => {
        if (dependencies.isDemo()) dependencies.setBassBoost(value);
      }, dependencies.notifyError);
    },
    setLdac(enabled: boolean) {
      return operation.run(() => setLdac(enabled), () => {
        if (dependencies.isDemo()) dependencies.setLdac(enabled);
      }, dependencies.notifyError);
    },
    setHearingThreshold(level: number) {
      return operation.run(() => {
        const enabled = dependencies.hearingEnabled();
        if (enabled === null) throw new Error(t("control.unknown"));
        return setHearingProtection(enabled, level);
      }, () => {}, dependencies.notifyError);
    },
    setHearingProtection(enabled: boolean) {
      return operation.run(() => {
        const threshold = dependencies.hearingThreshold();
        if (threshold === null) throw new Error(t("control.unknown"));
        return setHearingProtection(enabled, threshold);
      }, () => {
        if (dependencies.isDemo()) dependencies.setHearingProtection(enabled);
      }, dependencies.notifyError);
    },
  };
}
