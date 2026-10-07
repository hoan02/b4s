import { requestToggle } from "../lib/confirmedToggle";
/**
 * Home — model-supported noise, spatial and feature controls
 */
import { Component, Show } from "solid-js";
import type { BatteryData } from "../lib/battery";
import type { AncMode, NoiseEnvironment, SpatialMode, TransparencyMode } from "../lib/device";
import type { LinkHealth } from "../lib/ble";
import { createDeviceVisual, handleDeviceImageError } from "../lib/deviceImages";
import { t } from "../lib/i18n";
import OperationStatus from "./OperationStatus";
import { NavRow, ToggleRow } from "./ListRows";
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
  IconEar,
  IconTouch,
  IconLink,
  IconBalance,
  IconWind,
  IconReset,
  IconBolt,
} from "./Icons";

interface Props {
  name: string;
  /** Capability keys the reviewed profile marks Experimental-only. */
  experimentalFeatures?: string[];
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
  windNoiseSupported: boolean;
  windNoiseOn: boolean | null;
  windNoisePending?: boolean;
  windNoiseError?: string | null;
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
  onWindNoise: (enabled: boolean) => void;
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

const BatteryCell = (props: { label: string; value: number | null; charging?: boolean }) => (
  <div class="home-batt-cell">
    <div class={`pct ${pctClass(props.value)}`}>
      {fmt(props.value)}
      <Show when={props.charging}>
        <IconBolt size={13} class="batt-bolt" />
      </Show>
    </div>
    <div class="batt-bar" aria-hidden="true">
      <span class={pctClass(props.value)} style={{ width: `${Math.min(100, props.value ?? 0)}%` }} />
    </div>
    <div class="tag">{props.label}</div>
  </div>
);

const HomePanel: Component<Props> = (props) => {
  const isExperimental = (key: string) => props.experimentalFeatures?.includes(key) ?? false;
  const soundVisible = () =>
    props.gameSupported || props.eqSupported || props.windNoiseSupported || props.moreSupported;
  const controlsVisible = () =>
    props.gestureSupported || props.inEarSupported || props.multipointSupported || props.adaptiveLrSupported;
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

      <div class="home-batt" role="group" aria-label={t("home.battery")}>
        <BatteryCell label={t("home.indicatorLeft")} value={props.battery.left} charging={props.battery.leftCharging} />
        <BatteryCell label={t("home.indicatorCase")} value={props.battery.case} charging={props.battery.caseCharging} />
        <BatteryCell label={t("home.indicatorRight")} value={props.battery.right} charging={props.battery.rightCharging} />
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

      {/* Main list: grouped by purpose, one row structure everywhere */}
      <div class="home-list">
        <Show when={soundVisible()}>
        <div class="home-group">
          <p class="home-section-label">{t("home.groupSound")}</p>
          <div class="home-list-card">
            <Show when={props.gameSupported}>
              <ToggleRow
                icon={<IconGame size={22} />}
                title={t("home.gameMode")}
                hint={t("home.lowLatency")}
                checked={props.gameMode}
                pending={props.gamePending}
                error={props.gameError}
                onChange={props.onGameMode}
              />
            </Show>
            <Show when={props.eqSupported}>
              <NavRow icon={<IconEq size={22} />} title="EQ" hint={props.eqLabel} onClick={props.onOpenEq} />
            </Show>
            <Show when={props.windNoiseSupported}>
              <ToggleRow
                icon={<IconWind size={22} />}
                title={t("windNoise.title")}
                hint={t("windNoise.hint")}
                experimental={isExperimental("windNoise")}
                checked={props.windNoiseOn}
                pending={props.windNoisePending}
                error={props.windNoiseError}
                onChange={props.onWindNoise}
              />
            </Show>
            <Show when={props.moreSupported}>
              <NavRow
                icon={<IconMore size={22} />}
                title={t("home.moreAudio")}
                hint={t("home.bassLdac")}
                onClick={props.onOpenMore}
              />
            </Show>
          </div>
        </div>
        </Show>

        <Show when={controlsVisible()}>
        <div class="home-group">
          <p class="home-section-label">{t("home.groupControls")}</p>
          <div class="home-list-card">
            <Show when={props.gestureSupported}>
              <NavRow
                icon={<IconTouch size={22} />}
                title={t("gesture.title")}
                hint={t("gesture.entryHint")}
                experimental={isExperimental("gesture")}
                onClick={props.onOpenGestures}
              />
            </Show>
            <Show when={props.inEarSupported}>
              <ToggleRow
                icon={<IconEar size={22} />}
                title={t("gesture.inEar")}
                hint={t("gesture.inEarHint")}
                experimental={isExperimental("inEar")}
                checked={props.inEarOn}
                pending={props.inEarPending}
                error={props.inEarError}
                onChange={props.onInEar}
              />
            </Show>
            <Show when={props.multipointSupported}>
              <ToggleRow
                icon={<IconLink size={22} />}
                title={t("multipoint.title")}
                hint={t("multipoint.hint")}
                experimental={isExperimental("multipoint")}
                checked={props.multipointOn}
                pending={props.multipointPending}
                error={props.multipointError}
                onChange={props.onMultipoint}
              />
            </Show>
            <Show when={props.adaptiveLrSupported}>
              <ToggleRow
                icon={<IconBalance size={22} />}
                title={t("adaptiveLr.title")}
                hint={t("adaptiveLr.hint")}
                experimental={isExperimental("adaptiveLr")}
                checked={props.adaptiveLrOn}
                pending={props.adaptiveLrPending}
                error={props.adaptiveLrError}
                onChange={props.onAdaptiveLr}
              />
            </Show>
          </div>
        </div>
        </Show>

        <div class="home-group">
          <p class="home-section-label">{t("home.groupDevice")}</p>
          <div class="home-list-card">
            <Show when={props.findSupported}>
              <NavRow
                icon={<IconFind size={22} />}
                title={props.findActive ? t("home.finding") : t("home.find")}
                hint={t("home.playSound")}
                active={props.findActive}
                onClick={props.onFindBuds}
              />
            </Show>
            <NavRow icon={<IconSettings size={22} />} title={t("nav.settings")} onClick={props.onOpenSettings} />
            <Show when={props.restoreSupported}>
              <NavRow
                icon={<IconReset size={22} />}
                title={t("restore.title")}
                hint={t("restore.hint")}
                experimental={isExperimental("restoreDefaults")}
                danger
                disabled={props.restorePending}
                pending={props.restorePending}
                error={props.restoreError}
                onClick={props.onRestore}
              />
            </Show>
            <NavRow
              icon={<IconPower size={22} />}
              title={t("home.disconnect")}
              danger
              chevron={false}
              onClick={props.onDisconnect}
            />
          </div>
        </div>
      </div>
      </div>{/* home-body */}
    </div>
  );
};

export default HomePanel;
