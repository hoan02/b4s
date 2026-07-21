/**
 * App settings — version, update, theme, about
 */
import { Component, Show, createSignal, onMount } from "solid-js";
import {
  getAppInfo,
  checkForUpdates,
  installUpdate,
  openExternal,
  type AppInfo,
  type UpdateCheckResult,
} from "../lib/app";
import type { ThemeMode } from "../lib/theme";
import type { ToastKind } from "../lib/toast";

interface Props {
  theme: ThemeMode;
  onToggleTheme: () => void;
  onNotify?: (msg: string, kind?: ToastKind, title?: string) => void;
}

const Settings: Component<Props> = (props) => {
  const [info, setInfo] = createSignal<AppInfo | null>(null);
  const [checking, setChecking] = createSignal(false);
  const [installing, setInstalling] = createSignal(false);
  const [update, setUpdate] = createSignal<UpdateCheckResult | null>(null);
  const [status, setStatus] = createSignal<{
    kind: "ok" | "warn" | "err" | "info";
    text: string;
  } | null>(null);

  onMount(async () => {
    try {
      setInfo(await getAppInfo());
    } catch (e) {
      setStatus({ kind: "err", text: String(e) });
    }
  });

  const handleCheck = async () => {
    setChecking(true);
    setStatus(null);
    setUpdate(null);
    try {
      const r = await checkForUpdates();
      setUpdate(r);
      if (r.available) {
        setStatus({
          kind: "warn",
          text: `Có bản ${r.version ?? "mới"}. Cài đặt để cập nhật.`,
        });
        props.onNotify?.(`Bản ${r.version}`, "warn", "Cập nhật");
      } else if (r.error) {
        setStatus({ kind: "err", text: r.error });
      } else {
        setStatus({ kind: "ok", text: "Bạn đang dùng bản mới nhất." });
        props.onNotify?.("Đã là bản mới nhất", "success");
      }
    } catch (e) {
      setStatus({ kind: "err", text: String(e) });
    } finally {
      setChecking(false);
    }
  };

  const handleInstall = async () => {
    setInstalling(true);
    setStatus({ kind: "info", text: "Đang tải và cài cập nhật…" });
    try {
      await installUpdate();
      setStatus({
        kind: "ok",
        text: "Đã cài xong. Ứng dụng sẽ khởi động lại.",
      });
    } catch (e) {
      setStatus({ kind: "err", text: String(e) });
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div class="settings">
      <div class="settings-hero">
        <div class="settings-app-mark" aria-hidden="true">
          B4
        </div>
        <div class="settings-app-name">B4S</div>
        <div class="settings-app-ver">
          Phiên bản {info()?.version ?? "…"}
          <Show when={info()?.debug}> · Dev</Show>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">Giao diện</div>
        <div class="settings-list">
          <button
            type="button"
            class="settings-row action"
            onClick={() => props.onToggleTheme()}
          >
            <span class="settings-row-label">Giao diện</span>
            <span class="settings-row-value">
              {props.theme === "dark" ? "Tối" : "Sáng"}
            </span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
        <p class="settings-footnote">
          Sáng: nền sáng + nhấn vàng. Tối: nền tối + nhấn vàng.
        </p>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">Cập nhật</div>
        <div class="settings-list">
          <div class="settings-row">
            <span class="settings-row-label">Phiên bản</span>
            <span class="settings-row-value">{info()?.version ?? "—"}</span>
          </div>
          <button
            type="button"
            class="settings-row action"
            disabled={checking() || installing()}
            onClick={handleCheck}
          >
            <span class="settings-row-label">
              {checking() ? "Đang kiểm tra…" : "Kiểm tra cập nhật"}
            </span>
          </button>
          <Show when={update()?.available}>
            <button
              type="button"
              class="settings-row action"
              disabled={installing()}
              onClick={handleInstall}
            >
              <span class="settings-row-label">
                {installing()
                  ? "Đang cài…"
                  : `Cài bản ${update()?.version ?? "mới"}`}
              </span>
            </button>
          </Show>
          <button
            type="button"
            class="settings-row action"
            onClick={() =>
              openExternal("https://github.com/hoan02/b4s/releases")
            }
          >
            <span class="settings-row-label">GitHub Releases</span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
        <Show when={status()}>
          <div
            class={`settings-status ${
              status()!.kind === "info" ? "" : status()!.kind
            }`}
          >
            {status()!.text}
          </div>
        </Show>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">Thông tin</div>
        <div class="settings-list">
          <div class="settings-row">
            <span class="settings-row-label">Hệ điều hành</span>
            <span class="settings-row-value">{info()?.os ?? "—"}</span>
          </div>
          <div class="settings-row">
            <span class="settings-row-label">Identifier</span>
            <span class="settings-row-value">
              {info()?.identifier ?? "com.hoan02.b4s"}
            </span>
          </div>
          <button
            type="button"
            class="settings-row action"
            onClick={() => openExternal("https://github.com/hoan02/b4s")}
          >
            <span class="settings-row-label">Mã nguồn</span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
        <p class="settings-footnote">
          B4S là phần mềm không chính thức, độc lập. Không thuộc về hay được
          chứng nhận bởi nhà sản xuất tai nghe. Chỉ dùng cá nhân / tương thích
          kỹ thuật.
        </p>
      </div>
    </div>
  );
};

export default Settings;
