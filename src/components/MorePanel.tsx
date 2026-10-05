/**
 * “Âm thanh khác” — NOT EQ (EQ is its own screen).
 * Bass boost, LDAC, hearing protection, extras.
 */
import { Component, For } from "solid-js";
import { IconBack } from "./Icons";
import { t } from "../lib/i18n";

interface Props {
  bassBoost: number;
  ldac: boolean;
  hearingProtect: boolean;
  onBack: () => void;
  onBassBoost: (n: number) => void;
  onLdac: (on: boolean) => void;
  onHearingProtect: (on: boolean) => void;
}

const MorePanel: Component<Props> = (props) => (
  <div class="more-panel">
    <div class="screen-nav">
      <button type="button" class="screen-back" aria-label={t("nav.back")} onClick={() => props.onBack()}>
        <IconBack size={20} />
      </button>
      <span class="screen-title">{t("more.title")}</span>
      <div class="screen-nav-spacer" />
    </div>

    <section class="more-group">
      <p class="more-label">{t("more.electronicAudio")}</p>
      <div class="more-card">
        <div class="setting-row">
          <div>
            <span class="setting-title">{t("more.bassBoost")}</span>
            <span class="setting-desc">{t("listen.bassRange")}</span>
          </div>
          <div class="level-pills" aria-label={t("listen.bassLevel")}>
            <For each={[{ value: 0, key: "more.off" }, { value: 1, key: "more.mild" }, { value: 2, key: "more.medium" }, { value: 3, key: "more.strong" }]}>
              {(lv) => (
                <button
                  type="button"
                  class={props.bassBoost === lv.value ? "active" : ""}
                  aria-pressed={props.bassBoost === lv.value}
                  onClick={() => props.onBassBoost(lv.value)}
                >
                  <span>{t(lv.key)}</span>
                  <small>{lv.value}</small>
                </button>
              )}
            </For>
          </div>
        </div>
      </div>
    </section>

    <section class="more-group">
              <p class="more-label">{t("more.codecProtection")}</p>
      <div class="more-card">
        <div class="setting-row">
          <div>
            <span class="setting-title">LDAC</span>
            <span class="setting-desc">{t("more.hiRes")}</span>
          </div>
          <label class="toggle sm">
            <input
              type="checkbox"
              checked={props.ldac}
              onChange={(e) =>
                props.onLdac((e.currentTarget as HTMLInputElement).checked)
              }
            />
            <span class="slider" />
          </label>
        </div>
        <div class="setting-row">
          <div>
            <span class="setting-title">{t("more.hearingProtection")}</span>
            <span class="setting-desc">{t("more.hearingProtection")}</span>
          </div>
          <label class="toggle sm">
            <input
              type="checkbox"
              checked={props.hearingProtect}
              onChange={(e) =>
                props.onHearingProtect(
                  (e.currentTarget as HTMLInputElement).checked
                )
              }
            />
            <span class="slider" />
          </label>
        </div>
      </div>
      <p class="eq-footnote">
        {t("more.osCodecNote")}
      </p>
    </section>
  </div>
);

export default MorePanel;
