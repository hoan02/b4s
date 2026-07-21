import { Component, For, Show } from "solid-js";

export interface BatteryData {
  left: number;
  right: number;
  case: number;
  leftCharging?: boolean;
  rightCharging?: boolean;
  caseCharging?: boolean;
}

interface Props {
  data: BatteryData;
  compact?: boolean;
}

function getLevelClass(percent: number, charging?: boolean): string {
  if (charging) return "charging";
  if (percent > 50) return "high";
  if (percent > 20) return "medium";
  return "low";
}

const BatteryItem: Component<{
  label: string;
  percent: number;
  charging?: boolean;
}> = (props) => {
  const levelClass = () => getLevelClass(props.percent, props.charging);

  return (
    <div class="battery-item">
      <span class="battery-label">{props.label}</span>
      <div class="battery-visual">
        <div
          class={`battery-level ${levelClass()}`}
          style={{ width: `${Math.min(100, Math.max(0, props.percent))}%` }}
        />
        <Show when={props.charging}>
          <span class="battery-charging-icon">⚡</span>
        </Show>
      </div>
      <span class="battery-value">{props.percent}%</span>
    </div>
  );
};

const Battery: Component<Props> = (props) => {
  const items = () => [
    { label: "L", percent: props.data.left, charging: props.data.leftCharging },
    { label: "C", percent: props.data.case, charging: props.data.caseCharging },
    { label: "R", percent: props.data.right, charging: props.data.rightCharging },
  ];

  return (
    <Show
      when={!props.compact}
      fallback={
        <div class="battery-compact">
          <For each={items()}>
            {(item, i) => (
              <>
                <Show when={i() > 0}>
                  <div class="separator" />
                </Show>
                <div class="battery-compact-item">
                  <span class={`dot ${getLevelClass(item.percent, item.charging)}`} />
                  <span>{item.label}</span>
                  <strong>{item.percent}%</strong>
                </div>
              </>
            )}
          </For>
        </div>
      }
    >
      <div class="battery-group">
        <For each={items()}>
          {(item) => (
            <BatteryItem
              label={item.label}
              percent={item.percent}
              charging={item.charging}
            />
          )}
        </For>
      </div>
    </Show>
  );
};

export default Battery;
