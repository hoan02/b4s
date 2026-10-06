/**
 * App settings — version, update, theme, about
 */
import { Component, Show, createSignal, onMount } from "solid-js";
import { formatError, locale, LOCALE_NAMES, LOCALES, setLocale, t, type Locale } from "../lib/i18n";
import {
  getAppInfo,
  getStartAtLogin,
  checkForUpdates,
  installUpdate,
  openExternal,
  setStartAtLogin,
  type AppInfo,
  type UpdateCheckResult,
} from "../lib/app";
import type { ThemeMode } from "../lib/theme";
import type { ToastKind } from "../lib/toast";
import {
  IconDownload,
  IconGithub,
  IconId,
  IconOs,
  IconTheme,
  IconUpdate,
  IconVersion,
} from "./Icons";

interface Props {
  theme: ThemeMode;
  onSelectTheme: (mode: ThemeMode) => void;
  autoReconnect: boolean;
  onAutoReconnectChange: (enabled: boolean) => void;
  onNotify?: (msg: string, kind?: ToastKind, title?: string) => void;
  activeSubpage: "language" | "appearance" | null;
  onNavigate: (page: "language" | "appearance" | null) => void;
}

const Settings: Component<Props> = (props) => {
  const [info, setInfo] = createSignal<AppInfo | null>(null);
  const [checking, setChecking] = createSignal(false);
  const [installing, setInstalling] = createSignal(false);
  const [update, setUpdate] = createSignal<UpdateCheckResult | null>(null);
  const [startAtLogin, setStartAtLoginState] = createSignal<boolean | null>(null);
  const [savingStartAtLogin, setSavingStartAtLogin] = createSignal(false);
  const [status, setStatus] = createSignal<{
    kind: "ok" | "warn" | "err" | "info";
    text?: string;
    key?: string;
    options?: Record<string, unknown>;
  } | null>(null);

  onMount(async () => {
    try {
      setInfo(await getAppInfo());
    } catch (e) {
      setStatus({ kind: "err", text: formatError(e) });
    }
    try {
      setStartAtLoginState(await getStartAtLogin());
    } catch (e) {
      setStartAtLoginState(false);
      setStatus({ kind: "err", text: formatError(e) });
    }
  });

  const handleStartAtLoginChange = async (enabled: boolean) => {
    setSavingStartAtLogin(true);
    setStatus(null);
    try {
      await setStartAtLogin(enabled);
      setStartAtLoginState(enabled);
    } catch (e) {
      setStatus({ kind: "err", text: formatError(e) });
      props.onNotify?.(t("settings.preferenceSaveFailed"), "error");
    } finally {
      setSavingStartAtLogin(false);
    }
  };

  const handleCheck = async () => {
    setChecking(true);
    setStatus(null);
    setUpdate(null);
    try {
      const r = await checkForUpdates();
      setUpdate(r);
      if (r.available) {
        setStatus({
          kind: "warn",
          key: "settings.updateAvailable",
          options: { version: r.version ?? "new" },
        });
        props.onNotify?.(t("settings.updateAvailable", { version: r.version }), "warn", t("settings.updates"));
      } else if (r.error) {
      setStatus({ kind: "err", text: formatError(r.error) });
      } else {
        setStatus({ kind: "ok", key: "settings.latest" });
        props.onNotify?.(t("settings.latest"), "success");
      }
    } catch (e) {
      setStatus({ kind: "err", text: formatError(e) });
    } finally {
      setChecking(false);
    }
  };

  const handleInstall = async () => {
    setInstalling(true);
    setStatus({ kind: "info", key: "settings.downloading" });
    try {
      await installUpdate();
      setStatus({
        kind: "ok",
        key: "settings.updateInstalled",
      });
    } catch (e) {
      setStatus({ kind: "err", text: formatError(e) });
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div class="settings">
      <Show when={props.activeSubpage === "appearance"} fallback={
        <Show when={props.activeSubpage === "language"} fallback={
        <>
      <div class="settings-hero">
        <img class="settings-app-logo" src="/b4s-logo.png" alt="B4S" />
        <div class="settings-app-ver">
          {t("settings.versionCaption", { version: info()?.version ?? "…" })}
          <Show when={info()?.debug}> · {t("settings.development")}</Show>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">{t("settings.general")}</div>
        <div class="settings-list">
          <button
            type="button"
            class="settings-row action"
            onClick={() => props.onNavigate("language")}
            aria-label={`${t("settings.language")}: ${LOCALE_NAMES[locale()]}`}
          >
            <span class="settings-row-label">{t("settings.language")}</span>
            <span class="settings-row-value">{LOCALE_NAMES[locale()]}</span>
            <span class="settings-row-chev" aria-hidden="true">›</span>
          </button>
          <label class="settings-row settings-preference">
            <span class="settings-row-copy">
              <span class="settings-row-label">{t("settings.autoReconnect")}</span>
              <span class="settings-row-hint">{t("settings.autoReconnectHint")}</span>
            </span>
            <input
              type="checkbox"
              checked={props.autoReconnect}
              aria-label={t("settings.autoReconnect")}
              onChange={(event) => props.onAutoReconnectChange(event.currentTarget.checked)}
            />
          </label>
          <label class="settings-row settings-preference">
            <span class="settings-row-copy">
              <span class="settings-row-label">{t("settings.startAtLogin")}</span>
              <span class="settings-row-hint">{t("settings.startAtLoginHint")}</span>
            </span>
            <input
              type="checkbox"
              checked={startAtLogin() ?? false}
              disabled={startAtLogin() === null || savingStartAtLogin()}
              aria-label={t("settings.startAtLogin")}
              onChange={(event) => void handleStartAtLoginChange(event.currentTarget.checked)}
            />
          </label>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">{t("settings.interface")}</div>
        <div class="settings-list">
          <button
            type="button"
            class="settings-row action"
            onClick={() => props.onNavigate("appearance")}
          >
            <IconTheme size={19} />
            <span class="settings-row-label">{t("settings.interface")}</span>
            <span class="settings-row-value">
              {t(`settings.${props.theme}`)}
            </span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">{t("settings.updates")}</div>
        <div class="settings-list">
          <div class="settings-row">
            <IconVersion size={19} />
            <span class="settings-row-label">{t("settings.version")}</span>
            <span class="settings-row-value">{info()?.version ?? "—"}</span>
          </div>
          <button
            type="button"
            class="settings-row action"
            disabled={checking() || installing()}
            onClick={handleCheck}
          >
            <IconUpdate size={19} />
            <span class="settings-row-label">
              {checking() ? t("settings.checking") : t("settings.checkUpdates")}
            </span>
          </button>
          <Show when={update()?.available}>
            <button
              type="button"
              class="settings-row action"
              disabled={installing()}
              onClick={handleInstall}
            >
              <IconDownload size={19} />
              <span class="settings-row-label">
                {installing()
                  ? t("settings.installing")
                  : t("settings.installVersion", { version: update()?.version ?? "latest" })}
              </span>
            </button>
          </Show>
        </div>
        <Show when={status()}>
          <div
            class={`settings-status ${
              status()!.kind === "info" ? "" : status()!.kind
            }`}
          >
            {status()!.key ? t(status()!.key!, status()!.options) : status()!.text}
          </div>
        </Show>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">{t("settings.information")}</div>
        <div class="settings-list">
          <div class="settings-row">
            <IconOs size={19} />
            <span class="settings-row-label">{t("settings.operatingSystem")}</span>
            <span class="settings-row-value">{info()?.os ?? "—"}</span>
          </div>
          <div class="settings-row">
            <IconId size={19} />
            <span class="settings-row-label">{t("settings.identifier")}</span>
            <span class="settings-row-value">
              {info()?.identifier ?? "com.hoan02.b4s"}
            </span>
          </div>
          <button
            type="button"
            class="settings-row action"
            onClick={() => openExternal("https://github.com/hoan02/b4s")}
          >
            <span class="settings-source">
              <IconGithub size={19} />
              <span class="settings-row-label">{t("settings.source")}</span>
            </span>
            <span class="settings-row-value">hoan02/b4s</span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
        <div class="settings-legal-label">{t("settings.notes")}</div>
        <div class="settings-legal">
          <strong>{t("settings.legalTitle")}</strong>
          <div class="settings-legal-section">
            <b>{t("settings.softwareNature")}</b>
            <p>{t("legal.softwareNature")}</p>
          </div>
          <div class="settings-legal-section">
            <b>{t("settings.dataConnection")}</b>
            <p>{t("legal.dataConnection")}</p>
          </div>
          <div class="settings-legal-section">
            <b>{t("settings.modelSupport")}</b>
            <p>{t("legal.modelSupport")}</p>
          </div>
          <div class="settings-legal-section">
            <b>{t("settings.userResponsibility")}</b>
            <p>{t("legal.userResponsibility")}</p>
          </div>
          <div class="settings-legal-section">
            <b>{t("settings.hearingSafety")}</b>
            <p>{t("legal.hearingSafety")}</p>
          </div>
        </div>
      </div>
        </>
        }>
          <div class="settings-language-page">
            <div class="settings-list" role="group" aria-label={t("settings.language")}>
              {LOCALES.map((code) => (
                <button
                  type="button"
                  class={`settings-row action settings-language-option${locale() === code ? " selected" : ""}`}
                  aria-pressed={locale() === code}
                  onClick={() => {
                    void setLocale(code as Locale);
                    props.onNavigate(null);
                  }}
                >
                  <span class="settings-row-label" lang={code}>{LOCALE_NAMES[code]}</span>
                  <span class="settings-language-check" aria-hidden="true">{locale() === code ? "✓" : ""}</span>
                </button>
              ))}
            </div>
          </div>
        </Show>
      }>
        <div class="settings-language-page">
          <div class="settings-list" role="group" aria-label={t("settings.interface")}>
            {(["system", "light", "dark"] as const).map((mode) => (
              <button
                type="button"
                class={`settings-row action settings-language-option${props.theme === mode ? " selected" : ""}`}
                aria-pressed={props.theme === mode}
                onClick={() => {
                  props.onSelectTheme(mode);
                  props.onNavigate(null);
                }}
              >
                <span class="settings-row-label">{t(`settings.${mode}`)}</span>
                <span class="settings-language-check" aria-hidden="true">{props.theme === mode ? "✓" : ""}</span>
              </button>
            ))}
          </div>
        </div>
      </Show>
    </div>
  );
};

export default Settings;
