/**
 * “Âm thanh khác” — NOT EQ (EQ is its own screen).
 * Bass boost, LDAC, hearing protection, extras.
 */
import { Component, For } from "solid-js";
import { IconBack } from "./Icons";

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
      <button type="button" class="screen-back" aria-label="Quay lại" onClick={() => props.onBack()}>
        <IconBack size={20} />
      </button>
      <span class="screen-title">Âm thanh khác</span>
      <div class="screen-nav-spacer" />
    </div>

    <section class="more-group">
      <p class="more-label">Âm thanh điện tử</p>
      <div class="more-card">
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
      </div>
    </section>

    <section class="more-group">
      <p class="more-label">Codec & bảo vệ</p>
      <div class="more-card">
        <div class="setting-row">
          <div>
            <span class="setting-title">LDAC</span>
            <span class="setting-desc">Hi-res (UI / OS)</span>
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
      <p class="eq-footnote">
        LDAC / hearing thường do stack Bluetooth OS xử lý — toggle lưu trên B4S;
        PC có thể không đổi codec như app điện thoại.
      </p>
    </section>
  </div>
);

export default MorePanel;
