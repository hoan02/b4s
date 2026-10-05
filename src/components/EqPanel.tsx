/**
 * Full EQ screen — presets (like official list) + custom multi-band
 * Presets → BA43 + index. Custom curve stored in UI (apply best-effort).
 */
import { Component, For, Show, createSignal, createEffect } from "solid-js";
import type { EqPresetId } from "../lib/device";
import {
  EQ_BANDS,
  EQ_PRESETS,
  type CustomEqPreset,
  clampBand,
  curveForPreset,
  defaultCustomBands,
  loadCustomEqPresets,
  saveCustomEqPresets,
} from "../lib/eq";
import { IconBack } from "./Icons";
import { t } from "../lib/i18n";

interface Props {
  eqActive: EqPresetId;
  customBands: number[];
  customActive: boolean;
  storageKey: string;
  onBack: () => void;
  onEq: (id: EqPresetId) => void;
  onCustomBands: (bands: number[]) => boolean;
  onApplyCustom: (bands: number[], label: string) => void;
  onResetCustom: () => void;
}

const EqPanel: Component<Props> = (props) => {
  const [tab, setTab] = createSignal<"preset" | "custom">(
    props.customActive ? "custom" : "preset"
  );
  const [localBands, setLocalBands] = createSignal(
    props.customBands.length === EQ_BANDS.length
      ? props.customBands.slice()
      : defaultCustomBands()
  );
  const [customPresets, setCustomPresets] = createSignal<CustomEqPreset[]>(
    loadCustomEqPresets(props.storageKey)
  );
  const [customName, setCustomName] = createSignal("");
  const [customError, setCustomError] = createSignal("");

  createEffect(() => {
    if (props.customBands.length === EQ_BANDS.length) {
      setLocalBands(props.customBands.slice());
    }
  });

  const selectCustom = (preset: CustomEqPreset) => {
    if (props.onCustomBands(preset.bands)) {
      setLocalBands(preset.bands.slice());
      setCustomName(preset.label);
    }
  };

  const saveCustom = () => {
    const label = customName().trim();
    if (!label) {
      setCustomError(t("eq.saveName"));
      return;
    }
    if (customPresets().some((p) => p.label.toLowerCase() === label.toLowerCase())) {
      setCustomError(t("eq.duplicateName"));
      return;
    }
    if (customPresets().length >= 2) {
      setCustomError(t("eq.maxCustom"));
      return;
    }
    const next = [...customPresets(), { id: `custom-${Date.now()}`, label, bands: localBands().slice() }];
    setCustomPresets(next);
    saveCustomEqPresets(props.storageKey, next);
    setCustomError("");
  };

  const deleteCustom = (id: string) => {
    const next = customPresets().filter((p) => p.id !== id);
    setCustomPresets(next);
    saveCustomEqPresets(props.storageKey, next);
  };

  const previewCurve = () =>
    tab() === "custom"
      ? localBands()
      : curveForPreset(props.eqActive);

  const setBand = (i: number, v: number) => {
    const next = localBands().slice();
    next[i] = clampBand(v);
    if (props.onCustomBands(next)) {
      setLocalBands(next);
    }
  };

  return (
    <div class="eq-panel">
      <div class="screen-nav">
        <button type="button" class="screen-back" aria-label={t("nav.back")} onClick={() => props.onBack()}>
          <IconBack size={20} />
        </button>
        <span class="screen-title">{t("eq.title")}</span>
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
            ? t("eq.customBands")
            : (t(`eqPreset.${props.eqActive}`) || EQ_PRESETS.find((p) => p.id === props.eqActive)?.label) ??
              t("eq.preset")}
        </p>
      </div>

      {/* Tabs: preset | custom */}
      <div class="eq-tabs">
        <button
          type="button"
          class={tab() === "preset" ? "active" : ""}
          onClick={() => setTab("preset")}
        >
          {t("eq.preset")}
        </button>
        <button
          type="button"
          class={tab() === "custom" ? "active" : ""}
          onClick={() => setTab("custom")}
        >
          {t("eq.customize")}
        </button>
      </div>

      <Show when={tab() === "preset"}>
        <p class="more-label">{t("eq.choosePreset")}</p>
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
                          <span class="eq-preset-name">{t(`eqPreset.${p.id}`) || p.label}</span>
                          <span class="eq-preset-sub">{t(`eqPresetSub.${p.id}`) || p.sub}</span>
              </button>
            )}
          </For>
        </div>
        <p class="eq-footnote">{t("eq.presetFootnote")}</p>
      </Show>

      <Show when={tab() === "custom"}>
        <p class="more-label">{t("eq.customize")}</p>
        <Show when={customPresets().length > 0}>
          <div class="eq-saved-list">
            <For each={customPresets()}>
              {(preset) => (
                <div class="eq-saved-item">
                  <button type="button" onClick={() => selectCustom(preset)}>{preset.label}</button>
                  <button type="button" aria-label={t("eq.delete", { name: preset.label })} onClick={() => deleteCustom(preset.id)}>×</button>
                </div>
              )}
            </For>
          </div>
        </Show>
        <div class="eq-name-row">
          <input
            value={customName()}
            placeholder={t("eq.namePlaceholder")}
            onInput={(e) => setCustomName(e.currentTarget.value)}
            aria-label={t("eq.nameLabel")}
          />
          <button type="button" class="eq-btn ghost" onClick={saveCustom}>{t("eq.save")}</button>
        </div>
        <Show when={customError()}><p class="eq-inline-error">{customError()}</p></Show>
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
                    min={-12}
                    max={12}
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
                props.onResetCustom();
              }}
            >
              {t("eq.reset")}
            </button>
            <button
              type="button"
              class="eq-btn primary"
              onClick={() => props.onApplyCustom(localBands(), customName().trim() || t("eq.customize"))}
            >
              {t("eq.apply")}
            </button>
          </div>
        </div>
        <p class="eq-footnote">{t("eq.customFootnote")}</p>
      </Show>
    </div>
  );
};

export default EqPanel;
