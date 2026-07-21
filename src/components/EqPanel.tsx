/**
 * Full EQ screen — presets (like official list) + custom multi-band
 * Presets → BA43 + index. Custom curve stored in UI (apply best-effort).
 */
import { Component, For, Show, createSignal, createEffect } from "solid-js";
import type { EqPresetId } from "../lib/device";
import {
  EQ_BANDS,
  EQ_PRESETS,
  clampBand,
  curveForPreset,
  defaultCustomBands,
} from "../lib/eq";
import { IconBack } from "./Icons";

interface Props {
  eqActive: EqPresetId;
  customBands: number[];
  customActive: boolean;
  onBack: () => void;
  onEq: (id: EqPresetId) => void;
  onCustomBands: (bands: number[]) => void;
  onApplyCustom: () => void;
  onResetCustom: () => void;
}

const EqPanel: Component<Props> = (props) => {
  const [tab, setTab] = createSignal<"preset" | "custom">(
    props.customActive ? "custom" : "preset"
  );
  const [localBands, setLocalBands] = createSignal(
    props.customBands.length === 5
      ? props.customBands.slice()
      : defaultCustomBands()
  );

  createEffect(() => {
    if (props.customBands.length === 5) {
      setLocalBands(props.customBands.slice());
    }
  });

  const previewCurve = () =>
    tab() === "custom"
      ? localBands()
      : curveForPreset(props.eqActive);

  const setBand = (i: number, v: number) => {
    const next = localBands().slice();
    next[i] = clampBand(v);
    setLocalBands(next);
    props.onCustomBands(next);
  };

  return (
    <div class="eq-panel">
      <div class="screen-nav">
        <button type="button" class="screen-back" onClick={() => props.onBack()}>
          <IconBack size={20} />
          Xong
        </button>
        <span class="screen-title">EQ</span>
        <div class="screen-nav-spacer" />
      </div>

      {/* Live curve preview */}
      <div class="eq-preview-card">
        <div class="eq-preview-bars" aria-hidden="true">
          <For each={previewCurve()}>
            {(g) => (
              <div class="eq-preview-col">
                <div class="eq-preview-track">
                  <div
                    class="eq-preview-fill"
                    style={{
                      height: `${((g + 6) / 12) * 100}%`,
                    }}
                  />
                </div>
              </div>
            )}
          </For>
        </div>
        <div class="eq-preview-labels">
          <For each={[...EQ_BANDS]}>
            {(b) => <span>{b.label}</span>}
          </For>
        </div>
        <p class="eq-preview-hint">
          {tab() === "custom"
            ? "Tùy chỉnh · 5 băng tần"
            : EQ_PRESETS.find((p) => p.id === props.eqActive)?.label ??
              "Preset"}
        </p>
      </div>

      {/* Tabs: preset | custom */}
      <div class="eq-tabs">
        <button
          type="button"
          class={tab() === "preset" ? "active" : ""}
          onClick={() => setTab("preset")}
        >
          Preset
        </button>
        <button
          type="button"
          class={tab() === "custom" ? "active" : ""}
          onClick={() => setTab("custom")}
        >
          Tùy chỉnh
        </button>
      </div>

      <Show when={tab() === "preset"}>
        <p class="more-label">Chọn preset</p>
        <div class="eq-preset-grid">
          <For each={EQ_PRESETS}>
            {(p) => (
              <button
                type="button"
                class={`eq-preset-card ${
                  !props.customActive && props.eqActive === p.id ? "active" : ""
                }`}
                onClick={() => {
                  setTab("preset");
                  props.onEq(p.id);
                }}
              >
                <div class="eq-mini-bars" aria-hidden="true">
                  <For each={p.curve}>
                    {(g) => (
                      <span
                        style={{
                          height: `${20 + ((g + 6) / 12) * 28}px`,
                        }}
                      />
                    )}
                  </For>
                </div>
                <span class="eq-preset-name">{p.label}</span>
                <span class="eq-preset-sub">{p.sub}</span>
              </button>
            )}
          </For>
        </div>
        <p class="eq-footnote">
          Gửi thiết bị qua BA43 + chỉ số preset (0–10). Tên theo catalog nghe
          phổ biến.
        </p>
      </Show>

      <Show when={tab() === "custom"}>
        <p class="more-label">Tùy chỉnh EQ</p>
        <div class="eq-custom-card">
          <div class="eq-sliders">
            <For each={[...EQ_BANDS]}>
              {(b, i) => (
                <div class="eq-slider-col">
                  <span class="eq-gain">
                    {localBands()[i()] > 0 ? "+" : ""}
                    {localBands()[i()]}
                  </span>
                  <input
                    type="range"
                    min={-6}
                    max={6}
                    step={1}
                    value={localBands()[i()]}
                    class="eq-vslider"
                    aria-label={`${b.label} ${b.unit}`}
                    onInput={(e) =>
                      setBand(
                        i(),
                        Number((e.currentTarget as HTMLInputElement).value)
                      )
                    }
                  />
                  <span class="eq-freq">{b.label}</span>
                  <span class="eq-unit">{b.unit}</span>
                </div>
              )}
            </For>
          </div>
          <div class="eq-custom-actions">
            <button
              type="button"
              class="eq-btn ghost"
              onClick={() => {
                const z = defaultCustomBands();
                setLocalBands(z);
                props.onCustomBands(z);
                props.onResetCustom();
              }}
            >
              Đặt lại
            </button>
            <button
              type="button"
              class="eq-btn primary"
              onClick={() => props.onApplyCustom()}
            >
              Áp dụng
            </button>
          </div>
        </div>
        <p class="eq-footnote">
          Custom trên app official dùng màn Self-Define + BA4300 reset. B4S lưu
          đường cong trên máy; gửi BLE custom đầy đủ còn best-effort (một số
          firmware chỉ nhận preset).
        </p>
      </Show>
    </div>
  );
};

export default EqPanel;
