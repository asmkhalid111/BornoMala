import { useState, useEffect, useCallback } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { Header } from "./components/Header";
import { SandboxTypingArea } from "./components/SandboxTypingArea";
import { VirtualKeyboard } from "./components/VirtualKeyboard";
import { DeterministicTestRunner } from "./components/DeterministicTestRunner";
import { EventInspector } from "./components/EventInspector";
import { SettingsDrawer } from "./components/SettingsDrawer";
import {
  EngineState,
  EngineStatus,
  EngineEvent,
  EventLogItem,
  DeterministicTestResult,
  ProcessResult,
  PhysicalKey,
  Modifiers,
  InputMode,
  BornoConfig,
  ConfigResponse,
  PersistenceStatus,
} from "./types";

export function App() {
  const [status, setStatus] = useState<EngineStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [ipcError, setIpcError] = useState<string | null>(null);
  const inTauri = isTauri();

  // Settings & Configuration state
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [config, setConfig] = useState<BornoConfig | null>(null);
  const [persistenceStatus, setPersistenceStatus] = useState<PersistenceStatus | null>(null);
  const [configPath, setConfigPath] = useState<string>("");

  const [engineState, setEngineState] = useState<EngineState>({
    preedit: "",
    raw_buffer: "",
    cursor_pos: 0,
    candidates: [],
    selected_candidate_index: null,
    is_composing: false,
    mode: "Phonetic",
    status: {
      name: "Borno Core",
      version: "0.1.0",
      active_mode: "Phonetic (Sandbox)",
      ready: false,
    },
  });
  const [committedHistory, setCommittedHistory] = useState("");
  const [logs, setLogs] = useState<EventLogItem[]>([]);

  // Virtual keyboard interactive states
  const [activePhysicalKey, setActivePhysicalKey] = useState<PhysicalKey | null>(null);
  const [shiftActive, setShiftActive] = useState<boolean>(false);
  const [altGrActive, setAltGrActive] = useState<boolean>(false);

  // Fetch initial engine status
  const loadStatus = useCallback(async () => {
    if (!inTauri) {
      setLoading(false);
      return;
    }

    setLoading(true);
    setIpcError(null);
    try {
      const res = await invoke<EngineStatus>("get_engine_status");
      setStatus(res);
      setEngineState((prev) => ({
        ...prev,
        status: res,
      }));
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error("Tauri invoke get_engine_status failed:", err);
      setIpcError(`IPC Connection Failed: ${msg}`);
    } finally {
      setLoading(false);
    }
  }, [inTauri]);

  const loadConfig = useCallback(async () => {
    if (!inTauri) return;
    try {
      const res = await invoke<ConfigResponse>("get_config");
      setConfig(res.config);
      setPersistenceStatus(res.status);
      setConfigPath(res.config_path);
    } catch (err: unknown) {
      console.error("Tauri invoke get_config failed:", err);
    }
  }, [inTauri]);

  useEffect(() => {
    loadStatus();
    loadConfig();
  }, [loadStatus, loadConfig]);

  const handleSaveConfig = async (newConfig: BornoConfig) => {
    if (!inTauri) return;
    const saved = await invoke<BornoConfig>("save_config", { config: newConfig });
    setConfig(saved);
  };

  const handleResetConfig = async () => {
    if (!inTauri) return;
    const res = await invoke<ConfigResponse>("reset_config");
    setConfig(res.config);
    setPersistenceStatus(res.status);
    setConfigPath(res.config_path);
  };

  const dispatchEvent = useCallback(async (event: EngineEvent, label: string) => {
    if (!inTauri) {
      setIpcError("Cannot send event: Currently running inside an external browser without the native Tauri Rust bridge. Please switch to the running native desktop window.");
      return;
    }

    const t0 = performance.now();
    try {
      const result = await invoke<ProcessResult>("send_engine_event", { event });
      const elapsed = performance.now() - t0;

      setEngineState(result.state);
      if (result.committed_text) {
        setCommittedHistory((prev) => prev + result.committed_text);
      }
      setIpcError(null);

      const logItem: EventLogItem = {
        id: Math.random().toString(36).substring(2, 9),
        time: new Date().toLocaleTimeString(),
        action: label,
        rawBuffer: result.state.raw_buffer,
        preedit: result.state.preedit,
        cursorPos: result.state.cursor_pos,
        committed: result.committed_text,
        cancelled: result.cancelled,
        latencyMs: elapsed,
      };

      setLogs((prev) => [logItem, ...prev.slice(0, 24)]);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error("Failed to send engine event:", err);
      setIpcError(`Event Error: ${msg}`);
    }
  }, [inTauri]);

  const handleSendKey = useCallback((key: PhysicalKey, modifiers: Modifiers) => {
    setActivePhysicalKey(key);
    setTimeout(() => setActivePhysicalKey(null), 150);
    dispatchEvent(
      { type: "InputKey", payload: { key, modifiers } },
      `InputKey(${key}, shift=${modifiers.shift}, altgr=${modifiers.altgr})`
    );
  }, [dispatchEvent]);

  const handleSendChar = useCallback((c: string) => {
    dispatchEvent({ type: "InputChar", payload: c }, `InputChar('${c}')`);
  }, [dispatchEvent]);

  const handleBackspace = useCallback(() => {
    dispatchEvent({ type: "Backspace" }, "Backspace");
  }, [dispatchEvent]);

  const handleDelete = useCallback(() => {
    dispatchEvent({ type: "Delete" }, "Delete");
  }, [dispatchEvent]);

  const handleCommit = useCallback(() => {
    dispatchEvent({ type: "Commit" }, "Commit");
  }, [dispatchEvent]);

  const handleCancel = useCallback(() => {
    dispatchEvent({ type: "Cancel" }, "Cancel");
  }, [dispatchEvent]);

  const handleReset = useCallback(async () => {
    if (!inTauri) return;
    const t0 = performance.now();
    try {
      const result = await invoke<ProcessResult>("reset_engine");
      const elapsed = performance.now() - t0;

      setEngineState(result.state);
      setIpcError(null);
      setLogs((prev) => [
        {
          id: Math.random().toString(36).substring(2, 9),
          time: new Date().toLocaleTimeString(),
          action: "Reset",
          rawBuffer: "",
          preedit: "",
          cursorPos: 0,
          committed: null,
          cancelled: false,
          latencyMs: elapsed,
        },
        ...prev.slice(0, 24),
      ]);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error("Failed to reset engine:", err);
      setIpcError(`Reset Error: ${msg}`);
    }
  }, [inTauri]);

  const handleNextCandidate = useCallback(() => {
    dispatchEvent({ type: "NextCandidate" }, "NextCandidate");
  }, [dispatchEvent]);

  const handlePrevCandidate = useCallback(() => {
    dispatchEvent({ type: "PrevCandidate" }, "PrevCandidate");
  }, [dispatchEvent]);

  const handleSelectCandidate = useCallback((idx: number) => {
    dispatchEvent({ type: "SelectCandidate", payload: idx }, `SelectCandidate(${idx})`);
  }, [dispatchEvent]);

  const handleMoveCursorLeft = useCallback(() => {
    dispatchEvent({ type: "MoveCursorLeft" }, "MoveCursorLeft");
  }, [dispatchEvent]);

  const handleMoveCursorRight = useCallback(() => {
    dispatchEvent({ type: "MoveCursorRight" }, "MoveCursorRight");
  }, [dispatchEvent]);

  const handleSetCursor = useCallback((pos: number) => {
    dispatchEvent({ type: "SetCursor", payload: pos }, `SetCursor(${pos})`);
  }, [dispatchEvent]);

  const handleSelectMode = useCallback((mode: InputMode) => {
    const modeName = typeof mode === "string" ? mode : mode.Fixed;
    dispatchEvent({ type: "SwitchMode", payload: mode }, `SwitchMode('${modeName}')`);
  }, [dispatchEvent]);

  const handleToggleShift = useCallback(() => {
    setShiftActive((prev) => !prev);
  }, []);

  const handleToggleAltGr = useCallback(() => {
    setAltGrActive((prev) => !prev);
  }, []);

  const handleQuickInput = useCallback(async (text: string) => {
    await handleReset();
    await dispatchEvent({ type: "InputString", payload: text }, `InputString("${text}")`);
  }, [handleReset, dispatchEvent]);

  const handleAppendCommitted = useCallback((text: string) => {
    setCommittedHistory((prev) => (prev ? `${prev} ${text}` : text));
  }, []);

  const handleRunDeterministicTest = useCallback(async (): Promise<DeterministicTestResult> => {
    if (!inTauri) {
      throw new Error("Cannot run test in external browser: Native Tauri bridge not detected.");
    }
    const res = await invoke<DeterministicTestResult>("run_deterministic_test");
    const freshStatus = await invoke<EngineStatus>("get_engine_status");
    setStatus(freshStatus);
    setIpcError(null);
    return res;
  }, [inTauri]);

  return (
    <div style={{ maxWidth: "1050px", margin: "0 auto", padding: "24px 20px" }}>
      {/* Environment Notice Banner */}
      {!inTauri && (
        <div
          style={{
            padding: "14px 20px",
            marginBottom: "18px",
            borderRadius: "10px",
            backgroundColor: "rgba(245, 158, 11, 0.15)",
            border: "1px solid rgba(245, 158, 11, 0.4)",
            display: "flex",
            alignItems: "center",
            gap: "14px",
          }}
        >
          <span style={{ fontSize: "24px" }}>⚠️</span>
          <div style={{ fontSize: "13px", lineHeight: "1.5" }}>
            <strong style={{ color: "var(--accent-amber)" }}>Web Browser View Detected:</strong>
            <p style={{ color: "var(--text-secondary)", marginTop: "2px" }}>
              You are currently viewing this page in an external web browser. The native Rust engine (<code>borno-core</code>)
              connects directly to the <strong>native desktop application window</strong>. Please switch to the running{" "}
              <strong style={{ color: "#ffffff" }}>Borno Bangla Input — Sandbox (M4)</strong> window on your Windows taskbar to test live typing and verify the real Rust core.
            </p>
          </div>
        </div>
      )}

      {/* IPC Error Banner */}
      {ipcError && (
        <div
          style={{
            padding: "12px 18px",
            marginBottom: "18px",
            borderRadius: "10px",
            backgroundColor: "rgba(244, 63, 94, 0.15)",
            border: "1px solid rgba(244, 63, 94, 0.4)",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <div style={{ fontSize: "13px", color: "var(--accent-rose)" }}>
            <strong>Engine Communication Error:</strong> {ipcError}
          </div>
          <button
            onClick={loadStatus}
            style={{
              padding: "4px 12px",
              backgroundColor: "rgba(244, 63, 94, 0.25)",
              color: "#ffffff",
              border: "1px solid rgba(244, 63, 94, 0.5)",
              borderRadius: "6px",
              cursor: "pointer",
              fontSize: "12px",
            }}
          >
            Retry Connection
          </button>
        </div>
      )}

      <Header
        status={status}
        loading={loading}
        activeMode={engineState.mode}
        onSelectMode={handleSelectMode}
        onOpenSettings={() => setIsSettingsOpen(true)}
      />

      <SettingsDrawer
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        config={config}
        persistenceStatus={persistenceStatus}
        configPath={configPath}
        onSave={handleSaveConfig}
        onReset={handleResetConfig}
      />

      <SandboxTypingArea
        engineState={engineState}
        committedHistory={committedHistory}
        onSendChar={handleSendChar}
        onBackspace={handleBackspace}
        onDelete={handleDelete}
        onCommit={handleCommit}
        onCancel={handleCancel}
        onReset={handleReset}
        onNextCandidate={handleNextCandidate}
        onPrevCandidate={handlePrevCandidate}
        onSelectCandidate={handleSelectCandidate}
        onMoveCursorLeft={handleMoveCursorLeft}
        onMoveCursorRight={handleMoveCursorRight}
        onSetCursor={handleSetCursor}
        onQuickInput={handleQuickInput}
        onAppendCommitted={handleAppendCommitted}
        onSendKey={handleSendKey}
      />

      <VirtualKeyboard
        activeMode={engineState.mode}
        activePhysicalKey={activePhysicalKey}
        shiftActive={shiftActive}
        altGrActive={altGrActive}
        onKeyClick={handleSendKey}
        onBackspaceClick={handleBackspace}
        onCommitClick={handleCommit}
        onToggleShift={handleToggleShift}
        onToggleAltGr={handleToggleAltGr}
      />

      <DeterministicTestRunner onRunTest={handleRunDeterministicTest} />

      <EventInspector logs={logs} onClearLogs={() => setLogs([])} />

      <footer style={{ marginTop: "24px", textAlign: "center", color: "var(--text-muted)", fontSize: "12px" }}>
        Bangla Input Platform — Borno Sandbox • Built with Rust Core & Tauri v2 • Offline-first
      </footer>
    </div>
  );
}

export default App;
