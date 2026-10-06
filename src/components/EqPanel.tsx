/**
 * Full EQ screen — presets (like official list) + custom multi-band
 * Preset identities and preview data come from the selected model catalog.
 */
import { Component, For, Show, createSignal, createEffect } from "solid-js";
import type { EqPresetId } from "../lib/device";
import {
  type EqPresetMeta,
  type CustomEqPreset,
  loadCustomEqPresets,
  saveCustomEqPresets,
} from "../lib/eq";
import { IconBack } from "./Icons";
import { t } from "../lib/i18n";
import OperationStatus from "./OperationStatus";

interface Props {
  presets: EqPresetMeta[];
  frequencies: number[];
  minGain: number;
  maxGain: number;
  customSupported: boolean;
  eqActive: EqPresetId;
  pending?: boolean;
  error?: string | null;
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
    props.customBands.length === props.frequencies.length
      ? props.customBands.slice()
      : props.frequencies.map(() => 0)
  );
  const [customPresets, setCustomPresets] = createSignal<CustomEqPreset[]>(
    loadCustomEqPresets(props.storageKey, props.frequencies.length, props.minGain, props.maxGain)
  );
  const [customName, setCustomName] = createSignal("");
  const [customError, setCustomError] = createSignal("");

  createEffect(() => {
    setLocalBands(props.customBands.length === props.frequencies.length
      ? props.customBands.slice() : props.frequencies.map(() => 0));
  });
  createEffect(() => {
    setCustomPresets(loadCustomEqPresets(props.storageKey, props.frequencies.length, props.minGain, props.maxGain));
    setCustomName("");
    setCustomError("");
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
    try {
      saveCustomEqPresets(props.storageKey, next);
      setCustomPresets(next);
      setCustomError("");
    } catch {
      setCustomError(t("eq.storageError"));
    }
  };

  const deleteCustom = (id: string) => {
    const next = customPresets().filter((p) => p.id !== id);
    try {
      saveCustomEqPresets(props.storageKey, next);
      setCustomPresets(next);
      setCustomError("");
    } catch {
      setCustomError(t("eq.storageError"));
    }
  };

  const previewCurve = () =>
    tab() === "custom"
      ? localBands()
      : props.presets.find((preset) => preset.id === props.eqActive)?.curve ?? [];

  const setBand = (i: number, v: number) => {
    const next = localBands().slice();
    next[i] = Math.max(props.minGain, Math.min(props.maxGain, Math.round(v)));
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
      <Show when={previewCurve().length > 0}>
      <div class="eq-preview-card">
        <div class="eq-preview-bars" aria-hidden="true">
          <For each={previewCurve()}>
            {(g) => (
              <div class="eq-preview-col">
                <div class="eq-preview-track">
                  <div
                    class="eq-preview-fill"
                    style={{
                      height: `${((g - props.minGain) / (props.maxGain - props.minGain)) * 100}%`,
                    }}
                  />
                </div>
              </div>
            )}
          </For>
        </div>
        <div class="eq-preview-labels">
          <For each={props.frequencies}>
            {(frequency) => <span>{frequency}</span>}
          </For>
        </div>
        <p class="eq-preview-hint">
          {tab() === "custom"
            ? t("eq.customBands", { count: props.frequencies.length })
            : props.presets.find((p) => p.id === props.eqActive)?.label ??
              t("eq.preset")}
        </p>
      </div>

      </Show>

      {/* Tabs: preset | custom */}
      <div class="eq-tabs">
        <button
          type="button"
          class={tab() === "preset" ? "active" : ""}
          aria-pressed={tab() === "preset"}
          onClick={() => setTab("preset")}
        >
          {t("eq.preset")}
        </button>
        <button
          type="button"
          class={tab() === "custom" ? "active" : ""}
          disabled={!props.customSupported}
          aria-pressed={tab() === "custom"}
          onClick={() => setTab("custom")}
        >
          {t("eq.customize")}
        </button>
      </div>

      <Show when={tab() === "preset"}>
        <p class="more-label">{t("eq.choosePreset")}</p>
        <OperationStatus pending={props.pending} error={props.error} />
        <div class="eq-preset-grid" aria-busy={props.pending}>
          <For each={props.presets}>
            {(p) => (
              <button
                type="button"
                disabled={props.pending}
                class={`eq-preset-card ${
                  !props.customActive && props.eqActive === p.id ? "active" : ""
                }`}
                aria-pressed={!props.customActive && props.eqActive === p.id}
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
        <p class="eq-footnote">{t("eq.presetFootnote")}</p>
      </Show>

      <Show when={tab() === "custom" && props.customSupported}>
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
        <Show when={customError()}><p class="eq-inline-error" role="alert">{customError()}</p></Show>
        <div class="eq-custom-card">
          <div class="eq-sliders">
            <For each={props.frequencies}>
              {(frequency, i) => (
                <div class="eq-slider-col">
                  <span class="eq-gain">
                    {localBands()[i()] > 0 ? "+" : ""}
                    {localBands()[i()]}
                  </span>
                  <input
                    type="range"
                    min={props.minGain}
                    max={props.maxGain}
                    step={1}
                    value={localBands()[i()]}
                    class="eq-vslider"
                    aria-label={`${frequency} Hz`}
                    onInput={(e) =>
                      setBand(
                        i(),
                        Number((e.currentTarget as HTMLInputElement).value)
                      )
                    }
                  />
                  <span class="eq-freq">{frequency}</span>
                  <span class="eq-unit">Hz</span>
                </div>
              )}
            </For>
          </div>
          <div class="eq-custom-actions">
            <button
              type="button"
              class="eq-btn ghost"
              disabled={props.pending}
              onClick={() => {
                props.onResetCustom();
              }}
            >
              {t("eq.reset")}
            </button>
            <button
              type="button"
              class="eq-btn primary"
              disabled={props.pending}
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
