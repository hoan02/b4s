/**
 * “Âm thanh khác” — NOT EQ (EQ is its own screen).
 * Bass boost, LDAC, hearing protection, extras.
 */
import { Component, createMemo, For, Show } from "solid-js";
import { IconBack } from "./Icons";
import OperationStatus from "./OperationStatus";
import { t } from "../lib/i18n";

interface Props {
  pending?: boolean;
  error?: string | null;
  bassSupported: boolean;
  bassMaxLevel: number;
  ldacSupported: boolean;
  hearingSupported: boolean;
  bassBoost: number | null;
  ldac: boolean | null;
  hearingProtect: boolean | null;
  hearingThreshold: number | null;
  hearingThresholds: number[];
  onHearingThreshold: (value: number) => void;
  onBack: () => void;
  onBassBoost: (n: number) => void;
  onLdac: (on: boolean) => void;
  onHearingProtect: (on: boolean) => void;
}

const MorePanel: Component<Props> = (props) => {
  const maxBassLevel = createMemo(() => props.bassMaxLevel);
  const bassLevels = createMemo(() => Array.from({ length: maxBassLevel() + 1 }, (_, value) => ({
    value, key: value === 0 ? "more.off" : "more.on",
  })));
  return (
  <div class="more-panel">
    <div class="screen-nav">
      <button type="button" class="screen-back" aria-label={t("nav.back")} onClick={() => props.onBack()}>
        <IconBack size={20} />
      </button>
      <span class="screen-title">{t("more.title")}</span>
      <div class="screen-nav-spacer" />
    </div>

    <Show when={props.bassSupported}>
    <section class="more-group">
      <p class="more-label">{t("more.electronicAudio")}</p>
      <div class="more-card">
        <div class="setting-row" classList={{ "setting-row-stacked": props.bassMaxLevel > 1 }}>
          <div>
            <span class="setting-title">{t("more.bassBoost")}</span>
            <span class="setting-desc">{props.bassBoost === null ? t("control.unknown") : t("more.bassBoost")}</span>
          </div>
          <div class="level-pills" aria-label={t("listen.bassLevel")}>
            <For each={bassLevels()}>
              {(lv) => (
                <button
                  type="button"
                  disabled={props.pending}
                  class={props.bassBoost === lv.value ? "active" : ""}
                  aria-pressed={props.bassBoost === lv.value}
                  onClick={() => props.onBassBoost(lv.value)}
                >
                  <span>{lv.value === 0 || props.bassMaxLevel === 1 ? t(lv.key) : lv.value}</span>
                </button>
              )}
            </For>
          </div>
        </div>
      </div>
    </section>

    </Show>
    <OperationStatus pending={props.pending} error={props.error} />
    <Show when={props.ldacSupported || props.hearingSupported}>
    <section class="more-group">
              <p class="more-label">{t("more.codecProtection")}</p>
      <div class="more-card">
        <Show when={props.ldacSupported}>
        <div class="setting-row">
          <div>
            <span class="setting-title">LDAC</span>
            <span class="setting-desc">{props.ldac === null ? t("control.unknown") : t("more.hiRes")}</span>
          </div>
          <label class="toggle sm">
            <input
              type="checkbox"
              disabled={props.pending}
              aria-label="LDAC"
              aria-checked={props.ldac === null ? "mixed" : props.ldac}
              checked={props.ldac === true}
              onChange={(e) =>
                props.onLdac((e.currentTarget as HTMLInputElement).checked)
              }
            />
            <span class="slider" />
          </label>
        </div>
        </Show>
        <Show when={props.hearingSupported}>
        <div class="setting-row">
          <div>
            <span class="setting-title">{t("more.hearingProtection")}</span>
            <span class="setting-desc">{props.hearingProtect === null ? t("control.unknown") : t("more.hearingProtection")}</span>
          </div>
          <label class="toggle sm">
            <input
              type="checkbox"
              disabled={props.pending}
              aria-label={t("more.hearingProtection")}
              aria-checked={props.hearingProtect === null ? "mixed" : props.hearingProtect}
              checked={props.hearingProtect === true}
              onChange={(e) =>
                props.onHearingProtect(
                  (e.currentTarget as HTMLInputElement).checked
                )
              }
            />
            <span class="slider" />
          </label>
        </div>
        </Show>
      </div>
      <Show when={props.hearingSupported}>
        <label class="setting-row">
          <span class="setting-title">{t("more.hearingThreshold")}</span>
          <select aria-label={t("more.hearingThreshold")} disabled={props.pending || props.hearingProtect === null}
            value={props.hearingThreshold ?? ""}
            onChange={(event) => props.onHearingThreshold(Number(event.currentTarget.value))}>
            <option value="" disabled>{t("control.unknown")}</option>
            <For each={props.hearingThresholds}>{(value) => <option value={value}>{value} dB</option>}</For>
          </select>
        </label>
      </Show>
      <p class="eq-footnote">
        {t("more.osCodecNote")}
      </p>
    </section>
    </Show>
  </div>
  );
};

export default MorePanel;
