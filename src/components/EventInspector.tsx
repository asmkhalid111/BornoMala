import React from "react";
import { EventLogItem } from "../types";

interface EventInspectorProps {
  logs: EventLogItem[];
  onClearLogs: () => void;
}

export const EventInspector: React.FC<EventInspectorProps> = ({ logs, onClearLogs }) => {
  return (
    <div className="glass-panel" style={{ padding: "20px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
        <h3 style={{ fontSize: "14px", fontWeight: "600", display: "flex", alignItems: "center", gap: "8px" }}>
          <span>Normalized Event & State Inspector</span>
          <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
            ({logs.length} logged events)
          </span>
          <span style={{ fontSize: "11px", color: "var(--text-muted)", fontWeight: "normal" }}>
            • Latency measures round-trip IPC + engine execution
          </span>
        </h3>
        {logs.length > 0 && (
          <button
            onClick={onClearLogs}
            style={{
              background: "none",
              border: "none",
              color: "var(--text-muted)",
              fontSize: "12px",
              cursor: "pointer",
              textDecoration: "underline",
            }}
          >
            Clear Log
          </button>
        )}
      </div>

      <div
        style={{
          maxHeight: "240px",
          overflowY: "auto",
          backgroundColor: "rgba(11, 15, 25, 0.75)",
          borderRadius: "8px",
          border: "1px solid var(--border-subtle)",
          fontFamily: "var(--font-mono)",
          fontSize: "12px",
        }}
      >
        <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left" }}>
          <thead>
            <tr style={{ borderBottom: "1px solid var(--border-subtle)", color: "var(--text-muted)", fontSize: "11px" }}>
              <th style={{ padding: "8px 12px" }}>Time</th>
              <th style={{ padding: "8px 12px" }}>Normalized Event</th>
              <th style={{ padding: "8px 12px" }}>Raw Buffer</th>
              <th style={{ padding: "8px 12px" }}>Preedit (Bangla)</th>
              <th style={{ padding: "8px 12px" }}>Cursor</th>
              <th style={{ padding: "8px 12px" }}>Committed Effect</th>
              <th style={{ padding: "8px 12px" }}>Status</th>
              <th style={{ padding: "8px 12px", textAlign: "right" }}>Latency</th>
            </tr>
          </thead>
          <tbody>
            {logs.length > 0 ? (
              logs.map((log) => (
                <tr
                  key={log.id}
                  style={{
                    borderBottom: "1px solid rgba(255, 255, 255, 0.04)",
                    color: "var(--text-secondary)",
                  }}
                >
                  <td style={{ padding: "6px 12px", color: "var(--text-muted)" }}>{log.time}</td>
                  <td style={{ padding: "6px 12px", color: "var(--accent-cyan)", fontWeight: "500" }}>
                    {log.action}
                  </td>
                  <td style={{ padding: "6px 12px" }}>{log.rawBuffer || "—"}</td>
                  <td style={{ padding: "6px 12px", color: "#ffffff", fontFamily: "var(--font-bangla)", fontSize: "14px" }}>
                    {log.preedit || "—"}
                  </td>
                  <td style={{ padding: "6px 12px", color: "var(--accent-amber)" }}>
                    {log.cursorPos}
                  </td>
                  <td style={{ padding: "6px 12px", color: "var(--accent-emerald)", fontFamily: "var(--font-bangla)", fontSize: "14px" }}>
                    {log.committed ? `"${log.committed}"` : "—"}
                  </td>
                  <td style={{ padding: "6px 12px" }}>
                    {log.cancelled ? (
                      <span
                        style={{
                          fontSize: "10px",
                          padding: "2px 6px",
                          borderRadius: "4px",
                          backgroundColor: "rgba(244, 63, 94, 0.2)",
                          color: "var(--accent-rose)",
                          border: "1px solid rgba(244, 63, 94, 0.4)",
                        }}
                      >
                        Cancelled
                      </span>
                    ) : log.committed ? (
                      <span
                        style={{
                          fontSize: "10px",
                          padding: "2px 6px",
                          borderRadius: "4px",
                          backgroundColor: "rgba(16, 185, 129, 0.15)",
                          color: "var(--accent-emerald)",
                          border: "1px solid rgba(16, 185, 129, 0.3)",
                        }}
                      >
                        Committed
                      </span>
                    ) : (
                      <span style={{ color: "var(--text-muted)", fontSize: "11px" }}>Active</span>
                    )}
                  </td>
                  <td style={{ padding: "6px 12px", textAlign: "right", color: "var(--text-muted)" }}>
                    {log.latencyMs.toFixed(1)}ms
                  </td>
                </tr>
              ))
            ) : (
              <tr>
                <td colSpan={8} style={{ padding: "16px", textAlign: "center", color: "var(--text-muted)" }}>
                  No IPC events logged yet. Type in the sandbox above to view live events and output effects.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
