/**
 * Home — noise tiles; spatial + SoundFit on root; find/battery as action chips
 */
import { Component, Show } from "solid-js";
import type { BatteryData } from "./Battery";
import type { AncMode, NoiseEnvironment, SpatialMode, TransparencyMode } from "../lib/device";
import type { LinkHealth } from "../lib/ble";
import { resolveDeviceImage } from "../lib/deviceImages";
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
    <button type="button" class={props.selected === 102 ? "active" : ""} onClick={() => props.onSelect(102)}><IconOffice size={24} /><span><strong>Trong nhà</strong><small>Nhà / văn phòng</small></span></button>
    <button type="button" class={props.selected === 103 ? "active" : ""} onClick={() => props.onSelect(103)}><IconOutdoor size={24} /><span><strong>Ngoài trời</strong><small>Đường phố / công viên</small></span></button>
    <button type="button" class={props.selected === 101 ? "active" : ""} onClick={() => props.onSelect(101)}><IconTransit size={24} /><span><strong>Di chuyển</strong><small>Tàu điện ngầm / xe buýt</small></span></button>
    <button type="button" class={props.selected === 108 ? "active" : ""} onClick={() => props.onSelect(108)}><IconFlight size={24} /><span><strong>Đang di chuyển</strong><small>Máy bay / tàu hỏa</small></span></button>
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
        return "Đã kết nối";
      case "waiting":
        return "Chờ dữ liệu";
      case "demo":
        return "Demo";
      case "dead":
        return "Mất link";
      default:
        return "Đã kết nối";
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
          <div class="tag">L</div>
        </div>
        <div class="home-batt-cell">
          <div class={`pct ${pctClass(props.battery.case)}`}>
            {fmt(props.battery.case)}
          </div>
          <div class="tag">Case</div>
        </div>
        <div class="home-batt-cell">
          <div class={`pct ${pctClass(props.battery.right)}`}>
            {fmt(props.battery.right)}
          </div>
          <div class="tag">R</div>
        </div>
      </div>

      {/* Noise — only square tiles */}
      <div>
        <p class="home-section-label">Tiếng ồn</p>
        <div class="noise-tiles">
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "off" ? "active" : ""}`}
            onClick={() => props.onAncMode("off")}
          >
            <IconNormal size={28} />
            <span>Thường</span>
          </button>
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "transparency" ? "active" : ""}`}
            onClick={() => props.onAncMode("transparency")}
          >
            <IconAmbient size={28} />
            <span>Xuyên âm</span>
          </button>
          <button
            type="button"
            class={`noise-tile ${props.ancMode === "anc" ? "active" : ""}`}
            disabled={!props.noiseSupported}
            onClick={() => props.onAncMode("anc")}
          >
            <IconAnc size={28} />
            <span>Giảm ồn</span>
          </button>
        </div>
        <Show when={props.ancMode === "transparency"}>
          <div class="noise-options" aria-label="Tùy chọn xuyên âm">
            <button type="button" class={props.transparencyMode === "full" ? "active" : ""} onClick={() => props.onTransparencyMode("full")}><span>Xuyên âm hoàn toàn</span><small>Mặc định</small></button>
            <button type="button" class={props.transparencyMode === "voice" ? "active" : ""} onClick={() => props.onTransparencyMode("voice")}><span>Chế độ giọng nói</span><small>Ưu tiên giọng người</small></button>
          </div>
        </Show>
        <Show when={props.ancMode === "anc"}>
          <Show when={props.adaptiveNoise}>
          <div class="noise-environments noise-environments-new">
            <button type="button" class={props.noiseEnvironment === 102 ? "active" : ""} onClick={() => props.onNoiseEnvironment(102)}><IconOffice size={28} /><strong>Trong nhà</strong><small>Nhà / văn phòng</small></button>
            <button type="button" class={props.noiseEnvironment === 103 ? "active" : ""} onClick={() => props.onNoiseEnvironment(103)}><IconOutdoor size={28} /><strong>Ngoài trời</strong><small>Đường phố / công viên</small></button>
            <button type="button" class={props.noiseEnvironment === 101 ? "active" : ""} onClick={() => props.onNoiseEnvironment(101)}><IconTransit size={28} /><strong>Di chuyển</strong><small>Tàu điện ngầm / xe buýt</small></button>
            <button type="button" class={props.noiseEnvironment === 108 ? "active" : ""} onClick={() => props.onNoiseEnvironment(108)}><IconFlight size={28} /><strong>Đang di chuyển</strong><small>Máy bay / tàu hỏa</small></button>
          </div>
          </Show>
          <div class="noise-options noise-reduction-panel">
            <Show when={props.adaptiveNoise}>
              <AdaptiveEnvironmentCards selected={props.noiseEnvironment} onSelect={props.onNoiseEnvironment} />
            </Show>
            <div class="noise-adaptive-row"><div><strong>Tự động thích ứng</strong><small>Tự điều chỉnh theo môi trường</small></div><label class="toggle sm"><input type="checkbox" disabled={!props.adaptiveSupported} checked={props.adaptiveNoise} onChange={(e) => props.onAdaptiveNoise((e.currentTarget as HTMLInputElement).checked)} /><span class="slider" /></label></div>
            <Show when={props.adaptiveNoise} fallback={<div class="noise-levels"><div class="noise-level-heading"><span>Mức giảm tiếng ồn</span><strong>{props.noiseLevel}/{props.noiseMaxLevel}</strong></div><div class="noise-level-buttons">{Array.from({ length: props.noiseMaxLevel }, (_, i) => i + 1).map((level) => <button type="button" class={props.noiseLevel === level ? "active" : ""} aria-pressed={props.noiseLevel === level} onClick={() => props.onNoiseLevel(level)}>{level}</button>)}</div></div>}>
              <div class="noise-environments">{[[102, "Trong nhà", "Văn phòng"], [103, "Ngoài trời", "Đường phố · công viên"], [101, "Di chuyển", "Tàu điện ngầm · xe buýt"], [108, "Đang di chuyển", "Máy bay · tàu hỏa"]].map(([id, title, detail]) => <button type="button" class={props.noiseEnvironment === id ? "active" : ""} onClick={() => props.onNoiseEnvironment(id as NoiseEnvironment)}><span>{title}</span><small>{detail}</small></button>)}</div>
            </Show>
          </div>
        </Show>
        <Show when={false}>
          <div class="home-anc-level">
            <div class="row">
              <span>Mức</span>
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
            <span class="list-title">Âm học không gian</span>
            <span class="list-sub">Spatial / Panoramic</span>
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
              Âm nhạc
            </button>
            <button
              type="button"
              class={props.spatialMode === "cinema" ? "active" : ""}
              onClick={() => props.onSpatialMode("cinema")}
            >
              Điện ảnh
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
              <span class="list-title">Game mode</span>
              <span class="list-sub">Độ trễ thấp</span>
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
              <span class="list-sub">Cá nhân hóa thính lực</span>
            </div>
            <span class="list-chev">›</span>
          </button>

          <button type="button" class="list-row action" onClick={() => props.onOpenMore()}>
            <span class="list-ico">
              <IconMore size={22} />
            </span>
            <div class="list-text">
              <span class="list-title">Âm thanh khác</span>
              <span class="list-sub">Bass · LDAC · …</span>
            </div>
            <span class="list-chev">›</span>
          </button>
        </div>

        <div class="home-list-card">
          <button type="button" class={`list-row action find-row ${props.findActive ? "active" : ""}`} onClick={() => props.onFindBuds()} aria-pressed={props.findActive}>
            <span class="list-ico"><IconFind size={22} /></span>
            <div class="list-text">
              <span class="list-title">{props.findActive ? "Đang tìm tai nghe" : "Tìm tai nghe"}</span>
              <span class="list-sub">Phát âm thanh để xác định vị trí</span>
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
              <span class="list-title">Cài đặt</span>
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
              <span class="list-title">Ngắt kết nối</span>
            </div>
          </button>
        </div>
      </div>
      </div>{/* home-body */}
    </div>
  );
};

export default HomePanel;
