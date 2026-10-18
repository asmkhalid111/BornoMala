import React, { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { InputMode, PhysicalKey, Modifiers, LayoutDefinition } from "../types";

interface VirtualKeyboardProps {
  activeMode: InputMode;
  activePhysicalKey: PhysicalKey | null;
  shiftActive: boolean;
  altGrActive: boolean;
  onKeyClick: (key: PhysicalKey, modifiers: Modifiers) => void;
  onBackspaceClick: () => void;
  onCommitClick: () => void;
  onToggleShift: () => void;
  onToggleAltGr: () => void;
}

interface KeyConfig {
  code: PhysicalKey | "Backspace" | "Tab" | "CapsLock" | "Enter" | "ShiftLeft" | "ShiftRight" | "CtrlLeft" | "AltLeft" | "AltRight" | "CtrlRight";
  label?: string;
  width?: number; // relative flex or width
  isModifier?: boolean;
  isFunction?: boolean;
}

export const VirtualKeyboard: React.FC<VirtualKeyboardProps> = ({
  activeMode,
  activePhysicalKey,
  shiftActive,
  altGrActive,
  onKeyClick,
  onBackspaceClick,
  onCommitClick,
  onToggleShift,
  onToggleAltGr,
}) => {
  const [layoutDef, setLayoutDef] = useState<LayoutDefinition | null>(null);
  const [loadingLayout, setLoadingLayout] = useState(false);

  // Determine current layout ID if in Fixed mode
  const currentFixedId = useMemo(() => {
    if (typeof activeMode === "object" && "Fixed" in activeMode) {
      return activeMode.Fixed.toLowerCase();
    }
    return null;
  }, [activeMode]);

  // Fetch layout definition from Tauri backend
  useEffect(() => {
    if (!currentFixedId) {
      setLayoutDef(null);
      return;
    }

    let isMounted = true;
    setLoadingLayout(true);
    invoke<LayoutDefinition>("get_layout_definition", { layoutId: currentFixedId })
      .then((def) => {
        if (isMounted) {
          setLayoutDef(def);
          setLoadingLayout(false);
        }
      })
      .catch((err) => {
        console.error("Failed to load layout definition:", err);
        if (isMounted) setLoadingLayout(false);
      });

    return () => {
      isMounted = false;
    };
  }, [currentFixedId]);

  // Keyboard Rows Definition
  const rowE: KeyConfig[] = [
    { code: "Backquote", label: "`" },
    { code: "Digit1", label: "1" },
    { code: "Digit2", label: "2" },
    { code: "Digit3", label: "3" },
    { code: "Digit4", label: "4" },
    { code: "Digit5", label: "5" },
    { code: "Digit6", label: "6" },
    { code: "Digit7", label: "7" },
    { code: "Digit8", label: "8" },
    { code: "Digit9", label: "9" },
    { code: "Digit0", label: "0" },
    { code: "Minus", label: "-" },
    { code: "Equal", label: "=" },
    { code: "Backspace", label: "⌫ Backspace", width: 1.8, isFunction: true },
  ];

  const rowD: KeyConfig[] = [
    { code: "Tab", label: "Tab", width: 1.5, isFunction: true },
    { code: "KeyQ", label: "Q" },
    { code: "KeyW", label: "W" },
    { code: "KeyE", label: "E" },
    { code: "KeyR", label: "R" },
    { code: "KeyT", label: "T" },
    { code: "KeyY", label: "Y" },
    { code: "KeyU", label: "U" },
    { code: "KeyI", label: "I" },
    { code: "KeyO", label: "O" },
    { code: "KeyP", label: "P" },
    { code: "BracketLeft", label: "[" },
    { code: "BracketRight", label: "]" },
    { code: "Backslash", label: "\\", width: 1.3 },
  ];

  const rowC: KeyConfig[] = [
    { code: "CapsLock", label: "Caps", width: 1.8, isFunction: true },
    { code: "KeyA", label: "A" },
    { code: "KeyS", label: "S" },
    { code: "KeyD", label: "D" },
    { code: "KeyF", label: "F" },
    { code: "KeyG", label: "G" },
    { code: "KeyH", label: "H" },
    { code: "KeyJ", label: "J" },
    { code: "KeyK", label: "K" },
    { code: "KeyL", label: "L" },
    { code: "Semicolon", label: ";" },
    { code: "Quote", label: "'" },
    { code: "Enter", label: "⏎ Enter", width: 2.0, isFunction: true },
  ];

  const rowB: KeyConfig[] = [
    { code: "ShiftLeft", label: "⇧ Shift", width: 2.2, isModifier: true },
    { code: "KeyZ", label: "Z" },
    { code: "KeyX", label: "X" },
    { code: "KeyC", label: "C" },
    { code: "KeyV", label: "V" },
    { code: "KeyB", label: "B" },
    { code: "KeyN", label: "N" },
    { code: "KeyM", label: "M" },
    { code: "Comma", label: "," },
    { code: "Period", label: "." },
    { code: "Slash", label: "/" },
    { code: "ShiftRight", label: "⇧ Shift", width: 2.6, isModifier: true },
  ];

  const rowA: KeyConfig[] = [
    { code: "CtrlLeft", label: "Ctrl", width: 1.4, isFunction: true },
    { code: "AltLeft", label: "Alt", width: 1.4, isFunction: true },
    { code: "Space", label: "Space", width: 6.8 },
    { code: "AltRight", label: "AltGr", width: 1.4, isModifier: true },
    { code: "CtrlRight", label: "Ctrl", width: 1.4, isFunction: true },
  ];

  const renderKey = (config: KeyConfig) => {
    const isPhysical = config.code !== "Backspace" && config.code !== "Tab" && config.code !== "CapsLock" &&
      config.code !== "Enter" && config.code !== "ShiftLeft" && config.code !== "ShiftRight" &&
      config.code !== "CtrlLeft" && config.code !== "AltLeft" && config.code !== "AltRight" && config.code !== "CtrlRight";

    const isPressed = isPhysical && activePhysicalKey === config.code;
    const isShiftKey = config.code === "ShiftLeft" || config.code === "ShiftRight";
    const isAltGrKey = config.code === "AltRight";

    const isShiftActiveHighlight = isShiftKey && shiftActive;
    const isAltGrActiveHighlight = isAltGrKey && altGrActive;

    // Get characters for physical key
    let unshiftedChar = "";
    let shiftedChar = "";
    let altgrChar: string | null = null;

    if (isPhysical) {
      const physKey = config.code as PhysicalKey;
      if (layoutDef && layoutDef.keys[physKey]) {
        const m = layoutDef.keys[physKey];
        unshiftedChar = m.unshifted;
        shiftedChar = m.shifted;
        altgrChar = m.altgr || null;
      } else {
        // Fallback or phonetic / English
        unshiftedChar = config.label?.toLowerCase() || "";
        shiftedChar = config.label || "";
      }
    }

    const handleClick = () => {
      if (config.code === "Backspace") {
        onBackspaceClick();
      } else if (config.code === "Enter") {
        onCommitClick();
      } else if (isShiftKey) {
        onToggleShift();
      } else if (isAltGrKey) {
        onToggleAltGr();
      } else if (isPhysical) {
        const mods: Modifiers = {
          shift: shiftActive,
          altgr: altGrActive,
          ctrl: false,
          alt: false,
        };
        onKeyClick(config.code as PhysicalKey, mods);
      }
    };

    const flexWidth = config.width || 1;

    return (
      <button
        key={config.code}
        onClick={handleClick}
        style={{
          flex: `${flexWidth} 0 0`,
          height: "46px",
          minWidth: "32px",
          backgroundColor: isPressed
            ? "rgba(14, 165, 233, 0.45)"
            : isShiftActiveHighlight
            ? "rgba(99, 102, 241, 0.4)"
            : isAltGrActiveHighlight
            ? "rgba(245, 158, 11, 0.4)"
            : config.isModifier || config.isFunction
            ? "rgba(31, 41, 55, 0.65)"
            : "rgba(17, 24, 39, 0.8)",
          border: isPressed
            ? "1px solid var(--accent-cyan)"
            : isShiftActiveHighlight
            ? "1px solid var(--accent-indigo)"
            : isAltGrActiveHighlight
            ? "1px solid var(--accent-amber)"
            : "1px solid rgba(255, 255, 255, 0.08)",
          boxShadow: isPressed
            ? "0 0 12px rgba(14, 165, 233, 0.5), inset 0 0 8px rgba(14, 165, 233, 0.3)"
            : isShiftActiveHighlight
            ? "0 0 12px rgba(99, 102, 241, 0.4)"
            : isAltGrActiveHighlight
            ? "0 0 12px rgba(245, 158, 11, 0.4)"
            : "0 2px 4px rgba(0, 0, 0, 0.25)",
          borderRadius: "6px",
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          padding: "4px 6px",
          cursor: "pointer",
          transition: "all 0.1s ease",
          position: "relative",
          userSelect: "none",
        }}
        title={config.label || config.code}
      >
        {isPhysical ? (
          <>
            {/* Top Row: Shifted Char & Physical key label */}
            <div style={{ display: "flex", justifyContent: "space-between", width: "100%", alignItems: "center" }}>
              <span
                style={{
                  fontSize: "13px",
                  fontWeight: shiftActive ? "700" : "500",
                  color: shiftActive ? "var(--accent-cyan)" : "var(--text-secondary)",
                  fontFamily: "var(--font-bangla)",
                  lineHeight: "1",
                }}
              >
                {shiftedChar}
              </span>
              <span
                style={{
                  fontSize: "9px",
                  fontWeight: "600",
                  color: "var(--text-muted)",
                  fontFamily: "var(--font-mono)",
                }}
              >
                {config.label}
              </span>
            </div>

            {/* Bottom Row: Unshifted Char & AltGr Char */}
            <div style={{ display: "flex", justifyContent: "space-between", width: "100%", alignItems: "flex-end" }}>
              <span
                style={{
                  fontSize: config.code === "Space" ? "11px" : "15px",
                  fontWeight: !shiftActive ? "700" : "500",
                  color: !shiftActive ? "var(--text-primary)" : "var(--text-secondary)",
                  fontFamily: "var(--font-bangla)",
                  lineHeight: "1",
                }}
              >
                {config.code === "Space" ? "স্পেস (Space)" : unshiftedChar}
              </span>
              {altgrChar && (
                <span
                  style={{
                    fontSize: "12px",
                    fontWeight: altGrActive ? "700" : "500",
                    color: altGrActive ? "var(--accent-amber)" : "rgba(245, 158, 11, 0.6)",
                    fontFamily: "var(--font-bangla)",
                    lineHeight: "1",
                  }}
                >
                  {altgrChar}
                </span>
              )}
            </div>
          </>
        ) : (
          <div
            style={{
              width: "100%",
              height: "100%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: "12px",
              fontWeight: "600",
              color: isShiftActiveHighlight
                ? "var(--accent-indigo)"
                : isAltGrActiveHighlight
                ? "var(--accent-amber)"
                : "var(--text-secondary)",
            }}
          >
            {config.label}
          </div>
        )}
      </button>
    );
  };

  const layoutTitle = useMemo(() => {
    if (typeof activeMode === "object" && "Fixed" in activeMode) {
      if (activeMode.Fixed === "National") return "National (Jatiya) — Provisional BCC Baseline";
      if (activeMode.Fixed === "Probhat") return "Probhat — Verified Upstream Conformance";
      if (activeMode.Fixed === "Bijoy") return "Bijoy-Compatible — Provisional Unicode Baseline";
    }
    if (activeMode === "Phonetic") return "Phonetic Transliteration Layout";
    return "English Passthrough Layout";
  }, [activeMode]);

  return (
    <div className="glass-panel" style={{ padding: "18px 20px", marginBottom: "20px" }}>
      {/* Header bar */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <span style={{ fontSize: "16px" }}>⌨️</span>
          <div>
            <h3 style={{ fontSize: "14px", fontWeight: "600", color: "var(--text-primary)" }}>
              Interactive Virtual Keyboard
            </h3>
            <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "1px" }}>
              {layoutTitle}
            </p>
          </div>
        </div>

        {/* Status Indicators & Modifier Badges */}
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          {loadingLayout && (
            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Loading layout...</span>
          )}
          <span
            onClick={onToggleShift}
            style={{
              fontSize: "11px",
              padding: "2px 8px",
              borderRadius: "4px",
              backgroundColor: shiftActive ? "rgba(99, 102, 241, 0.25)" : "rgba(255, 255, 255, 0.05)",
              color: shiftActive ? "var(--accent-indigo)" : "var(--text-muted)",
              border: `1px solid ${shiftActive ? "rgba(99, 102, 241, 0.5)" : "rgba(255, 255, 255, 0.08)"}`,
              cursor: "pointer",
              fontWeight: "600",
              transition: "all 0.15s ease",
            }}
          >
            Shift: {shiftActive ? "ON" : "OFF"}
          </span>
          <span
            onClick={onToggleAltGr}
            style={{
              fontSize: "11px",
              padding: "2px 8px",
              borderRadius: "4px",
              backgroundColor: altGrActive ? "rgba(245, 158, 11, 0.25)" : "rgba(255, 255, 255, 0.05)",
              color: altGrActive ? "var(--accent-amber)" : "var(--text-muted)",
              border: `1px solid ${altGrActive ? "rgba(245, 158, 11, 0.5)" : "rgba(255, 255, 255, 0.08)"}`,
              cursor: "pointer",
              fontWeight: "600",
              transition: "all 0.15s ease",
            }}
          >
            AltGr: {altGrActive ? "ON" : "OFF"}
          </span>
        </div>
      </div>

      {/* Keyboard Grid */}
      <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
        <div style={{ display: "flex", gap: "6px" }}>{rowE.map(renderKey)}</div>
        <div style={{ display: "flex", gap: "6px" }}>{rowD.map(renderKey)}</div>
        <div style={{ display: "flex", gap: "6px" }}>{rowC.map(renderKey)}</div>
        <div style={{ display: "flex", gap: "6px" }}>{rowB.map(renderKey)}</div>
        <div style={{ display: "flex", gap: "6px" }}>{rowA.map(renderKey)}</div>
      </div>

      {/* Helper Legend */}
      <div style={{ display: "flex", justifyContent: "space-between", marginTop: "12px", fontSize: "11px", color: "var(--text-muted)" }}>
        <span>Click keycaps or type physical keys with live illumination</span>
        <span>Top-left: Shifted • Bottom-left: Normal • Bottom-right: AltGr</span>
      </div>
    </div>
  );
};
