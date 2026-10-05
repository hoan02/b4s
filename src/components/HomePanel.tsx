/**
 * Home — noise tiles; spatial + SoundFit on root; find/battery as action chips
 */
import { Component, Show } from "solid-js";
import type { BatteryData } from "./Battery";
import type { AncMode, NoiseEnvironment, SpatialMode, TransparencyMode } from "../lib/device";
import type { LinkHealth } from "../lib/ble";
import { resolveDeviceImage } from "../lib/deviceImages";
import { t } from "../lib/i18n";
import {
  IconNormal,
  IconAmbient,
  IconAnc,
  IconGame,
  IconFind,
  IconEq,
  IconMore,
  IconSettings,
  IconPower,
  IconSpatial,
  IconOffice,
  IconOutdoor,
  IconTransit,
  IconFlight,
} from "./Icons";

interface Props {
  name: string;
  modelId?: string | null;
  imageUrl?: string | null;
  battery: BatteryData;
  link: LinkHealth;
  ancMode: AncMode;
  ancStrength: number;
  transparencyMode: TransparencyMode;
  adaptiveNoise: boolean;
  noiseEnvironment: NoiseEnvironment;
  noiseLevel: number;
  noiseMaxLevel: number;
  noiseSupported: boolean;
  adaptiveSupported: boolean;
  transparencyVoiceSupported: boolean;
  gameMode: boolean;
  findActive: boolean;
  spatialOn: boolean;
  spatialMode: SpatialMode;
  eqLabel: string;
  onAncMode: (m: AncMode) => void;
  onAncStrength: (v: number) => void;
  onTransparencyMode: (m: TransparencyMode) => void;
  onAdaptiveNoise: (on: boolean) => void;
  onNoiseEnvironment: (v: NoiseEnvironment) => void;
  onNoiseLevel: (v: number) => void;
  onGameMode: (on: boolean) => void;
  onFindBuds: () => void;
  onOpenMore: () => void;
  onOpenSettings: () => void;
  onDisconnect: () => void;
  onOpenEq: () => void;
  onSpatialOn: (on: boolean) => void;
  onSpatialMode: (m: SpatialMode) => void;
  onSoundFit: () => void;
}

const AdaptiveEnvironmentCards = (props: {
  selected: NoiseEnvironment;
  onSelect: (value: NoiseEnvironment) => void;
}) => (
  <div class="noise-environments noise-environments-card">
    <button type="button" class={props.selected === 102 ? "active" : ""} onClick={() => props.onSelect(102)}><IconOffice size={24} /><span><strong>{t("home.indoor")}</strong><small>{t("home.homeOffice")}</small></span></button>
    <button type="button" class={props.selected === 103 ? "active" : ""} onClick={() => props.onSelect(103)}><IconOutdoor size={24} /><span><strong>{t("home.outdoor")}</strong><small>{t("home.streetPark")}</small></span></button>
    <button type="button" class={props.selected === 101 ? "active" : ""} onClick={() => props.onSelect(101)}><IconTransit size={24} /><span><strong>{t("home.commuting")}</strong><small>{t("home.subwayBus")}</small></span></button>
    <button type="button" class={props.selected === 108 ? "active" : ""} onClick={() => props.onSelect(108)}><IconFlight size={24} /><span><strong>{t("home.inTransit")}</strong><small>{t("home.planeTrain")}</small></span></button>
  </div>
);

function pctClass(p: number) {
  if (p <= 0) return "unk";
  if (p <= 20) return "low";
  if (p <= 50) return "mid";
  return "ok";
}
function fmt(p: number) {
  return p <= 0 ? "—" : `${Math.min(100, p)}%`;
}

const HomePanel: Component<Props> = (props) => {
  const visual = () => resolveDeviceImage(props.modelId, props.name, props.imageUrl);
  const level = () => props.link.level;
  const statusText = () => {
    switch (level()) {
      case "live":
        return t("home.connected");
      case "waiting":
        return t("home.waiting");
      case "demo":
        return t("home.demo");
      case "dead":
        return t("home.linkLost");
      default:
        return t("home.connected");
    }
  };

  const statusDot = () =>
    level() === "live" ? "live" : level() === "dead" ? "dead" : "wait";

  return (
    <div class="home">
      {/* Sticky: name + status — stays while scrolling */}
      <header class="home-sticky">
        <div class="home-sticky-inner">
          <div class="home-sticky-text">
            <h1 class="home-sticky-name" title={props.name}>
              {props.name}
            </h1>
            <div class="home-sticky-status">
              <span class={`dot ${statusDot()}`} />
              <span>{statusText()}</span>
            </div>
          </div>
        </div>
      </header>

      <div class="home-body">
      <div class="home-hero">
        <img
          class="home-device-img"
          src={visual().src}
          alt=""
          draggable={false}
        />
      </div>

      <div class="home-batt">
        <div class="home-batt-cell">
          <div class={`pct ${pctClass(props.battery.left)}`}>
            {fmt(props.battery.left)}
          </div>
          <div class="tag">{t("home.indicatorLeft")}</div>
        </div>
        <div class="home-batt-cell">
          <div class={`pct ${pctClass(props.battery.case)}`}>
            {fmt(props.battery.case)}
          </div>
          <div class="tag">{t("home.indicatorCase")}</div>
        </div>
        <div class="home-batt-cell">
          <div class={`pct ${pctClass(props.battery.right)}`}>
            {fmt(props.battery.right)}
          </div>
          <div class="tag">{t("home.indicatorRight")}</div>
        </div>
      </div>

      {/* Noise — only square tiles */}
      <div>
        <p class="home-section-label">{t("home.noise")}</p>
        <div class="noise-tiles">
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "off" ? "active" : ""}`}
            onClick={() => props.onAncMode("off")}
          >
            <IconNormal size={28} />
            <span>{t("home.normal")}</span>
          </button>
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "transparency" ? "active" : ""}`}
            onClick={() => props.onAncMode("transparency")}
          >
            <IconAmbient size={28} />
            <span>{t("home.transparency")}</span>
          </button>
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "anc" ? "active" : ""}`}
            disabled={!props.noiseSupported}
            onClick={() => props.onAncMode("anc")}
          >
            <IconAnc size={28} />
            <span>{t("home.anc")}</span>
          </button>
        </div>
        <Show when={props.ancMode === "transparency"}>
          <div class="noise-options" aria-label={t("home.transparencyOptions")}>
            <button type="button" class={props.transparencyMode === "full" ? "active" : ""} onClick={() => props.onTransparencyMode("full")}><span>{t("home.fullTransparency")}</span><small>{t("home.default")}</small></button>
            <button type="button" class={props.transparencyMode === "voice" ? "active" : ""} onClick={() => props.onTransparencyMode("voice")}><span>{t("home.voiceMode")}</span><small>{t("home.prioritizeVoice")}</small></button>
          </div>
        </Show>
        <Show when={props.ancMode === "anc"}>
          <Show when={props.adaptiveNoise}>
          <div class="noise-environments noise-environments-new">
            <button type="button" class={props.noiseEnvironment === 102 ? "active" : ""} onClick={() => props.onNoiseEnvironment(102)}><IconOffice size={28} /><strong>{t("home.indoor")}</strong><small>{t("home.homeOffice")}</small></button>
            <button type="button" class={props.noiseEnvironment === 103 ? "active" : ""} onClick={() => props.onNoiseEnvironment(103)}><IconOutdoor size={28} /><strong>{t("home.outdoor")}</strong><small>{t("home.streetPark")}</small></button>
            <button type="button" class={props.noiseEnvironment === 101 ? "active" : ""} onClick={() => props.onNoiseEnvironment(101)}><IconTransit size={28} /><strong>{t("home.commuting")}</strong><small>{t("home.subwayBus")}</small></button>
            <button type="button" class={props.noiseEnvironment === 108 ? "active" : ""} onClick={() => props.onNoiseEnvironment(108)}><IconFlight size={28} /><strong>{t("home.inTransit")}</strong><small>{t("home.planeTrain")}</small></button>
          </div>
          </Show>
          <div class="noise-options noise-reduction-panel">
            <Show when={props.adaptiveNoise}>
              <AdaptiveEnvironmentCards selected={props.noiseEnvironment} onSelect={props.onNoiseEnvironment} />
            </Show>
            <div class="noise-adaptive-row"><div><strong>{t("home.adaptive")}</strong><small>{t("home.autoEnvironment")}</small></div><label class="toggle sm"><input type="checkbox" disabled={!props.adaptiveSupported} checked={props.adaptiveNoise} onChange={(e) => props.onAdaptiveNoise((e.currentTarget as HTMLInputElement).checked)} /><span class="slider" /></label></div>
            <Show when={props.adaptiveNoise} fallback={<div class="noise-levels"><div class="noise-level-heading"><span>{t("home.noiseLevel")}</span><strong>{props.noiseLevel}/{props.noiseMaxLevel}</strong></div><div class="noise-level-buttons">{Array.from({ length: props.noiseMaxLevel }, (_, i) => i + 1).map((level) => <button type="button" class={props.noiseLevel === level ? "active" : ""} aria-pressed={props.noiseLevel === level} onClick={() => props.onNoiseLevel(level)}>{level}</button>)}</div></div>}>
              <div class="noise-environments">{[[102, t("home.indoor"), t("home.homeOffice")], [103, t("home.outdoor"), t("home.streetPark")], [101, t("home.commuting"), t("home.subwayBus")], [108, t("home.inTransit"), t("home.planeTrain")]].map(([id, title, detail]) => <button type="button" class={props.noiseEnvironment === id ? "active" : ""} onClick={() => props.onNoiseEnvironment(id as NoiseEnvironment)}><span>{title}</span><small>{detail}</small></button>)}</div>
            </Show>
          </div>
        </Show>
        <Show when={false}>
          <div class="home-anc-level">
            <div class="row">
              <span>{t("home.level")}</span>
              <strong>{props.ancStrength}%</strong>
            </div>
            <input
              type="range"
              min="0"
              max="100"
              step="5"
              value={props.ancStrength}
              onInput={(e) =>
                props.onAncStrength(
                  Number((e.currentTarget as HTMLInputElement).value)
                )
              }
            />
          </div>
        </Show>
      </div>

      {/* Spatial on home root */}
      <div class="home-feature-card">
        <div class="list-row">
          <span class="list-ico">
            <IconSpatial size={22} />
          </span>
          <div class="list-text">
            <span class="list-title">{t("home.spatial")}</span>
            <span class="list-sub">{t("listen.spatialHint")}</span>
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
          <div class="home-seg">
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
      </div>

      {/* Main list */}
      <div class="home-list">
        <div class="home-list-card">
          <div class="list-row">
            <span class="list-ico">
              <IconGame size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">{t("home.gameMode")}</span>
              <span class="list-sub">{t("home.lowLatency")}</span>
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

          <button type="button" class="list-row action" onClick={() => props.onOpenEq()}>
            <span class="list-ico">
              <IconEq size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">EQ</span>
              <span class="list-sub">{props.eqLabel}</span>
            </div>
            <span class="list-chev">›</span>
          </button>

          <button
            type="button"
            class="list-row action"
            onClick={() => props.onSoundFit()}
          >
            <span class="list-ico list-ico-text">SF</span>
            <div class="list-text">
              <span class="list-title">SoundFit</span>
              <span class="list-sub">{t("home.hearingPersonalization")}</span>
            </div>
            <span class="list-chev">›</span>
          </button>

          <button type="button" class="list-row action" onClick={() => props.onOpenMore()}>
            <span class="list-ico">
              <IconMore size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">{t("home.moreAudio")}</span>
              <span class="list-sub">{t("home.bassLdac")}</span>
            </div>
            <span class="list-chev">›</span>
          </button>
        </div>

        <div class="home-list-card">
          <button type="button" class={`list-row action find-row ${props.findActive ? "active" : ""}`} onClick={() => props.onFindBuds()} aria-pressed={props.findActive}>
            <span class="list-ico"><IconFind size={22} /></span>
            <div class="list-text">
              <span class="list-title">{props.findActive ? t("home.finding") : t("home.find")}</span>
              <span class="list-sub">{t("home.playSound")}</span>
            </div>
            <span class="list-chev">›</span>
          </button>
          <button
            type="button"
            class="list-row action"
            onClick={() => props.onOpenSettings()}
          >
            <span class="list-ico">
              <IconSettings size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">{t("nav.settings")}</span>
            </div>
            <span class="list-chev">›</span>
          </button>
          <button
            type="button"
            class="list-row action danger"
            onClick={() => props.onDisconnect()}
          >
            <span class="list-ico">
              <IconPower size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">{t("home.disconnect")}</span>
            </div>
          </button>
        </div>
      </div>
      </div>{/* home-body */}
    </div>
  );
};

export default HomePanel;
