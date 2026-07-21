/**
 * Home — noise tiles; spatial + SoundFit on root; find/battery as action chips
 */
import { Component, Show } from "solid-js";
import type { BatteryData } from "./Battery";
import type { AncMode, SpatialMode } from "../lib/device";
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
  IconBattery,
  IconSpatial,
} from "./Icons";

interface Props {
  name: string;
  modelId?: string | null;
  battery: BatteryData;
  link: LinkHealth;
  ancMode: AncMode;
  ancStrength: number;
  gameMode: boolean;
  spatialOn: boolean;
  spatialMode: SpatialMode;
  eqLabel: string;
  onAncMode: (m: AncMode) => void;
  onAncStrength: (v: number) => void;
  onGameMode: (on: boolean) => void;
  onFindBuds: () => void;
  onRefreshBattery: () => void;
  onOpenMore: () => void;
  onOpenSettings: () => void;
  onDisconnect: () => void;
  onOpenEq: () => void;
  onSpatialOn: (on: boolean) => void;
  onSpatialMode: (m: SpatialMode) => void;
  onSoundFit: () => void;
}

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
  const visual = () => resolveDeviceImage(props.modelId, props.name);
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

      {/* Find / Battery — distinct action chips */}
      <div class="home-actions">
        <button
          type="button"
          class="action-chip"
          onClick={() => props.onFindBuds()}
        >
          <IconFind size={20} />
          <span>Tìm tai</span>
        </button>
        <button
          type="button"
          class="action-chip"
          onClick={() => props.onRefreshBattery()}
        >
          <IconBattery size={20} />
          <span>Làm mới pin</span>
        </button>
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
            onClick={() => props.onAncMode("anc")}
          >
            <IconAnc size={28} />
            <span>Giảm ồn</span>
          </button>
        </div>
        <Show when={props.ancMode === "anc"}>
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
