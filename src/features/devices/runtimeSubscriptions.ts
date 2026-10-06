import type { DeviceSnapshot } from "../../bridge/deviceSnapshot";
import { onDeviceSnapshot } from "../../bridge/deviceSnapshot";
import type { ConnectionState, LinkHealth } from "../../lib/ble";
import { onConnection, onLinkHealth } from "../../lib/ble";
import type { UnlistenFn } from "../../lib/tauri";

export interface RuntimeHandlers {
  connection: (state: ConnectionState) => void;
  link: (state: LinkHealth) => void;
  snapshot: (state: DeviceSnapshot) => void;
}

export async function subscribeDeviceRuntime(
  handlers: RuntimeHandlers,
  isActive: () => boolean
): Promise<UnlistenFn> {
  let disposed = false;
  const unlisten: UnlistenFn[] = [];
  const dispose = () => {
    disposed = true;
    unlisten.splice(0).forEach((stop) => stop());
  };

  const register = async (subscribe: () => Promise<UnlistenFn>) => {
    if (disposed || !isActive()) return;
    const stop = await subscribe();
    if (disposed || !isActive()) stop();
    else unlisten.push(stop);
  };

  try {
    await register(() => onConnection(handlers.connection));
    await register(() => onLinkHealth(handlers.link));
    await register(() => onDeviceSnapshot(handlers.snapshot));
  } catch (error) {
    dispose();
    throw error;
  }

  return dispose;
}
