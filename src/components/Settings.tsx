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
import {
  IconDownload,
  IconGithub,
  IconId,
  IconOs,
  IconTheme,
  IconUpdate,
  IconVersion,
} from "./Icons";

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
        <img class="settings-app-logo" src="/b4s-logo.png" alt="B4S" />
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
            <IconTheme size={19} />
            <span class="settings-row-label">Giao diện</span>
            <span class="settings-row-value">
              {props.theme === "dark" ? "Tối" : "Sáng"}
            </span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
      </div>

      <div class="settings-group">
        <div class="settings-group-label">Cập nhật</div>
        <div class="settings-list">
          <div class="settings-row">
            <IconVersion size={19} />
            <span class="settings-row-label">Phiên bản</span>
            <span class="settings-row-value">{info()?.version ?? "—"}</span>
          </div>
          <button
            type="button"
            class="settings-row action"
            disabled={checking() || installing()}
            onClick={handleCheck}
          >
            <IconUpdate size={19} />
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
              <IconDownload size={19} />
              <span class="settings-row-label">
                {installing()
                  ? "Đang cài…"
                  : `Cài bản ${update()?.version ?? "mới"}`}
              </span>
            </button>
          </Show>
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
            <IconOs size={19} />
            <span class="settings-row-label">Hệ điều hành</span>
            <span class="settings-row-value">{info()?.os ?? "—"}</span>
          </div>
          <div class="settings-row">
            <IconId size={19} />
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
            <span class="settings-source">
              <IconGithub size={19} />
              <span class="settings-row-label">Mã nguồn</span>
            </span>
            <span class="settings-row-value">hoan02/b4s</span>
            <span class="settings-row-chev">›</span>
          </button>
        </div>
        <div class="settings-legal-label">Lưu ý</div>
        <div class="settings-legal">
          <strong>Miễn trừ trách nhiệm và sử dụng an toàn</strong>
          <div class="settings-legal-section">
            <b>Bản chất phần mềm</b>
            <p>
              B4S là mã nguồn mở, phi thương mại và được phát triển độc lập cho
              mục đích nghiên cứu, tương thích kỹ thuật và sử dụng cá nhân. B4S
              không được Baseus hoặc bất kỳ nhà sản xuất tai nghe nào tài trợ,
              chứng nhận, bảo hành hay liên kết chính thức.
            </p>
          </div>
          <div class="settings-legal-section">
            <b>Dữ liệu và kết nối</b>
            <p>
              Các chức năng điều khiển được xử lý cục bộ qua Bluetooth, không
              yêu cầu tài khoản và không chủ động gửi dữ liệu sử dụng, dữ liệu
              thiết bị hoặc thông tin cá nhân về máy chủ. Ứng dụng có thể chạy
              offline; Internet chỉ cần khi người dùng chủ động kiểm tra cập
              nhật hoặc mở liên kết bên ngoài.
            </p>
          </div>
          <div class="settings-legal-section">
            <b>Phạm vi hỗ trợ model</b>
            <p>
              Baseus Bass BP1 Ultra thuộc họ giao thức BP1 là mẫu đang được
              kiểm thử chính. Việc nhận diện được BP1 Pro hoặc model khác không
              đồng nghĩa model đó đã được xác minh, tương thích đầy đủ hoặc
              được nhà sản xuất hỗ trợ.
            </p>
          </div>
          <div class="settings-legal-section">
            <b>Trách nhiệm sử dụng</b>
            <p>
              Người dùng tự chịu trách nhiệm khi kết nối, cập nhật firmware và
              thay đổi âm lượng, EQ, ANC hoặc âm thanh không gian. B4S không
              bảo đảm thiết bị luôn tương thích, hoạt động liên tục hoặc có thể
              khôi phục sau lỗi firmware.
            </p>
          </div>
          <div class="settings-legal-section">
            <b>An toàn thính giác</b>
            <p>
              Tìm tai nghe có thể phát âm thanh lớn. Hãy tháo tai nghe khỏi tai
              trước khi xác nhận, đứng ở mức âm lượng an toàn và dừng ngay nếu
              cảm thấy đau, ù tai hoặc khó chịu.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Settings;
