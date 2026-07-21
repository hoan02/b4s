import { Component, createSignal, For } from "solid-js";

export type EqPreset = "classic" | "bass" | "voice" | "clear" | "custom" | "game";

interface EqItem {
  id: EqPreset;
  name: string;
  icon: string;
}

const presets: EqItem[] = [
  { id: "classic", name: "Classic", icon: "🎵" },
  { id: "bass", name: "Bass Boost", icon: "🔊" },
  { id: "voice", name: "Voice", icon: "🎤" },
  { id: "clear", name: "Clear", icon: "✨" },
  { id: "game", name: "Game", icon: "🎮" },
  { id: "custom", name: "Custom", icon: "⚙️" },
];

interface Props {
  active?: EqPreset;
  onChange?: (preset: EqPreset) => void;
}

const EqCards: Component<Props> = (props) => {
  const [active, setActive] = createSignal<EqPreset>(props.active ?? "classic");

  const handleSelect = (id: EqPreset) => {
    setActive(id);
    props.onChange?.(id);
  };

  const currentName = () =>
    presets.find((p) => p.id === active())?.name ?? "Classic";

  return (
    <div class="eq-section">
      <div class="eq-header">
        <h3>EQ Mode</h3>
        <span class="eq-current">{currentName()}</span>
      </div>

      <div class="eq-grid">
        <For each={presets}>
          {(preset) => (
            <button
              class={`eq-card ${preset.id} ${active() === preset.id ? "active" : ""}`}
              onClick={() => handleSelect(preset.id)}
              type="button"
            >
              <div class="eq-blob blob-1" />
              <div class="eq-blob blob-2" />
              <div class="eq-content">
                <span class="eq-icon">{preset.icon}</span>
                <span class="eq-name">{preset.name}</span>
              </div>
            </button>
          )}
        </For>
      </div>
    </div>
  );
};

export default EqCards;
