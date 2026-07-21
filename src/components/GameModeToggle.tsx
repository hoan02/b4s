import { Component, createSignal } from "solid-js";

interface Props {
  enabled?: boolean;
  onChange?: (enabled: boolean) => void;
}

const GameModeToggle: Component<Props> = (props) => {
  const [on, setOn] = createSignal(props.enabled ?? false);

  const toggle = () => {
    const next = !on();
    setOn(next);
    props.onChange?.(next);
  };

  return (
    <div class="game-mode-toggle">
      <div class="game-info">
        <div class="game-icon">🎮</div>
        <div class="game-text">
          <h4>Game Mode</h4>
          <p>Low latency for better sync</p>
        </div>
      </div>

      <label class="toggle">
        <input type="checkbox" checked={on()} onChange={toggle} />
        <span class="slider" />
      </label>
    </div>
  );
};

export default GameModeToggle;
