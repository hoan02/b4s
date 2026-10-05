/**
 * Listening controls (detail screen). Protocol BA34 / BA43 / BA24 / BA5E / BA02…
 */
import { Component, For, Show, createSignal } from "solid-js";
import type { AncMode, EqPresetId, SpatialMode } from "../lib/device";
import { t } from "../lib/i18n";

export type { AncMode, EqPresetId, SpatialMode };

interface Props {
  ancMode: AncMode;
  ancStrength: number;
  spatialOn: boolean;
  spatialMode: SpatialMode;
  eqActive: EqPresetId;
  gameMode: boolean;
  bassBoost: number; // 0–3
  ldac: boolean;
  hearingProtect: boolean;
  onAncMode: (m: AncMode) => void;
  onAncStrength: (v: number) => void;
  onSpatialOn: (on: boolean) => void;
  onSpatialMode: (m: SpatialMode) => void;
  onEq: (id: EqPresetId) => void;
  onGameMode: (on: boolean) => void;
  onBassBoost: (level: number) => void;
  onLdac: (on: boolean) => void;
  onHearingProtect: (on: boolean) => void;
  onSoundFit?: () => void;
}

const NOISE: { id: AncMode; label: string; sub: string }[] = [
  { id: "off", label: "home.normal", sub: "home.normal" },
  { id: "transparency", label: "home.transparency", sub: "home.transparency" },
  { id: "anc", label: "home.anc", sub: "home.anc" },
];

const EQ_LIST: { id: EqPresetId; label: string }[] = [
  { id: "classic", label: "eqPreset.classic" }, { id: "bass", label: "eqPreset.bass" },
  { id: "hifi", label: "eqPreset.hifi" }, { id: "pop", label: "eqPreset.pop" },
  { id: "jazz", label: "eqPreset.jazz" }, { id: "classical", label: "eqPreset.classical" },
  { id: "clear", label: "eqPreset.clear" }, { id: "acoustic", label: "eqPreset.acoustic" },
  { id: "bassReduce", label: "eqPreset.bassReduce" }, { id: "trebleReduce", label: "eqPreset.trebleReduce" },
  { id: "voice", label: "eqPreset.voice" },
];

const ListeningPanel: Component<Props> = (props) => {
  const [soundOpen, setSoundOpen] = createSignal(true);

  return (
    <div class="listen-panel">
      {/* —— Noise control —— */}
      <section class="listen-card">
        <div class="listen-card-head">
          <h3>{t("listen.noise")}</h3>
          <span class="listen-card-hint">{t("listen.noiseHint")}</span>
        </div>
        <div class="noise-modes">
          <For each={NOISE}>
            {(m) => (
              <button
                type="button"
                class={`noise-btn mode-${m.id} ${props.ancMode === m.id ? "active" : ""}`}
                onClick={() => props.onAncMode(m.id)}
              >
                <span class="noise-label">{t(m.label)}</span>
                <span class="noise-sub">{m.sub}</span>
              </button>
            )}
          </For>
        </div>
        <Show when={props.ancMode === "anc"}>
          <div class="anc-level">
            <div class="anc-level-row">
              <span>{t("listen.noiseLevel")}</span>
              <strong>{props.ancStrength}%</strong>
            </div>
            <input
              type="range"
              min="0"
              max="100"
              step="5"
              value={props.ancStrength}
              onInput={(e) =>
                props.onAncStrength(Number((e.currentTarget as HTMLInputElement).value))
              }
            />
          </div>
        </Show>
      </section>

      {/* —— Spatial / panoramic —— */}
      <section class="listen-card">
        <div class="listen-card-head row">
          <div>
            <h3>{t("listen.spatial")}</h3>
            <span class="listen-card-hint">{t("listen.spatialHint")}</span>
          </div>
          <label class="toggle sm">
            <input
              type="checkbox"
              checked={props.spatialOn}
              onChange={(e) =>
                props.onSpatialOn((e.currentTarget as HTMLInputElement).checked)
              }
            />
            <span class="slider" />
          </label>
        </div>
        <Show when={props.spatialOn}>
          <div class="spatial-modes">
            <button
              type="button"
              class={props.spatialMode === "music" ? "active" : ""}
              onClick={() => props.onSpatialMode("music")}
            >
              {t("listen.music")}
            </button>
            <button
              type="button"
              class={props.spatialMode === "cinema" ? "active" : ""}
              onClick={() => props.onSpatialMode("cinema")}
            >
              {t("listen.cinema")}
            </button>
          </div>
        </Show>
      </section>

      {/* —— EQ —— */}
      <section class="listen-card">
        <div class="listen-card-head">
          <h3>{t("eq.title")}</h3>
          <span class="listen-card-hint">{t("listen.equalizer")}</span>
        </div>
        <div class="eq-chips">
          <For each={EQ_LIST}>
            {(eq) => (
              <button
                type="button"
                class={`eq-chip ${props.eqActive === eq.id ? "active" : ""}`}
                onClick={() => props.onEq(eq.id)}
              >
                {t(eq.label)}
              </button>
            )}
          </For>
        </div>
      </section>

      {/* —— SoundFit —— */}
      <section class="listen-card row-card">
        <div class="row-card-left">
          <div>
            <h3>SoundFit</h3>
          <p>{t("listen.soundFitDescription")}</p>
          </div>
        </div>
        <button type="button" class="row-action" onClick={() => props.onSoundFit?.()}>
          {t("listen.open")}
        </button>
      </section>

      {/* —— Sound settings —— */}
      <section class="listen-card">
        <button
          type="button"
          class="listen-card-head row expand"
          onClick={() => setSoundOpen(!soundOpen())}
        >
          <div>
            <h3>{t("listen.soundSettings")}</h3>
            <span class="listen-card-hint">{t("listen.soundSettings")}</span>
          </div>
          <span class="chev">{soundOpen() ? "−" : "+"}</span>
        </button>

        <Show when={soundOpen()}>
          <div class="sound-block">
            <h4>{t("listen.electronic")}</h4>
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
            <div class="setting-row">
              <div>
                <span class="setting-title">{t("listen.gameMode")}</span>
                <span class="setting-desc">{t("home.lowLatency")}</span>
              </div>
              <label class="toggle sm">
                <input
                  type="checkbox"
                  checked={props.gameMode}
                  onChange={(e) =>
                    props.onGameMode((e.currentTarget as HTMLInputElement).checked)
                  }
                />
                <span class="slider" />
              </label>
            </div>
          </div>

          <div class="sound-block">
            <h4>{t("listen.natural")}</h4>
            <div class="setting-row">
              <div>
                <span class="setting-title">LDAC</span>
                <span class="setting-desc">{t("listen.hiRes")}</span>
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
            <p class="setting-note">{t("listen.ldacNote")}</p>
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
        </Show>
      </section>
    </div>
  );
};

export default ListeningPanel;
