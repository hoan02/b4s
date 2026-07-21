import { Component, createSignal, Show } from "solid-js";

export type AncMode = "off" | "anc" | "transparency";

interface Props {
  mode?: AncMode;
  strength?: number;
  onModeChange?: (mode: AncMode) => void;
  onStrengthChange?: (value: number) => void;
}

const modes: { id: AncMode; label: string; icon: string; class?: string }[] = [
  { id: "off", label: "Off", icon: "⊘", class: "off" },
  { id: "anc", label: "ANC", icon: "🎧", class: "" },
  { id: "transparency", label: "Ambient", icon: "🌿", class: "transparency" },
];

const AncControl: Component<Props> = (props) => {
  const [mode, setMode] = createSignal<AncMode>(props.mode ?? "anc");
  const [strength, setStrength] = createSignal(props.strength ?? 70);

  const handleMode = (m: AncMode) => {
    setMode(m);
    props.onModeChange?.(m);
  };

  const handleStrength = (e: Event) => {
    const val = Number((e.target as HTMLInputElement).value);
    setStrength(val);
    props.onStrengthChange?.(val);
  };

  const statusText = () => {
    switch (mode()) {
      case "anc":
        return "Noise Cancelling";
      case "transparency":
        return "Transparency";
      default:
        return "Off";
    }
  };

  return (
    <div class="anc-control">
      <div class="anc-header">
        <h3>Noise Control</h3>
        <span class="anc-status">{statusText()}</span>
      </div>

      <div class="anc-modes">
        {modes.map((m) => (
          <button
            class={`anc-mode-btn ${m.class} ${mode() === m.id ? "active" : ""}`}
            onClick={() => handleMode(m.id)}
            type="button"
          >
            <span class="anc-icon">{m.icon}</span>
            <span class="anc-label">{m.label}</span>
          </button>
        ))}
      </div>

      <Show when={mode() === "anc"}>
        <div class="anc-strength">
          <div class="strength-header">
            <span>ANC Strength</span>
            <span class="strength-value">{strength()}%</span>
          </div>
          <input
            type="range"
            class="strength-slider"
            min="0"
            max="100"
            step="5"
            value={strength()}
            onInput={handleStrength}
          />
          <div class="strength-labels">
            <span>Light</span>
            <span>Max</span>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default AncControl;
