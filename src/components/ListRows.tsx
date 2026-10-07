/** Shared rows for grouped control lists: one structure for icon, title, hint, state and badge. */
import { JSX, Show } from "solid-js";
import { requestToggle } from "../lib/confirmedToggle";
import { t } from "../lib/i18n";
import OperationStatus from "./OperationStatus";

export const ExperimentalBadge = () => (
  <span class="exp-badge">{t("common.experimental")}</span>
);

interface RowText {
  icon: JSX.Element;
  title: string;
  hint?: string;
  experimental?: boolean;
  pending?: boolean;
  error?: string | null;
}

const RowText = (props: RowText) => (
  <>
    <span class="list-ico">{props.icon}</span>
    <div class="list-text">
      <span class="list-title">
        <span class="list-title-text">{props.title}</span>
        <Show when={props.experimental}>
          <ExperimentalBadge />
        </Show>
      </span>
      <Show when={props.hint}>
        <span class="list-sub">{props.hint}</span>
      </Show>
      <OperationStatus pending={props.pending} error={props.error} />
    </div>
  </>
);

interface ToggleRowProps extends RowText {
  /** null = no confirmed device state yet (shown as mixed). */
  checked: boolean | null;
  onChange: (enabled: boolean) => void;
}

export const ToggleRow = (props: ToggleRowProps) => (
  <div class="list-row">
    <RowText {...props} />
    <label class="toggle sm">
      <input
        type="checkbox"
        disabled={props.pending}
        aria-busy={props.pending}
        aria-checked={props.checked === null ? "mixed" : props.checked}
        checked={props.checked === true}
        aria-label={props.title}
        onChange={(e) => requestToggle(e.currentTarget, props.checked === true, props.onChange)}
      />
      <span class="slider" />
    </label>
  </div>
);

interface NavRowProps extends RowText {
  danger?: boolean;
  active?: boolean;
  /** Hide the trailing chevron for actions that do not open a screen. */
  chevron?: boolean;
  disabled?: boolean;
  onClick: () => void;
}

export const NavRow = (props: NavRowProps) => (
  <button
    type="button"
    class="list-row action"
    classList={{ danger: props.danger, active: props.active }}
    disabled={props.disabled}
    aria-pressed={props.active === undefined ? undefined : props.active}
    onClick={() => props.onClick()}
  >
    <RowText {...props} />
    <Show when={props.chevron !== false}>
      <span class="list-chev">›</span>
    </Show>
  </button>
);
