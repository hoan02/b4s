import { storage } from "./storage";
import i18next from "i18next";
import { createSignal } from "solid-js";
import en from "../locales/en/translation.json";
import vi from "../locales/vi/translation.json";
import zhCN from "../locales/zh-CN/translation.json";
import es from "../locales/es/translation.json";
import ptBR from "../locales/pt-BR/translation.json";

export const LOCALES = ["en", "vi", "zh-CN", "es", "pt-BR"] as const;
export type Locale = (typeof LOCALES)[number];
export const LOCALE_NAMES: Record<Locale, string> = {
  en: "English",
  vi: "Tiếng Việt",
  "zh-CN": "简体中文",
  es: "Español",
  "pt-BR": "Português (Brasil)",
};

const STORAGE_KEY = "b4s.locale";
const normalizeLocale = (value: string | null): Locale =>
  LOCALES.find((locale) => locale === value) ?? "en";
const initialLocale = normalizeLocale((() => {
  try {
    return storage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
})());

export const i18nReady = i18next.init({
  lng: initialLocale,
  fallbackLng: "en",
  supportedLngs: [...LOCALES],
  defaultNS: "translation",
  resources: {
    en: { translation: en },
    vi: { translation: vi },
    "zh-CN": { translation: zhCN },
    es: { translation: es },
    "pt-BR": { translation: ptBR },
  },
  interpolation: { escapeValue: false },
  returnEmptyString: false,
});

const [currentLocale, setCurrentLocale] = createSignal<Locale>(initialLocale);
i18next.on("languageChanged", (language) => {
  const locale = normalizeLocale(language);
  setCurrentLocale(locale);
  document.documentElement.lang = locale;
  document.documentElement.dir = "ltr";
  try {
    storage.setItem(STORAGE_KEY, locale);
  } catch {
    // Language changes still work for this session when storage is unavailable.
  }
});

document.documentElement.lang = initialLocale;
document.documentElement.dir = "ltr";

export const locale = currentLocale;
export const t = (key: string, options?: Record<string, unknown>): string => {
  currentLocale();
  return i18next.t(key, options);
};

export function formatError(error: unknown): string {
  const payload = typeof error === "object" && error !== null ? error as Record<string, unknown> : null;
  const isVersionedApiError = payload?.contractVersion === 1 &&
    typeof payload.code === "string" &&
    typeof payload.retryable === "boolean" &&
    typeof payload.message === "string";
  const details = error instanceof Error
    ? error.message
    : isVersionedApiError
      ? payload.message
      : String(error);
  // Backend rejections carry the EarSetFailedUtil code reported by the earbuds.
  const deviceCode = typeof details === "string" ? /\[device-code:([0-9A-F]{2})\]/.exec(details)?.[1] : undefined;
  const knownDeviceCodes = ["00", "0A", "0B", "0C", "0D", "0E", "0F", "10", "11", "12", "13", "14", "17"];
  const message = typeof details === "string" && details.includes("Experimental control is disabled")
    ? t("error.experimentalControlDisabled")
    : deviceCode
      ? t(knownDeviceCodes.includes(deviceCode) ? `error.deviceCode.${deviceCode}` : "error.deviceCode.other", { code: deviceCode })
      : details;
  return `${t("error.operationFailed")}: ${message}`;
}

export async function setLocale(language: Locale): Promise<void> {
  await i18next.changeLanguage(language);
}
