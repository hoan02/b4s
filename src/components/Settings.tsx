/**
 * App settings — version, update, theme, about
 */
import { Component, Show, createSignal, onMount } from "solid-js";
import { formatError, locale, LOCALE_NAMES, LOCALES, setLocale, t, type Locale } from "../lib/i18n";
import {
  getAppInfo,
  checkForUpdates,
  installUpdate,
  openExternal,
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
  onToggleTheme: () => void;
  onNotify?: (msg: string, kind?: ToastKind, title?: string) => void;
}

const Settings: Component<Props> = (props) => {
  const [info, setInfo] = createSignal<AppInfo | null>(null);
  const [checking, setChecking] = createSignal(false);
  const [installing, setInstalling] = createSignal(false);
  const [update, setUpdate] = createSignal<UpdateCheckResult | null>(null);
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
  });

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
      <div class="settings-hero">
        <img class="settings-app-logo" src="/b4s-logo.png" alt="B4S" />
        <div class="settings-app-ver">
          {t("settings.versionCaption", { version: info()?.version ?? "…" })}
          <Show when={info()?.debug}> · {t("settings.development")}</Show>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">{t("settings.language")}</div>
        <div class="settings-list">
          <label class="settings-row">
            <span class="settings-row-label">{t("settings.language")}</span>
            <select aria-label={t("settings.language")} value={locale()} onChange={(event) => void setLocale(event.currentTarget.value as Locale)}>
              {LOCALES.map((code) => <option value={code}>{LOCALE_NAMES[code]}</option>)}
            </select>
          </label>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">{t("settings.interface")}</div>
        <div class="settings-list">
          <button
            type="button"
            class="settings-row action"
            onClick={() => props.onToggleTheme()}
          >
            <IconTheme size={19} />
            <span class="settings-row-label">{t("settings.interface")}</span>
            <span class="settings-row-value">
              {props.theme === "dark" ? t("settings.dark") : t("settings.light")}
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
    </div>
  );
};

export default Settings;
