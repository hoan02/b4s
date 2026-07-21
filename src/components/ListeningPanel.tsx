/**
 * Listening controls (detail screen). Protocol BA34 / BA43 / BA24 / BA5E / BA02…
 */
import { Component, For, Show, createSignal } from "solid-js";
import type { AncMode, EqPresetId, SpatialMode } from "../lib/device";

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
  { id: "off", label: "Bình thường", sub: "Normal" },
  { id: "transparency", label: "Xuyên âm", sub: "Ambient" },
  { id: "anc", label: "Giảm ồn", sub: "ANC" },
];

const EQ_LIST: { id: EqPresetId; label: string }[] = [
  { id: "classic", label: "Classic" },
  { id: "bass", label: "Powerful Bass" },
  { id: "hifi", label: "Hi-Fi Live" },
  { id: "pop", label: "Pop" },
  { id: "jazz", label: "Jazz Rock" },
  { id: "classical", label: "Trường cổ điển" },
  { id: "clear", label: "Clear Treble" },
  { id: "acoustic", label: "Acoustic" },
  { id: "bassReduce", label: "Giảm âm trầm" },
  { id: "trebleReduce", label: "Giảm âm bổng" },
  { id: "voice", label: "Voice" },
];

const ListeningPanel: Component<Props> = (props) => {
  const [soundOpen, setSoundOpen] = createSignal(true);

  return (
    <div class="listen-panel">
      {/* —— Noise control —— */}
      <section class="listen-card">
        <div class="listen-card-head">
          <h3>Kiểm soát tiếng ồn</h3>
          <span class="listen-card-hint">Noise Control</span>
        </div>
        <div class="noise-modes">
          <For each={NOISE}>
            {(m) => (
              <button
                type="button"
                class={`noise-btn mode-${m.id} ${props.ancMode === m.id ? "active" : ""}`}
                onClick={() => props.onAncMode(m.id)}
              >
                <span class="noise-label">{m.label}</span>
                <span class="noise-sub">{m.sub}</span>
              </button>
            )}
          </For>
        </div>
        <Show when={props.ancMode === "anc"}>
          <div class="anc-level">
            <div class="anc-level-row">
              <span>Mức giảm ồn</span>
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
            <h3>Âm học không gian</h3>
            <span class="listen-card-hint">Spatial / Panoramic Sound</span>
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
      </section>

      {/* —— EQ —— */}
      <section class="listen-card">
        <div class="listen-card-head">
          <h3>EQ</h3>
          <span class="listen-card-hint">Equalizer</span>
        </div>
        <div class="eq-chips">
          <For each={EQ_LIST}>
            {(eq) => (
              <button
                type="button"
                class={`eq-chip ${props.eqActive === eq.id ? "active" : ""}`}
                onClick={() => props.onEq(eq.id)}
              >
                {eq.label}
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
            <p>Cá nhân hóa âm thanh theo thính lực</p>
          </div>
        </div>
        <button type="button" class="row-action" onClick={() => props.onSoundFit?.()}>
          Mở
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
            <h3>Cài đặt âm thanh</h3>
            <span class="listen-card-hint">Sound settings</span>
          </div>
          <span class="chev">{soundOpen() ? "−" : "+"}</span>
        </button>

        <Show when={soundOpen()}>
          <div class="sound-block">
            <h4>Âm thanh điện tử</h4>
            <div class="setting-row">
              <div>
                <span class="setting-title">Tăng âm trầm</span>
                <span class="setting-desc">Bass boost · 0–3</span>
              </div>
                <div class="level-pills" aria-label="Mức tăng âm trầm">
                <For each={[{ value: 0, label: "Tắt" }, { value: 1, label: "Nhẹ" }, { value: 2, label: "Vừa" }, { value: 3, label: "Mạnh" }]}>
                  {(lv) => (
                    <button
                      type="button"
                      class={props.bassBoost === lv.value ? "active" : ""}
                      aria-pressed={props.bassBoost === lv.value}
                      onClick={() => props.onBassBoost(lv.value)}
                    >
                      <span>{lv.label}</span>
                      <small>{lv.value}</small>
                    </button>
                  )}
                </For>
              </div>
            </div>
            <div class="setting-row">
              <div>
                <span class="setting-title">Độ trễ thấp</span>
                <span class="setting-desc">Game Mode</span>
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
            <h4>Âm thanh tự nhiên</h4>
            <div class="setting-row">
              <div>
                <span class="setting-title">LDAC</span>
                <span class="setting-desc">Âm thanh độ phân giải cao</span>
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
            <p class="setting-note">
              LDAC thường do stack Bluetooth Windows/Android quản lý — toggle lưu UI;
              PC có thể không đổi codec được như app điện thoại.
            </p>
            <div class="setting-row">
              <div>
                <span class="setting-title">Bảo vệ thính giác</span>
                <span class="setting-desc">Hearing protection</span>
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
