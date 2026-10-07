import { requestToggle } from "../lib/confirmedToggle";
/**
 * Home — model-supported noise, spatial and feature controls
 */
import { Component, Show } from "solid-js";
import type { BatteryData } from "./Battery";
import type { AncMode, NoiseEnvironment, SpatialMode, TransparencyMode } from "../lib/device";
import type { LinkHealth } from "../lib/ble";
import { createDeviceVisual, handleDeviceImageError } from "../lib/deviceImages";
import { t } from "../lib/i18n";
import OperationStatus from "./OperationStatus";
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
  ancMode: AncMode | null;
  ancPending?: boolean;
  ancError?: string | null;
  transparencyMode: TransparencyMode | null;
  adaptiveNoise: boolean | null;
  noiseEnvironment: NoiseEnvironment | null;
  noiseLevel: number | null;
  listeningSupported: boolean;
  noiseMaxLevel: number;
  noiseSupported: boolean;
  adaptiveSupported: boolean;
  transparencyVoiceSupported: boolean;
  gameSupported: boolean;
  eqSupported: boolean;
  findSupported: boolean;
  spatialSupported: boolean;
  moreSupported: boolean;
  gestureSupported: boolean;
  inEarSupported: boolean;
  inEarOn: boolean | null;
  inEarPending?: boolean;
  inEarError?: string | null;
  multipointSupported: boolean;
  multipointOn: boolean | null;
  multipointPending?: boolean;
  multipointError?: string | null;
  restoreSupported: boolean;
  restorePending?: boolean;
  restoreError?: string | null;
  adaptiveLrSupported: boolean;
  adaptiveLrOn: boolean | null;
  adaptiveLrPending?: boolean;
  adaptiveLrError?: string | null;
  gameMode: boolean | null;
  gamePending?: boolean;
  gameError?: string | null;
  findActive: boolean;
  spatialPending?: boolean;
  spatialError?: string | null;
  spatialOn: boolean | null;
  spatialMode: SpatialMode | null;
  eqLabel: string;
  onAncMode: (m: AncMode) => void;
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
  onOpenGestures: () => void;
  onInEar: (enabled: boolean) => void;
  onMultipoint: (enabled: boolean) => void;
  onAdaptiveLr: (enabled: boolean) => void;
  onRestore: () => void;
  onSpatialOn: (on: boolean) => void;
  onSpatialMode: (m: SpatialMode) => void;
}

const AdaptiveEnvironmentCards = (props: {
  selected: NoiseEnvironment | null;
  disabled?: boolean;
  onSelect: (value: NoiseEnvironment) => void;
}) => (
  <div class="noise-environments noise-environments-card">
    <button type="button" disabled={props.disabled} class={props.selected === 102 ? "active" : ""} aria-pressed={props.selected === 102} onClick={() => props.onSelect(102)}><IconOffice size={24} /><span><strong>{t("home.indoor")}</strong><small>{t("home.homeOffice")}</small></span></button>
    <button type="button" disabled={props.disabled} class={props.selected === 103 ? "active" : ""} aria-pressed={props.selected === 103} onClick={() => props.onSelect(103)}><IconOutdoor size={24} /><span><strong>{t("home.outdoor")}</strong><small>{t("home.streetPark")}</small></span></button>
    <button type="button" disabled={props.disabled} class={props.selected === 101 ? "active" : ""} aria-pressed={props.selected === 101} onClick={() => props.onSelect(101)}><IconTransit size={24} /><span><strong>{t("home.commuting")}</strong><small>{t("home.subwayBus")}</small></span></button>
    <button type="button" disabled={props.disabled} class={props.selected === 108 ? "active" : ""} aria-pressed={props.selected === 108} onClick={() => props.onSelect(108)}><IconFlight size={24} /><span><strong>{t("home.inTransit")}</strong><small>{t("home.planeTrain")}</small></span></button>
  </div>
);

function pctClass(p: number | null) {
  if (p === null) return "unk";
  if (p <= 20) return "low";
  if (p <= 50) return "mid";
  return "ok";
}
function fmt(p: number | null) {
  return p === null ? "—" : `${Math.min(100, p)}%`;
}

const HomePanel: Component<Props> = (props) => {
  const visual = createDeviceVisual(() => props.imageUrl, () => props.name);
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
      case "offline":
        return t("device.offline");
      default:
        return "—";
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
            <div class="home-sticky-status" role="status" aria-live="polite">
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
          data-image-url={visual().sourceUrl}
          decoding="async"
          onError={handleDeviceImageError}
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
      <Show when={props.listeningSupported}>
      <div>
        <p class="home-section-label">{t("home.noise")}</p>
        <div class="noise-tiles" aria-busy={props.ancPending}>
          <button
            type="button"
            disabled={!props.listeningSupported || props.ancPending}
            class={`noise-tile ${props.ancMode === "off" ? "active" : ""}`}
            aria-pressed={props.ancMode === "off"}
            onClick={() => props.onAncMode("off")}
          >
            <IconNormal size={28} />
            <span>{t("home.normal")}</span>
          </button>
          <button
            type="button"
            disabled={!props.listeningSupported || props.ancPending}
            class={`noise-tile ${props.ancMode === "transparency" ? "active" : ""}`}
            aria-pressed={props.ancMode === "transparency"}
            onClick={() => props.onAncMode("transparency")}
          >
            <IconAmbient size={28} />
            <span>{t("home.transparency")}</span>
          </button>
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "anc" ? "active" : ""}`}
            disabled={!props.listeningSupported || !props.noiseSupported || props.ancPending}
            aria-pressed={props.ancMode === "anc"}
            onClick={() => props.onAncMode("anc")}
          >
            <IconAnc size={28} />
            <span>{t("home.anc")}</span>
          </button>
        </div>
        <OperationStatus pending={props.ancPending} error={props.ancError} />
        <Show when={props.ancMode === "transparency"}>
          <div class="noise-options" role="group" aria-label={t("home.transparencyOptions")}>
            <button type="button" disabled={props.ancPending} class={props.transparencyMode === "full" ? "active" : ""} aria-pressed={props.transparencyMode === "full"} onClick={() => props.onTransparencyMode("full")}><span>{t("home.fullTransparency")}</span><small>{t("home.default")}</small></button>
            <Show when={props.transparencyVoiceSupported}>
              <button type="button" disabled={props.ancPending} class={props.transparencyMode === "voice" ? "active" : ""} aria-pressed={props.transparencyMode === "voice"} onClick={() => props.onTransparencyMode("voice")}><span>{t("home.voiceMode")}</span><small>{t("home.prioritizeVoice")}</small></button>
            </Show>
          </div>
        </Show>
        <Show when={props.ancMode === "anc"}>
          <div class="noise-options noise-reduction-panel">
            <Show when={props.adaptiveNoise !== null}>
              <div class="noise-adaptive-row"><div><strong>{t("home.adaptive")}</strong><small>{t("home.autoEnvironment")}</small></div><label class="toggle sm"><input type="checkbox" disabled={!props.adaptiveSupported || props.ancPending} checked={props.adaptiveNoise === true} onChange={(e) => requestToggle(e.currentTarget, props.adaptiveNoise === true, props.onAdaptiveNoise)} /><span class="slider" /></label></div>
            </Show>
            <Show when={props.adaptiveNoise === false}>
              <div class="noise-levels"><div class="noise-level-heading"><span>{t("home.noiseLevel")}</span><strong>{props.noiseLevel === null ? "—" : `${props.noiseLevel}/${props.noiseMaxLevel}`}</strong></div><div class="noise-level-buttons">{Array.from({ length: props.noiseMaxLevel }, (_, i) => i + 1).map((level) => <button type="button" disabled={props.ancPending} class={props.noiseLevel === level ? "active" : ""} aria-pressed={props.noiseLevel === level} onClick={() => props.onNoiseLevel(level)}>{level}</button>)}</div></div>
            </Show>
            <Show when={props.adaptiveNoise === true}>
              <AdaptiveEnvironmentCards selected={props.noiseEnvironment} disabled={props.ancPending} onSelect={props.onNoiseEnvironment} />
            </Show>
          </div>
        </Show>
      </div>

      </Show>

      {/* Spatial on home root */}
      <Show when={props.spatialSupported}>
      <div class="home-feature-card">
        <div class="list-row">
          <span class="list-ico">
            <IconSpatial size={22} />
          </span>
          <div class="list-text">
            <span class="list-title">{t("home.spatial")}</span>
            <span class="list-sub">
              {t("listen.spatialHint")}
            </span>
          </div>
          <label class="toggle sm">
            <input
              type="checkbox"
              disabled={props.spatialPending || (props.spatialOn !== true && props.spatialMode === null)}
              aria-label={t("home.spatial")}
              aria-checked={props.spatialOn === null ? "mixed" : props.spatialOn}
              checked={props.spatialOn === true}
              onChange={(e) =>
                requestToggle(e.currentTarget, props.spatialOn === true, props.onSpatialOn)
              }
            />
            <span class="slider" />
          </label>
        </div>
        <OperationStatus pending={props.spatialPending} error={props.spatialError} />
        <div class="home-seg">
          <button
            type="button"
            disabled={props.spatialPending}
            class={props.spatialMode === "music" ? "active" : ""}
            aria-pressed={props.spatialMode === "music"}
            onClick={() => props.onSpatialMode("music")}
          >
            {t("listen.music")}
          </button>
          <button
            type="button"
            disabled={props.spatialPending}
            class={props.spatialMode === "cinema" ? "active" : ""}
            aria-pressed={props.spatialMode === "cinema"}
            onClick={() => props.onSpatialMode("cinema")}
          >
            {t("listen.cinema")}
          </button>
        </div>
      </div>

      </Show>

      {/* Main list */}
      <div class="home-list">
        <div class="home-list-card">
          <Show when={props.gameSupported}>
          <div class="list-row">
            <span class="list-ico">
              <IconGame size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">{t("home.gameMode")}</span>
              <span class="list-sub">{t("home.lowLatency")}</span>
              <OperationStatus pending={props.gamePending} error={props.gameError} />
            </div>
            <label class="toggle sm">
              <input
                type="checkbox"
                aria-checked={props.gameMode === null ? "mixed" : props.gameMode}
                checked={props.gameMode === true}
                disabled={props.gamePending}
                aria-busy={props.gamePending}
                aria-label={t("home.gameMode")}
                onChange={(e) =>
                  requestToggle(e.currentTarget, props.gameMode === true, props.onGameMode)
                }
              />
              <span class="slider" />
            </label>
          </div>

          </Show>

          <Show when={props.eqSupported}>
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
          </Show>

          <Show when={props.inEarSupported}>
          <div class="list-row">
            <span class="list-ico list-ico-text">IE</span>
            <div class="list-text">
              <span class="list-title">{t("gesture.inEar")}</span>
              <span class="list-sub">{t("gesture.inEarHint")}</span>
              <OperationStatus pending={props.inEarPending} error={props.inEarError} />
            </div>
            <label class="toggle sm">
              <input
                type="checkbox"
                disabled={props.inEarPending}
                aria-checked={props.inEarOn === null ? "mixed" : props.inEarOn}
                checked={props.inEarOn === true}
                aria-label={t("gesture.inEar")}
                onChange={(e) => requestToggle(e.currentTarget, props.inEarOn === true, props.onInEar)}
              />
              <span class="slider" />
            </label>
          </div>
          </Show>

          <Show when={props.gestureSupported}>
          <button type="button" class="list-row action" onClick={() => props.onOpenGestures()}>
            <span class="list-ico list-ico-text">G</span>
            <div class="list-text">
              <span class="list-title">{t("gesture.title")}</span>
              <span class="list-sub">{t("gesture.entryHint")}</span>
            </div>
            <span class="list-chev">›</span>
          </button>
          </Show>

          <Show when={props.multipointSupported}>
          <div class="list-row">
            <span class="list-ico list-ico-text">MP</span>
            <div class="list-text">
              <span class="list-title">{t("multipoint.title")}</span>
              <span class="list-sub">{t("multipoint.hint")}</span>
              <OperationStatus pending={props.multipointPending} error={props.multipointError} />
            </div>
            <label class="toggle sm">
              <input
                type="checkbox"
                disabled={props.multipointPending}
                aria-checked={props.multipointOn === null ? "mixed" : props.multipointOn}
                checked={props.multipointOn === true}
                aria-label={t("multipoint.title")}
                onChange={(e) => requestToggle(e.currentTarget, props.multipointOn === true, props.onMultipoint)}
              />
              <span class="slider" />
            </label>
          </div>
          </Show>

          <Show when={props.moreSupported}>
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
          </Show>
        </div>

        <div class="home-list-card">
          <Show when={props.findSupported}>
          <button type="button" class={`list-row action find-row ${props.findActive ? "active" : ""}`} onClick={() => props.onFindBuds()} aria-pressed={props.findActive}>
            <span class="list-ico"><IconFind size={22} /></span>
            <div class="list-text">
              <span class="list-title">{props.findActive ? t("home.finding") : t("home.find")}</span>
              <span class="list-sub">{t("home.playSound")}</span>
            </div>
            <span class="list-chev">›</span>
          </button>
          </Show>
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
          <Show when={props.adaptiveLrSupported}>
          <div class="list-row">
            <span class="list-ico list-ico-text">AL</span>
            <div class="list-text">
              <span class="list-title">{t("adaptiveLr.title")}</span>
              <span class="list-sub">{t("adaptiveLr.hint")}</span>
              <OperationStatus pending={props.adaptiveLrPending} error={props.adaptiveLrError} />
            </div>
            <label class="toggle sm">
              <input
                type="checkbox"
                disabled={props.adaptiveLrPending}
                aria-checked={props.adaptiveLrOn === null ? "mixed" : props.adaptiveLrOn}
                checked={props.adaptiveLrOn === true}
                aria-label={t("adaptiveLr.title")}
                onChange={(e) => requestToggle(e.currentTarget, props.adaptiveLrOn === true, props.onAdaptiveLr)}
              />
              <span class="slider" />
            </label>
          </div>
          </Show>
          <Show when={props.restoreSupported}>
          <button
            type="button"
            class="list-row action danger"
            disabled={props.restorePending}
            onClick={() => props.onRestore()}
          >
            <span class="list-ico list-ico-text">R</span>
            <div class="list-text">
              <span class="list-title">{t("restore.title")}</span>
              <span class="list-sub">{t("restore.hint")}</span>
              <OperationStatus pending={props.restorePending} error={props.restoreError} />
            </div>
            <span class="list-chev">›</span>
          </button>
          </Show>

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
