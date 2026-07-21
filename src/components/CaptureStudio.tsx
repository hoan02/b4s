import {
  Component,
  For,
  Show,
  createEffect,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import {
  CaptureEntry,
  CaptureSession,
  GuidedStep,
  startCapture,
  stopCapture,
  clearCapture,
  getCapture,
  setStep,
  markStep,
  addNote,
  exportJson,
  exportMarkdown,
  writeHex,
  onCaptureSession,
  onCaptureEntry,
  downloadJson,
  downloadText,
} from "../lib/capture";

interface Props {
  deviceName?: string | null;
  deviceAddress?: string | null;
  connected?: boolean;
  onClose?: () => void;
}

const CaptureStudio: Component<Props> = (props) => {
  const [session, setSession] = createSignal<CaptureSession | null>(null);
  const [filter, setFilter] = createSignal<"all" | "tx" | "rx" | "info">("all");
  const [hexInput, setHexInput] = createSignal("BA 34 01 68");
  const [noteInput, setNoteInput] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [tab, setTab] = createSignal<"log" | "steps" | "gatt" | "write">("log");
  let logEnd: HTMLDivElement | undefined;

  let unsubs: Array<() => void> = [];

  onMount(async () => {
    try {
      setSession(await getCapture());
    } catch {
      /* ok */
    }
    unsubs.push(await onCaptureSession((s) => setSession(s)));
    unsubs.push(
      await onCaptureEntry(() => {
        // session event also fires; just scroll
        requestAnimationFrame(() => {
          logEnd?.scrollIntoView({ behavior: "smooth" });
        });
      })
    );
  });

  onCleanup(() => unsubs.forEach((u) => u()));

  createEffect(() => {
    // auto-scroll when entries change
    const s = session();
    if (s?.entries.length) {
      requestAnimationFrame(() => logEnd?.scrollIntoView({ behavior: "smooth" }));
    }
  });

  const active = () => session()?.active ?? false;
  const entries = () => {
    const all = session()?.entries ?? [];
    const f = filter();
    if (f === "all") return all;
    return all.filter((e) => e.direction === f);
  };
  const steps = () => session()?.steps ?? [];
  const gatt = () => session()?.gattMap ?? [];
  const currentStep = () => session()?.currentStepId;

  const handleStart = async () => {
    setBusy(true);
    setError(null);
    try {
      const s = await startCapture(
        props.deviceName ?? undefined,
        props.deviceAddress ?? undefined
      );
      setSession(s);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleStop = async () => {
    setBusy(true);
    try {
      setSession(await stopCapture());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleClear = async () => {
    try {
      setSession(await clearCapture());
    } catch (e) {
      setError(String(e));
    }
  };

  const handleStep = async (id: string) => {
    try {
      await setStep(id);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleMark = async (id: string, done: boolean) => {
    try {
      await markStep(id, done);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleWrite = async () => {
    setBusy(true);
    setError(null);
    try {
      await writeHex(hexInput());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleNote = async () => {
    const t = noteInput().trim();
    if (!t) return;
    try {
      await addNote(t);
      setNoteInput("");
    } catch (e) {
      setError(String(e));
    }
  };

  const handleExportJson = async () => {
    try {
      const bundle = await exportJson();
      const name = `baseus-capture-${Date.now()}.json`;
      downloadJson(name, bundle);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleExportMd = async () => {
    try {
      const md = await exportMarkdown();
      downloadText(`baseus-capture-${Date.now()}.md`, md, "text/markdown");
    } catch (e) {
      setError(String(e));
    }
  };

  const doneCount = () => steps().filter((s) => s.done).length;

  return (
    <div class="capture-studio">
      {/* Header */}
      <div class="cs-header">
        <div class="cs-title">
          <span class="cs-icon">🔬</span>
          <div>
            <h2>Capture Studio</h2>
            <p class="cs-sub">
              Reverse-engineer Baseus BLE protocol
              <Show when={session()?.deviceName}>
                {" · "}
                <strong>{session()!.deviceName}</strong>
              </Show>
            </p>
          </div>
        </div>
        <div class="cs-header-actions">
          <Show
            when={active()}
            fallback={
              <button
                class="cs-btn primary"
                type="button"
                disabled={busy()}
                onClick={handleStart}
              >
                ● Start capture
              </button>
            }
          >
            <button
              class="cs-btn danger"
              type="button"
              disabled={busy()}
              onClick={handleStop}
            >
              ■ Stop
            </button>
          </Show>
          <Show when={props.onClose}>
            <button class="cs-btn ghost" type="button" onClick={() => props.onClose?.()}>
              ✕
            </button>
          </Show>
        </div>
      </div>

      {/* Status bar */}
      <div class="cs-status">
        <span class={`dot ${active() ? "live" : ""}`} />
        <span>{active() ? "Recording" : "Idle"}</span>
        <span class="sep">·</span>
        <span>{session()?.entries.length ?? 0} frames</span>
        <span class="sep">·</span>
        <span>
          Steps {doneCount()}/{steps().length}
        </span>
        <Show when={!props.connected}>
          <span class="sep">·</span>
          <span class="warn">Not connected — connect a device for live TX/RX</span>
        </Show>
      </div>

      <Show when={error()}>
        <div class="cs-error">{error()}</div>
      </Show>

      {/* Tabs */}
      <div class="cs-tabs">
        {(
          [
            ["log", "Hex log"],
            ["steps", "Guided"],
            ["gatt", "GATT"],
            ["write", "Raw write"],
          ] as const
        ).map(([id, label]) => (
          <button
            type="button"
            class={`cs-tab ${tab() === id ? "active" : ""}`}
            onClick={() => setTab(id)}
          >
            {label}
            <Show when={id === "log" && (session()?.entries.length ?? 0) > 0}>
              <span class="badge">{session()!.entries.length}</span>
            </Show>
          </button>
        ))}
      </div>

      {/* TAB: Log */}
      <Show when={tab() === "log"}>
        <div class="cs-toolbar">
          <div class="cs-filters">
            {(["all", "tx", "rx", "info"] as const).map((f) => (
              <button
                type="button"
                class={`chip ${filter() === f ? "active" : ""}`}
                onClick={() => setFilter(f)}
              >
                {f.toUpperCase()}
              </button>
            ))}
          </div>
          <div class="cs-toolbar-right">
            <button class="cs-btn ghost sm" type="button" onClick={handleClear}>
              Clear
            </button>
            <button class="cs-btn ghost sm" type="button" onClick={handleExportJson}>
              Export JSON
            </button>
            <button class="cs-btn ghost sm" type="button" onClick={handleExportMd}>
              Export MD
            </button>
          </div>
        </div>

        <div class="cs-log">
          <Show
            when={entries().length > 0}
            fallback={
              <div class="cs-empty">
                <p>No frames yet</p>
                <span>
                  Start capture, then use controls or Raw write to generate traffic
                </span>
              </div>
            }
          >
            <For each={entries()}>
              {(e) => <LogRow entry={e} />}
            </For>
            <div ref={logEnd} />
          </Show>
        </div>

        <div class="cs-note-bar">
          <input
            type="text"
            placeholder="Add note to log…"
            value={noteInput()}
            onInput={(ev) => setNoteInput(ev.currentTarget.value)}
            onKeyDown={(ev) => ev.key === "Enter" && handleNote()}
          />
          <button class="cs-btn secondary sm" type="button" onClick={handleNote}>
            Note
          </button>
        </div>
      </Show>

      {/* TAB: Guided steps */}
      <Show when={tab() === "steps"}>
        <div class="cs-steps">
          <p class="cs-hint">
            Follow each stimulus, mark done when finished. Frames are tagged with the
            active step.
          </p>
          <For each={steps()}>
            {(step) => (
              <StepCard
                step={step}
                active={currentStep() === step.id}
                onSelect={() => handleStep(step.id)}
                onToggle={(done) => handleMark(step.id, done)}
              />
            )}
          </For>
        </div>
      </Show>

      {/* TAB: GATT */}
      <Show when={tab() === "gatt"}>
        <div class="cs-gatt">
          <Show
            when={gatt().length > 0}
            fallback={
              <div class="cs-empty">
                <p>No GATT map</p>
                <span>Connect a device to discover services</span>
              </div>
            }
          >
            <table>
              <thead>
                <tr>
                  <th>Service</th>
                  <th>Characteristic</th>
                  <th>Props</th>
                </tr>
              </thead>
              <tbody>
                <For each={gatt()}>
                  {(c) => (
                    <tr>
                      <td class="mono">{shortUuid(c.serviceUuid)}</td>
                      <td class="mono">{shortUuid(c.uuid)}</td>
                      <td>
                        <For each={c.properties}>
                          {(p) => <span class="prop-tag">{p}</span>}
                        </For>
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </Show>
        </div>
      </Show>

      {/* TAB: Raw write */}
      <Show when={tab() === "write"}>
        <div class="cs-write">
          <p class="cs-hint">
            Send raw hex to the write characteristic. Useful for probing unknown opcodes.
          </p>
          <div class="cs-presets">
            {[
              ["ANC Off", "BA 34 00 FF"],
              ["ANC On", "BA 34 01 68"],
              ["Transparency", "BA 34 02 FF"],
              ["EQ Balanced", "BA 43 00"],
              ["EQ Bass", "BA 43 01"],
              ["Game ON", "BA 24 01"],
              ["Game OFF", "BA 24 00"],
              ["EQ Query", "BA 42"],
            ].map(([label, hex]) => (
              <button
                type="button"
                class="chip"
                onClick={() => setHexInput(hex)}
              >
                {label}
              </button>
            ))}
          </div>
          <div class="cs-write-row">
            <input
              class="mono"
              type="text"
              value={hexInput()}
              onInput={(ev) => setHexInput(ev.currentTarget.value)}
              placeholder="BA 34 01 68"
            />
            <button
              class="cs-btn primary"
              type="button"
              disabled={busy() || !props.connected}
              onClick={handleWrite}
            >
              Write
            </button>
          </div>
          <Show when={!props.connected}>
            <p class="cs-hint warn">Connect a device first to send writes.</p>
          </Show>
        </div>
      </Show>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Subcomponents
// ---------------------------------------------------------------------------

const LogRow: Component<{ entry: CaptureEntry }> = (props) => {
  const e = () => props.entry;
  const time = () => {
    const d = new Date(e().tsMs);
    return d.toLocaleTimeString("en-GB", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      fractionalSecondDigits: 3,
    } as Intl.DateTimeFormatOptions);
  };

  return (
    <div class={`log-row ${e().direction}`}>
      <span class="log-time">{time()}</span>
      <span class={`log-dir ${e().direction}`}>
        {e().direction === "tx" ? "TX" : e().direction === "rx" ? "RX" : "··"}
      </span>
      <span class="log-hex mono">
        <Show when={e().hex} fallback={<span class="muted">{e().label ?? "—"}</span>}>
          {e().hex}
        </Show>
      </span>
      <span class="log-meta">
        <Show when={e().decoded}>
          <span class="decoded">{e().decoded}</span>
        </Show>
        <Show when={e().label && e().hex}>
          <span class="label">{e().label}</span>
        </Show>
      </span>
    </div>
  );
};

const StepCard: Component<{
  step: GuidedStep;
  active: boolean;
  onSelect: () => void;
  onToggle: (done: boolean) => void;
}> = (props) => (
  <div
    class={`step-card ${props.active ? "active" : ""} ${props.step.done ? "done" : ""}`}
    onClick={() => props.onSelect()}
  >
    <div class="step-top">
      <label class="step-check" onClick={(ev) => ev.stopPropagation()}>
        <input
          type="checkbox"
          checked={props.step.done}
          onChange={(ev) => props.onToggle(ev.currentTarget.checked)}
        />
        <span class="step-title">{props.step.title}</span>
      </label>
      <span class="step-count">{props.step.entryIds.length} f</span>
    </div>
    <p class="step-instruction">{props.step.instruction}</p>
    <Show when={props.active}>
      <span class="step-active-badge">Recording into this step</span>
    </Show>
  </div>
);

function shortUuid(u: string): string {
  if (u.length > 12) return u.slice(0, 8) + "…" + u.slice(-4);
  return u;
}

export default CaptureStudio;
