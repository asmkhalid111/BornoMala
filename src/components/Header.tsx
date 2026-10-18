import React from "react";
import { EngineStatus, InputMode } from "../types";

interface HeaderProps {
  status: EngineStatus | null;
  loading: boolean;
  activeMode: InputMode;
  onSelectMode: (mode: InputMode) => void;
  onOpenSettings: () => void;
}

export const Header: React.FC<HeaderProps> = ({ status, loading, activeMode, onSelectMode, onOpenSettings }) => {
  const isModeActive = (target: "Phonetic" | "National" | "Probhat" | "Bijoy" | "English") => {
    if (target === "Phonetic") return activeMode === "Phonetic";
    if (target === "English") return activeMode === "EnglishPassthrough";
    if (typeof activeMode === "object" && "Fixed" in activeMode) {
      return activeMode.Fixed === target;
    }
    return false;
  };

  return (
    <header className="glass-panel" style={{ padding: "16px 24px", marginBottom: "20px", display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "14px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
        <div
          style={{
            width: "42px",
            height: "42px",
            borderRadius: "10px",
            background: "linear-gradient(135deg, #0ea5e9 0%, #10b981 100%)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontWeight: "700",
            fontSize: "22px",
            color: "#ffffff",
            boxShadow: "0 0 16px rgba(14, 165, 233, 0.4)",
            fontFamily: "var(--font-bangla)",
          }}
        >
          ব
        </div>
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <h1 style={{ fontSize: "18px", fontWeight: "700", letterSpacing: "-0.02em" }}>
              BornoMala (বর্ণমালা)
            </h1>
            <span
              style={{
                fontSize: "11px",
                fontWeight: "600",
                textTransform: "uppercase",
                letterSpacing: "0.06em",
                padding: "2px 8px",
                borderRadius: "6px",
                backgroundColor: "rgba(16, 185, 129, 0.15)",
                color: "var(--accent-emerald)",
                border: "1px solid rgba(16, 185, 129, 0.3)",
              }}
            >
              Native TSF + Sandbox
            </span>
          </div>
          <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
            High-Performance Bangla Input Platform (Phonetic, National, Probhat, Bijoy)
          </p>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "12px", flexWrap: "wrap" }}>
        {/* Layout Mode Selector Buttons */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            backgroundColor: "rgba(17, 24, 39, 0.8)",
            padding: "3px",
            borderRadius: "8px",
            border: "1px solid rgba(255, 255, 255, 0.08)",
            gap: "2px",
          }}
        >
          <button
            onClick={() => onSelectMode("Phonetic")}
            style={{
              padding: "5px 10px",
              borderRadius: "6px",
              backgroundColor: isModeActive("Phonetic") ? "rgba(14, 165, 233, 0.25)" : "transparent",
              color: isModeActive("Phonetic") ? "var(--accent-cyan)" : "var(--text-secondary)",
              border: isModeActive("Phonetic") ? "1px solid rgba(14, 165, 233, 0.4)" : "1px solid transparent",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: isModeActive("Phonetic") ? "600" : "500",
              transition: "all 0.15s ease",
            }}
          >
            🇧🇩 Phonetic
          </button>

          <button
            onClick={() => onSelectMode({ Fixed: "National" })}
            style={{
              padding: "5px 10px",
              borderRadius: "6px",
              backgroundColor: isModeActive("National") ? "rgba(16, 185, 129, 0.25)" : "transparent",
              color: isModeActive("National") ? "var(--accent-emerald)" : "var(--text-secondary)",
              border: isModeActive("National") ? "1px solid rgba(16, 185, 129, 0.4)" : "1px solid transparent",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: isModeActive("National") ? "600" : "500",
              transition: "all 0.15s ease",
            }}
          >
            🏛️ National
          </button>

          <button
            onClick={() => onSelectMode({ Fixed: "Probhat" })}
            style={{
              padding: "5px 10px",
              borderRadius: "6px",
              backgroundColor: isModeActive("Probhat") ? "rgba(99, 102, 241, 0.25)" : "transparent",
              color: isModeActive("Probhat") ? "var(--accent-indigo)" : "var(--text-secondary)",
              border: isModeActive("Probhat") ? "1px solid rgba(99, 102, 241, 0.4)" : "1px solid transparent",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: isModeActive("Probhat") ? "600" : "500",
              transition: "all 0.15s ease",
            }}
          >
            🌅 Probhat
          </button>

          <button
            onClick={() => onSelectMode({ Fixed: "Bijoy" })}
            style={{
              padding: "5px 10px",
              borderRadius: "6px",
              backgroundColor: isModeActive("Bijoy") ? "rgba(245, 158, 11, 0.25)" : "transparent",
              color: isModeActive("Bijoy") ? "var(--accent-amber)" : "var(--text-secondary)",
              border: isModeActive("Bijoy") ? "1px solid rgba(245, 158, 11, 0.4)" : "1px solid transparent",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: isModeActive("Bijoy") ? "600" : "500",
              transition: "all 0.15s ease",
            }}
          >
            ⌨️ Bijoy
          </button>

          <button
            onClick={() => onSelectMode("EnglishPassthrough")}
            style={{
              padding: "5px 10px",
              borderRadius: "6px",
              backgroundColor: isModeActive("English") ? "rgba(244, 63, 94, 0.25)" : "transparent",
              color: isModeActive("English") ? "var(--accent-rose)" : "var(--text-secondary)",
              border: isModeActive("English") ? "1px solid rgba(244, 63, 94, 0.4)" : "1px solid transparent",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: isModeActive("English") ? "600" : "500",
              transition: "all 0.15s ease",
            }}
          >
            🔤 English
          </button>
        </div>

        {/* Engine Ready Indicator */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "8px",
            padding: "6px 12px",
            borderRadius: "999px",
            backgroundColor: status?.ready
              ? "rgba(16, 185, 129, 0.12)"
              : "rgba(244, 63, 94, 0.12)",
            border: `1px solid ${
              status?.ready
                ? "rgba(16, 185, 129, 0.3)"
                : "rgba(244, 63, 94, 0.3)"
            }`,
          }}
        >
          <span
            style={{
              width: "8px",
              height: "8px",
              borderRadius: "50%",
              backgroundColor: status?.ready ? "var(--accent-emerald)" : "var(--accent-rose)",
              boxShadow: status?.ready
                ? "0 0 10px var(--accent-emerald)"
                : "0 0 10px var(--accent-rose)",
              display: "inline-block",
            }}
          />
          <span
            style={{
              fontSize: "12px",
              fontWeight: "600",
              color: status?.ready ? "var(--accent-emerald)" : "var(--accent-rose)",
            }}
          >
            {loading ? "Connecting..." : status?.ready ? "Engine Ready" : "Disconnected"}
          </span>
        </div>

        {/* Settings Open Button */}
        <button
          id="settings-open-btn"
          onClick={onOpenSettings}
          aria-label="Open Settings"
          style={{
            display: "flex",
            alignItems: "center",
            gap: "6px",
            padding: "6px 14px",
            borderRadius: "8px",
            backgroundColor: "rgba(14, 165, 233, 0.15)",
            color: "var(--accent-cyan)",
            border: "1px solid rgba(14, 165, 233, 0.35)",
            cursor: "pointer",
            fontSize: "12px",
            fontWeight: "600",
            transition: "all 0.2s ease",
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = "rgba(14, 165, 233, 0.25)";
            e.currentTarget.style.borderColor = "rgba(14, 165, 233, 0.6)";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = "rgba(14, 165, 233, 0.15)";
            e.currentTarget.style.borderColor = "rgba(14, 165, 233, 0.35)";
          }}
        >
          <span aria-hidden="true" style={{ fontSize: "14px" }}>⚙️</span>
          Settings
        </button>
      </div>
    </header>
  );
};
