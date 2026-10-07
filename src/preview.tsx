/**
 * Development-only gallery (`bun run dev`, open /preview.html). It renders the
 * control screens with fixture props so UI work needs no Tauri backend. It is
 * not an entry of the production build.
 */
import { render } from "solid-js/web";
import { createSignal, For, Show } from "solid-js";
import "./styles/main.scss";
import { initializeStorage } from "./lib/storage";

const LEVELS = ["live", "waiting", "demo", "dead", "offline"] as const;

async function start() {
  await initializeStorage();
  const { applyTheme } = await import("./lib/theme");
  const { i18nReady, setLocale, LOCALES } = await import("./lib/i18n");
  await i18nReady;
  const { default: HomePanel } = await import("./components/HomePanel");
  const { default: GesturePanel } = await import("./components/GesturePanel");
  const { default: MorePanel } = await import("./components/MorePanel");
  const { emptyLink } = await import("./lib/ble");

  const [theme, setTheme] = createSignal<"dark" | "light">("dark");
  const [level, setLevel] = createSignal<(typeof LEVELS)[number]>("live");
  const [ldac, setLdac] = createSignal<boolean | null>(true);
  applyTheme("dark");

  const link = () => ({
    ...emptyLink(),
    level: level(),
    connected: level() !== "offline",
    mock: level() === "demo",
    peripheralConnected: level() !== "dead",
    hasWriteUuid: true,
    hasNotifyUuid: true,
    handshakeOk: true,
    notifyCount: 128,
    txCount: 14,
    lastRxHex: "AA0264640064",
    lastTxHex: "BA02",
  });
  const noop = () => {};
  const view = new URLSearchParams(location.search).get("view");
  const show = (name: string) => view === null || view === name;

  const App = () => (
    <div style={{ display: "flex", gap: "24px", padding: "16px", "align-items": "flex-start", "flex-wrap": "wrap" }}>
      <div style={{ width: "100%", display: "flex", gap: "8px", "flex-wrap": "wrap" }}>
        <button onClick={() => { const next = theme() === "dark" ? "light" : "dark"; setTheme(next); applyTheme(next); }}>
          theme: {theme()}
        </button>
        <For each={LEVELS}>{(item) => <button onClick={() => setLevel(item)}>{item}</button>}</For>
        <For each={[...LOCALES]}>{(item) => <button onClick={() => void setLocale(item)}>{item}</button>}</For>
      </div>
      <Show when={show("home")}>
      <div class="preview-frame" style={{ width: "400px", background: "var(--bg)", height: "860px", overflow: "auto" }}>
        <section class="section section-scroll">
          <HomePanel
            name="Baseus Bass BP1 Pro"
            modelId="bass-bp1-pro"
            experimentalFeatures={["gesture", "inEar", "multipoint", "restoreDefaults", "adaptiveLr", "windNoise"]}
            battery={{ left: 82, right: 14, case: 100, rightCharging: true }}
            link={link()}
            ancMode="anc"
            transparencyMode="full"
            adaptiveNoise
            noiseEnvironment={102}
            noiseLevel={3}
            listeningSupported
            noiseMaxLevel={5}
            noiseSupported
            adaptiveSupported
            transparencyVoiceSupported
            gameSupported
            eqSupported
            findSupported
            spatialSupported
            moreSupported
            gestureSupported
            inEarSupported
            inEarOn
            multipointSupported
            multipointOn={false}
            restoreSupported
            adaptiveLrSupported
            adaptiveLrOn={null}
            windNoiseSupported
            windNoiseOn
            gameMode={false}
            findActive={false}
            spatialOn
            spatialMode="music"
            eqLabel="Baseus Classic"
            onAncMode={noop}
            onTransparencyMode={noop}
            onAdaptiveNoise={noop}
            onNoiseEnvironment={noop}
            onNoiseLevel={noop}
            onGameMode={noop}
            onFindBuds={noop}
            onOpenMore={noop}
            onOpenSettings={noop}
            onDisconnect={noop}
            onOpenEq={noop}
            onOpenGestures={noop}
            onInEar={noop}
            onMultipoint={noop}
            onAdaptiveLr={noop}
            onWindNoise={noop}
            onRestore={noop}
            onSpatialOn={noop}
            onSpatialMode={noop}
          />
        </section>
      </div>
      </Show>
      <Show when={show("gesture")}>
      <div class="preview-frame" style={{ width: "400px", background: "var(--bg)" }}>
        <section class="section section-scroll">
          <GesturePanel
            dualButton
            layouts={[
              { layout: 0, functions: [11, 12, 1, 6, 2, 3, 4, 0] },
              { layout: 3, functions: [1, 0] },
            ]}
            gestureState={[{ layout: 0, left: 1, right: 2 }, { layout: 3, left: 1, right: 1 }]}
            inEarSupported
            inEarOn
            pending={false}
            error={null}
            experimental
            onBack={noop}
            onInEar={noop}
            onGesture={noop}
          />
        </section>
      </div>
      </Show>
      <Show when={show("more")}>
      <div class="preview-frame" style={{ width: "400px", background: "var(--bg)" }}>
        <section class="section section-scroll">
          <MorePanel
            bassSupported
            bassMaxLevel={3}
            ldacSupported
            hearingSupported
            bassBoost={1}
            ldac={ldac()}
            hearingProtect
            hearingThreshold={85}
            hearingThresholds={[75, 80, 85, 90, 95, 100]}
            onHearingThreshold={noop}
            onBack={noop}
            onBassBoost={noop}
            onLdac={(value: boolean) => setLdac(value)}
            onHearingProtect={noop}
          />
        </section>
      </div>
      </Show>
    </div>
  );
  render(() => <App />, document.getElementById("root")!);
}
void start();
