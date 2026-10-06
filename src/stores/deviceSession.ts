import type { DeviceSnapshot } from "../bridge/deviceSnapshot";

/** Own snapshot ordering and async-operation identity independently of views. */
export function createDeviceSession(onSnapshot: (snapshot: DeviceSnapshot | null) => void) {
  let deviceId: string | null = null;
  let snapshot: DeviceSnapshot | null = null;
  let minimumSession = 0;
  let generation = 0;

  return {
    selectDevice(next: string | null) {
      if (next === deviceId) return;
      if (snapshot) minimumSession = snapshot.sessionId + 1;
      deviceId = next;
      snapshot = null;
      generation++;
      onSnapshot(null);
    },
    accept(next: DeviceSnapshot) {
      if (!deviceId || next.deviceId !== deviceId || next.schemaVersion !== 2 || next.sessionId < minimumSession) return;
      if (snapshot && (next.sessionId < snapshot.sessionId ||
        (next.sessionId === snapshot.sessionId && next.revision <= snapshot.revision))) return;
      if (snapshot && next.sessionId !== snapshot.sessionId) {
        generation++;
        onSnapshot(null);
      }
      snapshot = next;
      minimumSession = next.sessionId;
      onSnapshot(next);
    },
    capture() { return generation; },
    isCurrent(value: number) { return deviceId !== null && generation === value; },
  };
}
