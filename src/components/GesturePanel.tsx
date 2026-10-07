import { Component, For, Show } from "solid-js";
import { t } from "../lib/i18n";
import OperationStatus from "./OperationStatus";
import { IconBack, IconEar } from "./Icons";
import { ToggleRow } from "./ListRows";
import { gestureFunctionLabelKey, gestureLayoutLabelKey } from "../features/gestures/actions";

export interface GestureLayout {
  layout: number;
  functions: number[];
}

export interface GestureValue {
  layout: number;
  left: number;
  right: number;
}

interface Props {
  dualButton: boolean;
  layouts: GestureLayout[];
  gestureState: GestureValue[];
  inEarSupported: boolean;
  inEarOn: boolean | null;
  pending: boolean;
  error: string | null;
  experimental: boolean;
  onBack: () => void;
  onInEar: (enabled: boolean) => void;
  onGesture: (layout: number, left: number | null, right: number | null) => void;
}

const GesturePanel: Component<Props> = (props) => {
  const valueFor = (layout: number, side: "left" | "right"): number | null => {
    const current = props.gestureState.find((value) => value.layout === layout);
    if (!current) return null;
    return side === "left" ? current.left : current.right;
  };

  const selectValue = (layout: number, side: "left" | "right"): string => {
    const value = valueFor(layout, side);
    return value === null || value === undefined ? "" : String(value);
  };

  return (
    <div class="gesture-panel">
      <div class="screen-nav">
        <button type="button" class="screen-back" aria-label={t("nav.back")} onClick={() => props.onBack()}>
          <IconBack size={20} />
        </button>
        <span class="screen-title">{t("gesture.title")}</span>
        <div class="screen-nav-spacer" />
      </div>

      <Show when={props.experimental}>
        <p class="gesture-experimental" role="note">{t("gesture.experimental")}</p>
      </Show>

      <Show when={props.inEarSupported}>
        <div class="home-list-card gesture-inear">
          <ToggleRow
            icon={<IconEar size={22} />}
            title={t("gesture.inEar")}
            hint={t("gesture.inEarHint")}
            checked={props.inEarOn}
            pending={props.pending}
            onChange={props.onInEar}
          />
        </div>
      </Show>

      <OperationStatus pending={props.pending} error={props.error} />

      <For each={props.layouts}>
        {(layout) => (
          <div class="gesture-layout">
            <p class="gesture-layout-title">{t(gestureLayoutLabelKey(layout.layout))}</p>
            <div class="gesture-selects">
              <label class="gesture-select">
                <span>{t("gesture.left")}</span>
                <select
                  disabled={props.pending}
                  value={selectValue(layout.layout, "left")}
                  onChange={(e) =>
                    props.onGesture(
                      layout.layout,
                      Number(e.currentTarget.value),
                      props.dualButton ? valueFor(layout.layout, "right") : Number(e.currentTarget.value)
                    )
                  }
                >
                  <option value="">—</option>
                  <For each={layout.functions}>
                    {(functionId) => (
                      <option value={String(functionId)}>{t(gestureFunctionLabelKey(functionId))}</option>
                    )}
                  </For>
                </select>
              </label>
              <Show when={props.dualButton}>
                <label class="gesture-select">
                  <span>{t("gesture.right")}</span>
                  <select
                    disabled={props.pending}
                    value={selectValue(layout.layout, "right")}
                    onChange={(e) =>
                      props.onGesture(
                        layout.layout,
                        valueFor(layout.layout, "left"),
                        Number(e.currentTarget.value)
                      )
                    }
                  >
                    <option value="">—</option>
                    <For each={layout.functions}>
                      {(functionId) => (
                        <option value={String(functionId)}>{t(gestureFunctionLabelKey(functionId))}</option>
                      )}
                    </For>
                  </select>
                </label>
              </Show>
            </div>
          </div>
        )}
      </For>
    </div>
  );
};

export default GesturePanel;
