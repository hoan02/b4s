import { Component } from "solid-js";

interface Props {
  title: string;
  message: string;
  onCancel: () => void;
  onConfirm: () => void;
  confirmLabel?: string;
  cancelLabel?: string;
  showCancel?: boolean;
}

const ConfirmDialog: Component<Props> = (props) => (
  <div class="confirm-backdrop" role="presentation" onClick={(e) => e.currentTarget === e.target && props.onCancel()}>
    <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
      <div class="confirm-dialog-icon">♫</div>
      <h2 id="confirm-title">{props.title}</h2>
      <p>{props.message}</p>
      <div class={`confirm-dialog-actions ${props.showCancel === false ? "single" : ""}`}>
        {props.showCancel !== false && <button type="button" class="confirm-btn secondary" onClick={props.onCancel}>{props.cancelLabel ?? "Hủy"}</button>}
        <button type="button" class="confirm-btn primary" onClick={props.onConfirm}>{props.confirmLabel ?? "Tắt và tiếp tục"}</button>
      </div>
    </section>
  </div>
);

export default ConfirmDialog;
