import React, { useState } from "react";
import { DeterministicTestResult } from "../types";

interface DeterministicTestRunnerProps {
  onRunTest: () => Promise<DeterministicTestResult>;
}

export const DeterministicTestRunner: React.FC<DeterministicTestRunnerProps> = ({ onRunTest }) => {
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<DeterministicTestResult | null>(null);

  const handleRun = async () => {
    setRunning(true);
    try {
      const res = await onRunTest();
      setResult(res);
    } catch (err) {
      console.error("Test execution failed:", err);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="glass-panel" style={{ padding: "20px", marginBottom: "20px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
        <div>
          <h3 style={{ fontSize: "15px", fontWeight: "600", display: "flex", alignItems: "center", gap: "8px" }}>
            <span>Deterministic Engine Verification (M0 Gate)</span>
            {result && (
              <span
                style={{
                  fontSize: "11px",
                  padding: "2px 8px",
                  borderRadius: "4px",
                  backgroundColor: result.success ? "rgba(16, 185, 129, 0.2)" : "rgba(244, 63, 94, 0.2)",
                  color: result.success ? "var(--accent-emerald)" : "var(--accent-rose)",
                  fontWeight: "600",
                }}
              >
                {result.success ? "PASS (Verified)" : "FAIL"}
              </span>
            )}
          </h3>
          <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
            Tests the end-to-end pipeline: frontend → Tauri IPC → BornoEngine state machine → Unicode output
          </p>
        </div>

        <button
          onClick={handleRun}
          disabled={running}
          style={{
            padding: "8px 18px",
            backgroundColor: "var(--accent-cyan)",
            color: "#0b0f19",
            border: "none",
            borderRadius: "6px",
            fontWeight: "600",
            fontSize: "13px",
            cursor: running ? "not-allowed" : "pointer",
            boxShadow: "0 0 12px rgba(6, 182, 212, 0.35)",
            transition: "all 0.2s ease",
            opacity: running ? 0.7 : 1,
          }}
        >
          {running ? "Executing Test..." : "Run Deterministic Test"}
        </button>
      </div>

      {result ? (
        <div
          style={{
            padding: "14px",
            borderRadius: "8px",
            backgroundColor: "rgba(11, 15, 25, 0.8)",
            border: `1px solid ${result.success ? "rgba(16, 185, 129, 0.3)" : "rgba(244, 63, 94, 0.3)"}`,
            fontSize: "13px",
          }}
        >
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "12px", marginBottom: "8px" }}>
            <div>
              <span style={{ color: "var(--text-muted)", fontSize: "11px", display: "block" }}>Input Sequence</span>
              <strong style={{ fontFamily: "var(--font-mono)", color: "var(--text-primary)" }}>{result.input_sequence}</strong>
            </div>
            <div>
              <span style={{ color: "var(--text-muted)", fontSize: "11px", display: "block" }}>Expected Preedit</span>
              <strong style={{ fontFamily: "var(--font-bangla)", fontSize: "16px", color: "var(--text-primary)" }}>
                {result.expected_preedit}
              </strong>
            </div>
            <div>
              <span style={{ color: "var(--text-muted)", fontSize: "11px", display: "block" }}>Actual Preedit</span>
              <strong
                style={{
                  fontFamily: "var(--font-bangla)",
                  fontSize: "16px",
                  color: result.success ? "var(--accent-emerald)" : "var(--accent-rose)",
                }}
              >
                {result.actual_preedit}
              </strong>
            </div>
            <div>
              <span style={{ color: "var(--text-muted)", fontSize: "11px", display: "block" }}>Candidates</span>
              <strong style={{ fontFamily: "var(--font-bangla)", color: "var(--text-primary)" }}>
                {result.candidates.join(", ")}
              </strong>
            </div>
          </div>
          <div style={{ color: result.success ? "var(--accent-emerald)" : "var(--accent-rose)", fontSize: "12px", marginTop: "6px" }}>
            {result.message}
          </div>
        </div>
      ) : (
        <div style={{ fontSize: "12px", color: "var(--text-muted)", fontStyle: "italic" }}>
          Click "Run Deterministic Test" to dispatch the verification sequence to the Rust core.
        </div>
      )}
    </div>
  );
};
