/** Validate model data at the bridge; missing constraints never select defaults. */
export const featureKeys = ["anc", "eq", "customEq", "gameMode", "bassBoost", "spatial", "ldac",
  "hearingProtection", "findBuds", "gesture", "inEar", "multipoint", "restoreDefaults", "adaptiveLr", "windNoise",
  "callEnhancement", "batteryEnhancement", "soundBalance", "deviceManagement", "gestureV2",
  "personalizedSound", "firmwareUpdate", "aiServices"] as const;

const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

export function isSoundContract(value: unknown): boolean {
  return value === null || (record(value) && Number.isInteger(value.maxBassLevel) &&
    (value.maxBassLevel as number) > 0 && (value.maxBassLevel as number) <= 255 &&
    typeof value.provenance === "string" && value.provenance.trim().length > 0);
}

export function isEvidenceMap(value: unknown): boolean {
  return record(value) && Object.keys(value).length === featureKeys.length &&
    featureKeys.every(key => key in value) && Object.entries(value).every(([key, entry]) =>
    (featureKeys as readonly string[]).includes(key) && record(entry) &&
    ["unknown", "unsupported", "sourceReviewed", "implemented", "hardwareVerified"].includes(entry.status as string) &&
    typeof entry.provenance === "string" && (entry.status === "unknown" || entry.provenance.trim().length > 0) &&
    Array.isArray(entry.firmwareVersions) && entry.firmwareVersions.every(version => typeof version === "string"));
}

export function assertModelContracts(value: unknown): asserts value is Record<string, unknown> {
  if (!record(value) || value.schemaVersion !== 3 || !isSoundContract(value.sound) ||
    !isEvidenceMap(value.featureEvidence) || !record(value.featureEvidence)) {
    throw new Error("Invalid model contract: schema, sound or feature evidence");
  }
  const evidence = value.featureEvidence;
  if (featureKeys.some(key => !(key in evidence))) throw new Error("Incomplete feature evidence schema");
  if (!record(value.capabilities) || (value.capabilities.bassBoost === true && value.sound === null) ||
    (value.capabilities.eq === true && value.eq === null)) throw new Error("Missing enabled feature constraints");
  const eq = value.eq;
  if (eq !== null && (!record(eq) || !Array.isArray(eq.bands) || !Array.isArray(eq.qValues) ||
    eq.bands.length === 0 || eq.bands.length !== eq.qValues.length ||
    eq.qValues.some(q => typeof q !== "number" || !Number.isFinite(q) || q <= 0) ||
    !record(eq.customWrite) || !Number.isInteger(eq.customWrite.slot) ||
    typeof eq.customWrite.ancBank !== "boolean")) {
    throw new Error("Invalid model contract: explicit EQ constraints required");
  }
}
