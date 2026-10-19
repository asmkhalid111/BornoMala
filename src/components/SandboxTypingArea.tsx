import React, { useRef, useEffect, useState, useMemo } from "react";
import { EngineState, PhysicalKey, Modifiers, CandidateKind, CandidateOrientation } from "../types";

export interface ShapingSample {
  id: string;
  category: "Complex Conjuncts" | "Two-Part Vowels" | "Nukta Letters" | "Khanda-Ta & Joiners";
  label: string;
  word: string;
  breakdown: string;
  codepoints: string;
  note: string;
}

export const SHAPING_SAMPLES: ShapingSample[] = [
  {
    id: "kkha",
    category: "Complex Conjuncts",
    label: "ক্ষ (ক + ্ + ষ)",
    word: "ক্ষমা",
    breakdown: "ক + ্ + ষ + ম + া",
    codepoints: "U+0995 U+09CD U+09B7 U+09AE U+09BE",
    note: "Primary conjunct: shapes seamlessly without broken virama",
  },
  {
    id: "jnya",
    category: "Complex Conjuncts",
    label: "জ্ঞ (জ + ্ + ঞ)",
    word: "বিজ্ঞান",
    breakdown: "ব + ি + জ + ্ + ঞ + া + ন",
    codepoints: "U+09AC U+09BF U+099C U+09CD U+099E U+09BE U+09A8",
    note: "Primary conjunct: seamless OpenType glyph ligature",
  },
  {
    id: "skra",
    category: "Complex Conjuncts",
    label: "স্ক্র (স + ্ + ক + ্ + র)",
    word: "স্ক্রিপ্ট",
    breakdown: "স + ্ + ক + ্ + র + ি + প + ্ + ট",
    codepoints: "U+09B8 U+09CD U+0995 U+09CD U+09B0 U+09BF U+09AA U+09CD U+099F",
    note: "Triple conjunct: base + consonant + ra-phala subjoined form",
  },
  {
    id: "hma",
    category: "Complex Conjuncts",
    label: "হ্ম (হ + ্ + ম)",
    word: "ব্রহ্মপুত্র",
    breakdown: "ব + ্ + র + হ + ্ + ম + প + ু + ত + ্ + র",
    codepoints: "U+09AC U+09CD U+09B0 U+09B9 U+09CD U+09AE U+09AA U+09C1 U+09A4 U+09CD U+09B0",
    note: "Complex conjunct: vertical Ha-Ma ligature",
  },
  {
    id: "bristi",
    category: "Complex Conjuncts",
    label: "ষ্টি (ষ + ্ + ট + ি)",
    word: "বৃষ্টি",
    breakdown: "ব + ৃ + ষ + ্ + ট + ি",
    codepoints: "U+09AC U+09C3 U+09B7 U+09CD U+099F U+09BF",
    note: "Vocalic R vowel + Ssa-Ta conjunct + I-kar",
  },
  {
    id: "vowel_o",
    category: "Two-Part Vowels",
    label: "কো (ক + ো)",
    word: "কোনো",
    breakdown: "ক + ো + ন + ো",
    codepoints: "U+0995 U+09CB U+09A8 U+09CB",
    note: "NFC Composed: U+09C7 (ে) + U+09BE (া) → U+09CB (ো)",
  },
  {
    id: "vowel_ou",
    category: "Two-Part Vowels",
    label: "কৌ (ক + ৌ)",
    word: "কৌতুক",
    breakdown: "ক + ৌ + ত + ু + ক",
    codepoints: "U+0995 U+09CC U+09A4 U+09C1 U+0995",
    note: "NFC Composed: U+09C7 (ে) + U+09D7 (ৗ) → U+09CC (ৌ)",
  },
  {
    id: "nukta_rra",
    category: "Nukta Letters",
    label: "ড় (ড + ়)",
    word: "গাড়ি",
    breakdown: "গ + া + ড + ় + ি",
    codepoints: "U+0997 U+09BE U+09A1 U+09BC U+09BF",
    note: "UAX #15 Exclusion: decomposed canonical sequence",
  },
  {
    id: "nukta_rrha",
    category: "Nukta Letters",
    label: "ঢ় (ঢ + ়)",
    word: "আষাঢ়",
    breakdown: "আ + ষ + া + ঢ + ়",
    codepoints: "U+0986 U+09B7 U+09BE U+09A2 U+09BC",
    note: "UAX #15 Exclusion: decomposed canonical sequence",
  },
  {
    id: "nukta_yya",
    category: "Nukta Letters",
    label: "য় (য + ়)",
    word: "যায়",
    breakdown: "য + া + য + ়",
    codepoints: "U+09AF U+09BE U+09AF U+09BC",
    note: "UAX #15 Exclusion: decomposed canonical sequence",
  },
  {
    id: "khanda_ta",
    category: "Khanda-Ta & Joiners",
    label: "ৎ (U+09CE)",
    word: "উৎসব",
    breakdown: "উ + ৎ + স + ব",
    codepoints: "U+0989 U+09CE U+09B8 U+09AC",
    note: "Atomic Khanda-Ta code point preserved without decomposing",
  },
  {
    id: "zwnj_virama",
    category: "Khanda-Ta & Joiners",
    label: "ক্‌ক (ZWNJ)",
    word: "ক\u{09CD}\u{200C}ক",
    breakdown: "ক + ্ + ZWNJ + ক",
    codepoints: "U+0995 U+09CD U+200C U+0995",
    note: "ZWNJ inhibits ligature: visible halant preserved",
  },
  {
    id: "zwj_raphala",
    category: "Khanda-Ta & Joiners",
    label: "র‍্য (ZWJ)",
    word: "র\u{09CD}\u{200D}য",
    breakdown: "র + ্ + ZWJ + য",
    codepoints: "U+09B0 U+09CD U+200D U+09AF",
    note: "ZWJ forces ya-phala form instead of reph",
  },
];

interface SandboxTypingAreaProps {
  engineState: EngineState;
  committedHistory: string;
  candidateOrientation?: CandidateOrientation;
  candidateFontSize?: number;
  onSendChar: (c: string) => void;
  onBackspace: () => void;
  onDelete: () => void;
  onCommit: () => void;
  onCancel: () => void;
  onReset: () => void;
  onNextCandidate: () => void;
  onPrevCandidate: () => void;
  onSelectCandidate: (idx: number) => void;
  onMoveCursorLeft: () => void;
  onMoveCursorRight: () => void;
  onSetCursor: (pos: number) => void;
  onQuickInput: (text: string) => void;
  onAppendCommitted?: (text: string) => void;
  onSendKey?: (key: PhysicalKey, modifiers: Modifiers) => void;
}

const PHYSICAL_KEY_CODES: Record<string, PhysicalKey> = {
  Backquote: "Backquote",
  Digit1: "Digit1",
  Digit2: "Digit2",
  Digit3: "Digit3",
  Digit4: "Digit4",
  Digit5: "Digit5",
  Digit6: "Digit6",
  Digit7: "Digit7",
  Digit8: "Digit8",
  Digit9: "Digit9",
  Digit0: "Digit0",
  Minus: "Minus",
  Equal: "Equal",
  KeyQ: "KeyQ",
  KeyW: "KeyW",
  KeyE: "KeyE",
  KeyR: "KeyR",
  KeyT: "KeyT",
  KeyY: "KeyY",
  KeyU: "KeyU",
  KeyI: "KeyI",
  KeyO: "KeyO",
  KeyP: "KeyP",
  BracketLeft: "BracketLeft",
  BracketRight: "BracketRight",
  Backslash: "Backslash",
  KeyA: "KeyA",
  KeyS: "KeyS",
  KeyD: "KeyD",
  KeyF: "KeyF",
  KeyG: "KeyG",
  KeyH: "KeyH",
  KeyJ: "KeyJ",
  KeyK: "KeyK",
  KeyL: "KeyL",
  Semicolon: "Semicolon",
  Quote: "Quote",
  KeyZ: "KeyZ",
  KeyX: "KeyX",
  KeyC: "KeyC",
  KeyV: "KeyV",
  KeyB: "KeyB",
  KeyN: "KeyN",
  KeyM: "KeyM",
  Comma: "Comma",
  Period: "Period",
  Slash: "Slash",
  Space: "Space",
};

const renderCandidateBadge = (kind?: CandidateKind) => {
  if (!kind || kind === "PrimaryTransliteration") return null;
  switch (kind) {
    case "ExactLexiconMatch":
      return (
        <span
          style={{
            fontSize: "10px",
            fontWeight: "700",
            padding: "1px 6px",
            borderRadius: "4px",
            backgroundColor: "rgba(16, 185, 129, 0.2)",
            color: "var(--accent-emerald)",
            border: "1px solid rgba(16, 185, 129, 0.4)",
            letterSpacing: "0.02em",
            whiteSpace: "nowrap",
          }}
        >
          ✓ Word
        </span>
      );
    case "PrefixCompletion":
      return (
        <span
          style={{
            fontSize: "10px",
            fontWeight: "700",
            padding: "1px 6px",
            borderRadius: "4px",
            backgroundColor: "rgba(99, 102, 241, 0.2)",
            color: "var(--accent-indigo)",
            border: "1px solid rgba(99, 102, 241, 0.4)",
            letterSpacing: "0.02em",
            whiteSpace: "nowrap",
          }}
        >
          📖 Completion
        </span>
      );
    case "SpellingCorrection":
      return (
        <span
          style={{
            fontSize: "10px",
            fontWeight: "700",
            padding: "1px 6px",
            borderRadius: "4px",
            backgroundColor: "rgba(245, 158, 11, 0.2)",
            color: "var(--accent-amber)",
            border: "1px solid rgba(245, 158, 11, 0.4)",
            letterSpacing: "0.02em",
            whiteSpace: "nowrap",
          }}
        >
          ✏️ Correction
        </span>
      );
    case "PhoneticAlternative":
      return (
        <span
          style={{
            fontSize: "10px",
            fontWeight: "700",
            padding: "1px 6px",
            borderRadius: "4px",
            backgroundColor: "rgba(14, 165, 233, 0.15)",
            color: "var(--accent-cyan)",
            border: "1px solid rgba(14, 165, 233, 0.35)",
            letterSpacing: "0.02em",
            whiteSpace: "nowrap",
          }}
        >
          ALT
        </span>
      );
  }
};

export const SandboxTypingArea: React.FC<SandboxTypingAreaProps> = ({
  engineState,
  committedHistory,
  candidateOrientation = "horizontal",
  candidateFontSize = 14,
  onSendChar,
  onBackspace,
  onDelete,
  onCommit,
  onCancel,
  onReset,
  onNextCandidate,
  onPrevCandidate,
  onSelectCandidate,
  onMoveCursorLeft,
  onMoveCursorRight,
  onSetCursor,
  onQuickInput,
  onAppendCommitted,
  onSendKey,
}) => {
  const inputRef = useRef<HTMLInputElement>(null);
  const [activeCategory, setActiveCategory] = useState<string>("All");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [showPhoneticGuide, setShowPhoneticGuide] = useState<boolean>(false);
  const [guideTab, setGuideTab] = useState<"vowels" | "consonants" | "conjuncts" | "phalas" | "candidates">("vowels");

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    // 1-9 direct selection when candidates are available during composition
    if (
      engineState.is_composing &&
      engineState.candidates.length > 0 &&
      /^[1-9]$/.test(e.key)
    ) {
      const idx = parseInt(e.key, 10) - 1;
      if (idx < engineState.candidates.length) {
        e.preventDefault();
        onSelectCandidate(idx);
        return;
      }
    }

    if (e.key === "Tab") {
      if (engineState.is_composing && engineState.candidates.length > 0) {
        e.preventDefault();
        if (e.shiftKey) {
          onPrevCandidate();
        } else {
          onNextCandidate();
        }
        return;
      }
    } else if (e.key === "ArrowDown") {
      if (engineState.is_composing && engineState.candidates.length > 0) {
        e.preventDefault();
        onNextCandidate();
        return;
      }
    } else if (e.key === "ArrowUp") {
      if (engineState.is_composing && engineState.candidates.length > 0) {
        e.preventDefault();
        onPrevCandidate();
        return;
      }
    } else if (e.key === "ArrowLeft") {
      if (engineState.is_composing) {
        e.preventDefault();
        onMoveCursorLeft();
        return;
      }
    } else if (e.key === "ArrowRight") {
      if (engineState.is_composing) {
        e.preventDefault();
        onMoveCursorRight();
        return;
      }
    } else if (e.key === "Backspace") {
      e.preventDefault();
      onBackspace();
      return;
    } else if (e.key === "Delete") {
      e.preventDefault();
      onDelete();
      return;
    } else if (e.key === "Enter") {
      e.preventDefault();
      onCommit();
      return;
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
      return;
    }

    const physicalKey = PHYSICAL_KEY_CODES[e.code];
    const isFixed = typeof engineState.mode === "object" && "Fixed" in engineState.mode;

    if (isFixed && physicalKey && onSendKey) {
      e.preventDefault();
      const altgr = e.getModifierState("AltGraph") || (e.ctrlKey && e.altKey);
      onSendKey(physicalKey, {
        shift: e.shiftKey,
        altgr,
        ctrl: e.ctrlKey && !altgr,
        alt: e.altKey && !altgr,
      });
      return;
    }

    if (e.key === " ") {
      e.preventDefault();
      onSendChar(" ");
      return;
    } else if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
      e.preventDefault();
      onSendChar(e.key);
      return;
    }
  };

  const sampleChips = useMemo(() => {
    if (typeof engineState.mode === "object" && "Fixed" in engineState.mode) {
      if (engineState.mode.Fixed === "National") {
        return [
          { label: "j + g + j → ক্ক (Double Consonant)", val: "jgj" },
          { label: "j + g + Shift+X → ক্ষ (Conjunct)", val: "jgX" },
          { label: "u + g + Shift+I → জ্ঞ (jnya)", val: "ugI" },
          { label: "g + f → আ (Link Vowel)", val: "gf" },
          { label: "h + g + d → বই (Medial Vowel)", val: "hgd" },
          { label: "h + d → বি (Direct Kar)", val: "hd" },
          { label: "v + g + j → র্ক (Reph)", val: "vgj" },
          { label: "AltGr+4 → $", val: "$" },
          { label: "AltGr+Equal → ॥", val: "॥" },
        ];
      }
      if (engineState.mode.Fixed === "Probhat") {
        return [
          { label: "k + / + Shift+S → ক্ষ (Conjunct)", val: "k/S" },
          { label: "j + / + Shift+7 → জ্ঞ (jnya)", val: "j/&" },
          { label: "Shift+A → অ (Direct Vowel)", val: "A" },
          { label: "v → আ (Direct Vowel)", val: "v" },
          { label: "k + / + a → কা (Kar Replacement)", val: "k/a" },
          { label: "r + / + k → র্ক (Reph)", val: "r/k" },
          { label: "\\ → ZWNJ", val: "\u{200C}" },
          { label: "~ → ZWJ", val: "\u{200D}" },
        ];
      }
      if (engineState.mode.Fixed === "Bijoy") {
        return [
          { label: "j + g + Shift+N → ক্ষ (Conjunct)", val: "jgN" },
          { label: "g + f → আ (Link Vowel)", val: "gf" },
          { label: "h + g + d → বই (Medial Vowel)", val: "hgd" },
          { label: "; → ম / ণ (Semicolon)", val: ";" },
          { label: "' → ন / ষ (Quote)", val: "'" },
          { label: ", → , / ৎ (Comma)", val: "," },
          { label: "? → ? / ঃ (Slash)", val: "?" },
        ];
      }
    }
    if (engineState.mode === "EnglishPassthrough") {
      return [
        { label: "Hello World!", val: "Hello World!" },
        { label: "Borno Input Platform", val: "Borno Input Platform" },
        { label: "Deterministic Passthrough", val: "Deterministic Passthrough" },
      ];
    }
    // Phonetic defaults
    return [
      { label: ".. → ।", val: ".." },
      { label: ".... → ॥", val: "...." },
      { label: "tomake → তোমাকে (Slot 1)", val: "tomake" },
      { label: "bornomala → বর্ণমালা (Slot 1)", val: "bornomala" },
      { label: "k → ক", val: "k" },
      { label: "ka → কা (3 cands)", val: "ka" },
      { label: "ami → আমি (2 cands)", val: "ami" },
      { label: "bangla → বাংলা", val: "bangla" },
      { label: "xoma → ক্ষমা", val: "xoma" },
      { label: "bijNGan → বিজ্ঞান", val: "bijNGan" },
      { label: "brriSTi → বৃষ্টি", val: "brriSTi" },
      { label: "shreshTho → শ্রেষ্ঠ", val: "shreshTho" },
      { label: "c^ad → চাঁদ", val: "c^ad" },
      { label: "ut`sob → উৎসব", val: "ut`sob" },
      { label: "swadheen → স্বাধীন", val: "swadheen" },
      { label: "kOnO → কোনো", val: "kOnO" },
      { label: "bakyo → বাক্য", val: "bakyo" },
    ];
  }, [engineState.mode]);

  const preeditChars = Array.from(engineState.preedit);

  return (
    <div className="glass-panel" style={{ padding: "24px", marginBottom: "20px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
        <h2 style={{ fontSize: "16px", fontWeight: "600", display: "flex", alignItems: "center", gap: "8px" }}>
          <span>Interactive Typing Sandbox</span>
          <span
            style={{
              fontSize: "11px",
              padding: "2px 8px",
              borderRadius: "4px",
              backgroundColor: "rgba(99, 102, 241, 0.15)",
              color: "var(--accent-indigo)",
              fontWeight: "500",
            }}
          >
            Authoritative Rust Core IPC
          </span>
          {engineState.mode === "EnglishPassthrough" && (
            <span
              style={{
                fontSize: "11px",
                padding: "2px 8px",
                borderRadius: "4px",
                backgroundColor: "rgba(245, 158, 11, 0.15)",
                color: "var(--accent-amber)",
                fontWeight: "500",
              }}
            >
              Passthrough Active
            </span>
          )}
        </h2>
        <div style={{ display: "flex", gap: "8px" }}>
          <button
            onClick={onCancel}
            disabled={!engineState.is_composing}
            style={{
              padding: "6px 14px",
              backgroundColor: engineState.is_composing ? "rgba(245, 158, 11, 0.15)" : "rgba(255, 255, 255, 0.05)",
              color: engineState.is_composing ? "var(--accent-amber)" : "var(--text-muted)",
              border: `1px solid ${engineState.is_composing ? "rgba(245, 158, 11, 0.3)" : "var(--border-subtle)"}`,
              borderRadius: "6px",
              cursor: engineState.is_composing ? "pointer" : "not-allowed",
              fontSize: "12px",
              fontWeight: "500",
              transition: "all 0.2s ease",
            }}
            title="Cancel active composition (Esc)"
          >
            Cancel (Esc)
          </button>
          <button
            onClick={onReset}
            style={{
              padding: "6px 14px",
              backgroundColor: "rgba(244, 63, 94, 0.12)",
              color: "var(--accent-rose)",
              border: "1px solid rgba(244, 63, 94, 0.25)",
              borderRadius: "6px",
              cursor: "pointer",
              fontSize: "12px",
              fontWeight: "500",
              transition: "all 0.2s ease",
            }}
            title="Reset Engine State"
          >
            Reset Engine
          </button>
        </div>
      </div>

      {/* Main interactive keystroke input */}
      <div style={{ marginBottom: "20px" }}>
        <label
          htmlFor="sandbox-keystroke-input"
          style={{
            display: "block",
            fontSize: "12px",
            color: "var(--text-secondary)",
            marginBottom: "8px",
          }}
        >
          Type phonetically below (keystrokes dispatch normalized events directly into Borno Core):
        </label>
        <div style={{ position: "relative" }}>
          <input
            id="sandbox-keystroke-input"
            ref={inputRef}
            type="text"
            onKeyDown={handleKeyDown}
            placeholder="Click here and type phonetically (e.g. 'ka', 'ami', 'kor')..."
            style={{
              width: "100%",
              padding: "14px 16px",
              fontSize: "15px",
              backgroundColor: "var(--bg-secondary)",
              border: "1px solid var(--border-subtle)",
              borderRadius: "8px",
              color: "var(--text-primary)",
              outline: "none",
              fontFamily: "var(--font-mono)",
              boxShadow: "inset 0 2px 4px rgba(0, 0, 0, 0.3)",
              transition: "border-color 0.2s, box-shadow 0.2s",
            }}
            onFocus={(e) => {
              e.currentTarget.style.borderColor = "var(--border-focus)";
              e.currentTarget.style.boxShadow = "var(--glow-cyan)";
            }}
            onBlur={(e) => {
              e.currentTarget.style.borderColor = "var(--border-subtle)";
              e.currentTarget.style.boxShadow = "inset 0 2px 4px rgba(0, 0, 0, 0.3)";
            }}
            value={engineState.raw_buffer}
            onChange={() => {}} // Controlled by onKeyDown to simulate native IME keystroke capture
          />
        </div>
      </div>

      {/* Real-time Preedit and Candidate Display */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: "16px",
          marginBottom: "20px",
        }}
      >
        {/* Preedit / Active Composition */}
        <div
          style={{
            padding: "16px",
            borderRadius: "8px",
            backgroundColor: "rgba(11, 15, 25, 0.6)",
            border: "1px solid var(--border-subtle)",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "8px" }}>
            <span style={{ fontSize: "12px", color: "var(--text-muted)", fontWeight: "500" }}>
              Active Preedit & In-Composition Cursor
            </span>
            <span style={{ fontSize: "11px", color: "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}>
              Cursor: {engineState.cursor_pos} / {preeditChars.length} codepoints
            </span>
          </div>
          <div
            style={{
              minHeight: "52px",
              display: "flex",
              alignItems: "center",
              fontSize: "28px",
              fontFamily: "var(--font-bangla)",
              color: engineState.preedit ? "var(--accent-cyan)" : "var(--text-muted)",
              borderBottom: engineState.preedit ? "2px solid var(--accent-cyan)" : "2px dashed rgba(255, 255, 255, 0.1)",
              paddingBottom: "4px",
              userSelect: "none",
            }}
          >
            {preeditChars.length > 0 ? (
              <>
                <span onClick={() => onSetCursor(0)} style={{ cursor: "pointer" }} title="Click to move cursor to start">
                  {preeditChars.slice(0, engineState.cursor_pos).join("")}
                </span>
                <span
                  style={{
                    width: "2px",
                    height: "32px",
                    backgroundColor: "#38bdf8",
                    display: "inline-block",
                    margin: "0 2px",
                    verticalAlign: "middle",
                    boxShadow: "0 0 10px #38bdf8",
                    animation: "cursor-blink 1s step-start infinite",
                  }}
                />
                <span onClick={() => onSetCursor(preeditChars.length)} style={{ cursor: "pointer" }} title="Click to move cursor to end">
                  {preeditChars.slice(engineState.cursor_pos).join("")}
                </span>
              </>
            ) : (
              <span style={{ fontSize: "16px", color: "var(--text-muted)", fontStyle: "italic" }}>
                {engineState.mode === "EnglishPassthrough" ? "Passthrough mode active (direct typing)" : "Idle (type to compose)"}
              </span>
            )}
          </div>
        </div>

        {/* Candidates Panel */}
        <div
          style={{
            padding: "16px",
            borderRadius: "8px",
            backgroundColor: "rgba(11, 15, 25, 0.6)",
            border: "1px solid var(--border-subtle)",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
            <span style={{ fontSize: "12px", color: "var(--text-muted)", fontWeight: "500" }}>
              Engine Candidates ({engineState.candidates.length})
            </span>
            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
              (1-9 / click: commit, Tab / ↓↑: highlight)
            </span>
          </div>
          <div
            style={{
              display: "flex",
              flexDirection: candidateOrientation === "vertical" ? "column" : "row",
              flexWrap: candidateOrientation === "vertical" ? "nowrap" : "wrap",
              gap: "8px",
              minHeight: "52px",
              alignItems: candidateOrientation === "vertical" ? "stretch" : "center",
              width: "100%",
            }}
          >
            {engineState.candidates.length > 0 ? (
              engineState.candidates.map((cand, idx) => {
                const isSelected = engineState.selected_candidate_index === idx;
                const detail = engineState.candidate_details?.[idx];
                const badge = renderCandidateBadge(detail?.kind);
                return (
                  <button
                    key={idx}
                    onClick={() => onSelectCandidate(idx)}
                    style={{
                      padding: candidateOrientation === "vertical" ? "8px 14px" : "6px 14px",
                      borderRadius: "6px",
                      backgroundColor: isSelected ? "rgba(14, 165, 233, 0.35)" : "rgba(14, 165, 233, 0.12)",
                      border: `1px solid ${isSelected ? "var(--accent-cyan)" : "rgba(14, 165, 233, 0.3)"}`,
                      boxShadow: isSelected ? "0 0 12px rgba(14, 165, 233, 0.5)" : "none",
                      color: "#ffffff",
                      fontSize: `${candidateFontSize}pt`,
                      fontFamily: "var(--font-bangla)",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: candidateOrientation === "vertical" ? "space-between" : "flex-start",
                      gap: "8px",
                      cursor: "pointer",
                      transition: "all 0.15s ease",
                      outline: "none",
                      width: candidateOrientation === "vertical" ? "100%" : "auto",
                    }}
                    title={`Click or press '${idx + 1}' to commit`}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <span style={{ fontSize: "11px", color: isSelected ? "#ffffff" : "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}>
                        {idx + 1}.
                      </span>
                      <span>{cand}</span>
                    </div>
                    {badge}
                  </button>
                );
              })
            ) : (
              <span style={{ fontSize: "13px", color: "var(--text-muted)", fontStyle: "italic" }}>
                No active candidate list
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Committed Text Area */}
      <div style={{ marginBottom: "16px" }}>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "6px" }}>
          <span style={{ fontSize: "12px", color: "var(--text-secondary)", fontWeight: "500" }}>
            Committed Output Text Buffer:
          </span>
          <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
            (Space, Enter, or selecting a candidate commits text)
          </span>
        </div>
        <div
          style={{
            minHeight: "72px",
            padding: "12px 16px",
            backgroundColor: "var(--bg-secondary)",
            border: "1px solid var(--border-subtle)",
            borderRadius: "8px",
            fontSize: "20px",
            fontFamily: "var(--font-bangla)",
            color: "var(--text-primary)",
            whiteSpace: "pre-wrap",
            wordBreak: "break-word",
          }}
        >
          {committedHistory ? committedHistory : <span style={{ color: "var(--text-muted)", fontSize: "14px", fontStyle: "italic" }}>Committed words will appear here...</span>}
        </div>
      </div>

      {/* Deterministic Quick-Test Chips */}
      <div style={{ display: "flex", alignItems: "center", gap: "10px", flexWrap: "wrap" }}>
        <span style={{ fontSize: "12px", color: "var(--text-muted)" }}>Quick test tokens:</span>
        {sampleChips.map((chip) => (
          <button
            key={chip.val}
            onClick={() => onQuickInput(chip.val)}
            style={{
              padding: "4px 10px",
              borderRadius: "6px",
              backgroundColor: "rgba(255, 255, 255, 0.05)",
              border: "1px solid var(--border-subtle)",
              color: "var(--text-secondary)",
              fontSize: "12px",
              cursor: "pointer",
              transition: "all 0.15s ease",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.1)";
              e.currentTarget.style.color = "var(--text-primary)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.05)";
              e.currentTarget.style.color = "var(--text-secondary)";
            }}
          >
            {chip.label}
          </button>
        ))}
      </div>

      {/* Interactive Collapsible Avro Phonetic Keystroke Guide */}
      <div
        style={{
          marginTop: "20px",
          padding: "16px",
          borderRadius: "10px",
          backgroundColor: "rgba(15, 23, 42, 0.6)",
          border: "1px solid rgba(14, 165, 233, 0.25)",
        }}
      >
        <div
          onClick={() => setShowPhoneticGuide(!showPhoneticGuide)}
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            cursor: "pointer",
            userSelect: "none",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "16px" }}>⌨️</span>
            <div>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <h3 style={{ fontSize: "14px", fontWeight: "600", color: "#f8fafc" }}>
                  Avro Phonetic Keystroke Reference Guide
                </h3>
                <span
                  style={{
                    fontSize: "10px",
                    padding: "2px 6px",
                    borderRadius: "4px",
                    backgroundColor: "rgba(14, 165, 233, 0.2)",
                    color: "var(--accent-cyan)",
                    fontWeight: "600",
                  }}
                >
                  Avro-Compatible
                </span>
              </div>
              <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                Grounded rules: vowels & kars, consonants, 30 curated conjuncts, phalas, modifiers, and candidates
              </p>
            </div>
          </div>
          <button
            style={{
              padding: "4px 10px",
              fontSize: "11px",
              fontWeight: "500",
              borderRadius: "6px",
              backgroundColor: "rgba(255, 255, 255, 0.06)",
              color: "var(--text-secondary)",
              border: "1px solid var(--border-subtle)",
              cursor: "pointer",
            }}
          >
            {showPhoneticGuide ? "▲ Hide Guide" : "▼ Show Guide"}
          </button>
        </div>

        {showPhoneticGuide && (
          <div style={{ marginTop: "16px", paddingTop: "14px", borderTop: "1px solid rgba(255, 255, 255, 0.08)" }}>
            {/* Guide Tabs */}
            <div style={{ display: "flex", gap: "6px", flexWrap: "wrap", marginBottom: "14px" }}>
              {[
                { id: "vowels", label: "Vowels & Kars" },
                { id: "consonants", label: "Consonants & Distinctions" },
                { id: "conjuncts", label: "Curated Conjunct Matrix (30)" },
                { id: "phalas", label: "Phalas, Modifiers & Escapes" },
                { id: "candidates", label: "Candidate Generation Rules" },
              ].map((t) => (
                <button
                  key={t.id}
                  onClick={() => setGuideTab(t.id as any)}
                  style={{
                    padding: "5px 12px",
                    borderRadius: "6px",
                    fontSize: "11px",
                    fontWeight: guideTab === t.id ? "600" : "400",
                    backgroundColor: guideTab === t.id ? "rgba(14, 165, 233, 0.2)" : "rgba(255, 255, 255, 0.03)",
                    color: guideTab === t.id ? "var(--accent-cyan)" : "var(--text-muted)",
                    border: `1px solid ${guideTab === t.id ? "rgba(14, 165, 233, 0.4)" : "var(--border-subtle)"}`,
                    cursor: "pointer",
                    transition: "all 0.15s ease",
                  }}
                >
                  {t.label}
                </button>
              ))}
            </div>

            {/* Tab 1: Vowels */}
            {guideTab === "vowels" && (
              <div style={{ overflowX: "auto" }}>
                <table style={{ width: "100%", fontSize: "12px", borderCollapse: "collapse", textAlign: "left" }}>
                  <thead>
                    <tr style={{ borderBottom: "1px solid rgba(255,255,255,0.1)", color: "var(--text-muted)" }}>
                      <th style={{ padding: "6px 8px" }}>Key</th>
                      <th style={{ padding: "6px 8px" }}>Initial / Independent</th>
                      <th style={{ padding: "6px 8px" }}>Post-Consonant (Kar)</th>
                      <th style={{ padding: "6px 8px" }}>Rule & Example</th>
                      <th style={{ padding: "6px 8px" }}>Quick Try</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[
                      { key: "o", ind: "অ", dep: "(Inherent bridge)", ex: "omuk → অমুক, kor → কর (no kar emitted)", test: "kor" },
                      { key: "a / A / aa", ind: "আ", dep: "া (U+09BE)", ex: "Ami → আমি, ka → কা, maa → মা", test: "Ami" },
                      { key: "i", ind: "ই", dep: "ি (U+09BF)", ex: "iti → ইতি, ki → কি", test: "iti" },
                      { key: "I / ee", ind: "ঈ", dep: "ী (U+09C0)", ex: "Id → ঈদ, teer → তীর", test: "teer" },
                      { key: "u", ind: "উ", dep: "ু (U+09C1)", ex: "upor → উপর, tumi → তুমি", test: "tumi" },
                      { key: "U / oo", ind: "ঊ", dep: "ূ (U+09C2)", ex: "Usha → ঊষা, dUr → দূর", test: "dUr" },
                      { key: "e", ind: "এ", dep: "ে (U+09C7)", ex: "ek → এক, ke → কে", test: "ek" },
                      { key: "E / ae", ind: "অ্যা", dep: "্যা (U+09CD U+09AF U+09BE)", ex: "Eka → অ্যাকা, baek → ব্যাক", test: "baek" },
                      { key: "O", ind: "ও", dep: "ো (U+09CB)", ex: "Capital O forms O-kar: kOnO → কোনো", test: "kOnO" },
                      { key: "oi / OI", ind: "ঐ", dep: "ৈ (U+09C8)", ex: "oikyo → ঐক্য, toiri → তৈরি", test: "toiri" },
                      { key: "ou / OU", ind: "ঔ", dep: "ৌ (U+09CC)", ex: "oushokh → ঔষধ, koutuk → কৌতুক", test: "koutuk" },
                      { key: "rri", ind: "ঋ", dep: "ৃ (U+09C3)", ex: "rritu → ঋতু, krripa → কৃপা, brriSTi → বৃষ্টি", test: "brriSTi" },
                    ].map((row) => (
                      <tr key={row.key} style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }}>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-mono)", color: "var(--accent-amber)" }}>{row.key}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-bangla)", fontSize: "15px" }}>{row.ind}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-bangla)", fontSize: "15px", color: "var(--accent-cyan)" }}>{row.dep}</td>
                        <td style={{ padding: "6px 8px", color: "var(--text-secondary)" }}>{row.ex}</td>
                        <td style={{ padding: "6px 8px" }}>
                          <button
                            onClick={() => onQuickInput(row.test)}
                            style={{
                              padding: "2px 8px",
                              fontSize: "10px",
                              borderRadius: "4px",
                              backgroundColor: "rgba(14, 165, 233, 0.15)",
                              color: "var(--accent-cyan)",
                              border: "1px solid rgba(14, 165, 233, 0.3)",
                              cursor: "pointer",
                            }}
                          >
                            Type {row.test}
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {/* Tab 2: Consonants */}
            {guideTab === "consonants" && (
              <div style={{ overflowX: "auto" }}>
                <table style={{ width: "100%", fontSize: "12px", borderCollapse: "collapse", textAlign: "left" }}>
                  <thead>
                    <tr style={{ borderBottom: "1px solid rgba(255,255,255,0.1)", color: "var(--text-muted)" }}>
                      <th style={{ padding: "6px 8px" }}>Category</th>
                      <th style={{ padding: "6px 8px" }}>Key</th>
                      <th style={{ padding: "6px 8px" }}>Letter</th>
                      <th style={{ padding: "6px 8px" }}>Contrast / Note</th>
                      <th style={{ padding: "6px 8px" }}>Example</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[
                      { cat: "Retroflex vs Dental", key: "T vs t", l: "ট vs ত", note: "Capital T = retroflex ট; Lowercase t = dental ত", ex: "Taka (টাকা) vs tumi (তুমি)" },
                      { cat: "Retroflex vs Dental", key: "Th vs th", l: "ঠ vs থ", note: "Capital Th = retroflex ঠ; Lowercase th = dental থ", ex: "Thakur (ঠাকুর) vs thama (থামা)" },
                      { cat: "Retroflex vs Dental", key: "D vs d", l: "ড vs দ", note: "Capital D = retroflex ড; Lowercase d = dental দ", ex: "Daak (ডাক) vs dokan (দোকান)" },
                      { cat: "Retroflex vs Dental", key: "Dh vs dh", l: "ঢ vs ধ", note: "Capital Dh = retroflex ঢ; Lowercase dh = dental ধ", ex: "Dhaka (ঢাকা) vs dhon (ধন)" },
                      { cat: "Retroflex vs Dental", key: "N vs n", l: "ণ vs ন", note: "Capital N = retroflex ণ; Lowercase n = dental ন", ex: "biNa (বীণা) vs naam (নাম)" },
                      { cat: "Sibilants", key: "sh / S", l: "শ", note: "Palatal sibilant (lowercase sh or capital S)", ex: "shanti (শান্তি), desh (দেশ)" },
                      { cat: "Sibilants", key: "Sh", l: "ষ", note: "Retroflex sibilant (capital S with h)", ex: "bhaSha (ভাষা), koshTo (কষ্ট)" },
                      { cat: "Sibilants", key: "s", l: "স", note: "Dental sibilant (lowercase s)", ex: "sob (সব), sundor (সুন্দর)" },
                      { cat: "Flaps & Nukta", key: "R vs Rh vs r", l: "ড় / ঢ় / র", note: "R = ড় (ড+়), Rh = ঢ় (ঢ+়), r = alveolar flap র", ex: "gaRi (গাড়ি), ashaRh (আষাঢ়), raat (রাত)" },
                      { cat: "Semivowels", key: "y / Y vs z", l: "য় vs য", note: "y/Y = Antastha Ya (য+়); z = Antastha Za", ex: "doya (দয়া) vs zodi (যদি)" },
                      { cat: "Nasals", key: "Ng vs ng vs NG", l: "ঙ vs ং vs ঞ", note: "Ng = ঙ, ng = ং (Anusvara), NG = ঞ (Palatal nasal)", ex: "baNga (বাঙা), bangla (বাংলা), miNGa (মিঞা)" },
                    ].map((row, i) => (
                      <tr key={i} style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }}>
                        <td style={{ padding: "6px 8px", color: "var(--accent-indigo)" }}>{row.cat}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-mono)", color: "var(--accent-amber)" }}>{row.key}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-bangla)", fontSize: "15px", color: "#ffffff" }}>{row.l}</td>
                        <td style={{ padding: "6px 8px", color: "var(--text-secondary)" }}>{row.note}</td>
                        <td style={{ padding: "6px 8px", color: "var(--text-muted)" }}>{row.ex}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {/* Tab 3: Conjuncts */}
            {guideTab === "conjuncts" && (
              <div style={{ overflowX: "auto" }}>
                <table style={{ width: "100%", fontSize: "12px", borderCollapse: "collapse", textAlign: "left" }}>
                  <thead>
                    <tr style={{ borderBottom: "1px solid rgba(255,255,255,0.1)", color: "var(--text-muted)" }}>
                      <th style={{ padding: "6px 8px" }}>Key</th>
                      <th style={{ padding: "6px 8px" }}>Conjunct</th>
                      <th style={{ padding: "6px 8px" }}>Breakdown</th>
                      <th style={{ padding: "6px 8px" }}>Example</th>
                      <th style={{ padding: "6px 8px" }}>Try</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[
                      { key: "kSh / x", c: "ক্ষ", b: "ক + ্ + ষ", ex: "xoma / kShoma → ক্ষমা", test: "xoma" },
                      { key: "kShm / xm", c: "ক্ষ্ম", b: "ক + ্ + ষ + ্ + ম", ex: "sukShmo → সূক্ষ্ম", test: "sukShmo" },
                      { key: "kShN / xN", c: "ক্ষ্ণ", b: "ক + ্ + ষ + ্ + ণ", ex: "teekShNo → তীক্ষ্ণ", test: "teekShNo" },
                      { key: "jNG / jgy", c: "জ্ঞ", b: "জ + ্ + ঞ", ex: "jNGan → জ্ঞান, bijNGan → বিজ্ঞান", test: "bijNGan" },
                      { key: "cch", c: "চ্ছ", b: "চ + ্ + ছ", ex: "iccha → ইচ্ছা", test: "iccha" },
                      { key: "NGc / Nc", c: "ঞ্চ", b: "ঞ + ্ + চ", ex: "oncol → অঞ্চল, poNGco → পঞ্চ", test: "oncol" },
                      { key: "NGch / Nch", c: "ঞ্ছ", b: "ঞ + ্ + ছ", ex: "baNGcha → বাঞ্ছা", test: "baNGcha" },
                      { key: "NGj / nj", c: "ঞ্জ", b: "ঞ + ্ + জ", ex: "gonj → গঞ্জ, guNGjon → গুঞ্জন", test: "gonj" },
                      { key: "Ngk", c: "ঙ্ক", b: "ঙ + ্ + ক", ex: "oNgko → অঙ্ক", test: "oNgko" },
                      { key: "sT", c: "স্ট", b: "স + ্ + ট (Loanword)", ex: "sTeshon → স্টেশন", test: "sTeshon" },
                      { key: "ShT / ST / shT", c: "ষ্ট", b: "ষ + ্ + ট (Tatsama)", ex: "brriSTi → বৃষ্টি, koshTo → কষ্ট", test: "brriSTi" },
                      { key: "ShTh / shTh / STh", c: "ষ্ঠ", b: "ষ + ্ + ঠ", ex: "shreshTho → শ্রেষ্ঠ", test: "shreshTho" },
                      { key: "st", c: "স্ত", b: "স + ্ + ত", ex: "rasta → রাস্তা, prostab → প্রস্তাব", test: "rasta" },
                      { key: "skr", c: "স্ক্র", b: "স + ্ + ক + ্ + র", ex: "skript → স্ক্রিপ্ট", test: "skript" },
                      { key: "str", c: "স্ত্র", b: "স + ্ + ত + ্ + র", ex: "stri → স্ত্রী", test: "stri" },
                      { key: "hm", c: "হ্ম", b: "হ + ্ + ম", ex: "brohmo → ব্রহ্ম", test: "brohmo" },
                      { key: "hn", c: "হ্ন", b: "হ + ্ + ন", ex: "chinho → চিহ্ন", test: "chinho" },
                      { key: "gg / nn / kk / tt", c: "গ্গ / ন্ন / ক্ক / ত্ত", b: "Geminates", ex: "biggan → বিগ্গান, banno → বান্ন", test: "biggan" },
                    ].map((row) => (
                      <tr key={row.key} style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }}>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-mono)", color: "var(--accent-amber)" }}>{row.key}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-bangla)", fontSize: "16px", color: "var(--accent-cyan)" }}>{row.c}</td>
                        <td style={{ padding: "6px 8px", color: "var(--text-secondary)", fontFamily: "var(--font-bangla)" }}>{row.b}</td>
                        <td style={{ padding: "6px 8px", color: "var(--text-muted)" }}>{row.ex}</td>
                        <td style={{ padding: "6px 8px" }}>
                          <button
                            onClick={() => onQuickInput(row.test)}
                            style={{
                              padding: "2px 8px",
                              fontSize: "10px",
                              borderRadius: "4px",
                              backgroundColor: "rgba(14, 165, 233, 0.15)",
                              color: "var(--accent-cyan)",
                              border: "1px solid rgba(14, 165, 233, 0.3)",
                              cursor: "pointer",
                            }}
                          >
                            {row.test}
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {/* Tab 4: Phalas & Modifiers */}
            {guideTab === "phalas" && (
              <div style={{ overflowX: "auto" }}>
                <table style={{ width: "100%", fontSize: "12px", borderCollapse: "collapse", textAlign: "left" }}>
                  <thead>
                    <tr style={{ borderBottom: "1px solid rgba(255,255,255,0.1)", color: "var(--text-muted)" }}>
                      <th style={{ padding: "6px 8px" }}>Feature</th>
                      <th style={{ padding: "6px 8px" }}>Trigger</th>
                      <th style={{ padding: "6px 8px" }}>Bangla</th>
                      <th style={{ padding: "6px 8px" }}>Rule & Examples</th>
                      <th style={{ padding: "6px 8px" }}>Try</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[
                      { f: "Reph (রেফ)", t: "r preceding consonant", b: "র্", ex: "rk → র্ক, rb → র্ব, borNo → বর্ণ", test: "borNo" },
                      { f: "Ra-phala (র-ফলা)", t: "consonant + r", b: "্র", ex: "kr → ক্র, gr → গ্র, pr → প্র, br → ব্র", test: "prostab" },
                      { f: "Ya-phala (য-ফলা)", t: "consonant + y / Z", b: "্য", ex: "ky / kZ → ক্য, by → ব্য, bakyo → বাক্য", test: "bakyo" },
                      { f: "Ba-phala (ব-ফলা)", t: "consonant + w / v", b: "্ব", ex: "sw → স্ব, swadheen → স্বাধীন, biswo → বিশ্ব", test: "swadheen" },
                      { f: "Candrabindu", t: "^ (caret)", b: "ঁ", ex: "c^ad → চাঁদ, b^aTi → বাঁটি", test: "c^ad" },
                      { f: "Khanda-Ta", t: "t` or \\t", b: "ৎ", ex: "ut`sob → উৎসব, hoThat` → হঠাৎ, u\\tsob → উৎসব", test: "ut`sob" },
                      { f: "Explicit Virama", t: ",, or ``", b: "্", ex: "k,, → ক্, k`` → ক্", test: "k,," },
                      { f: "Token Break", t: "` (backtick)", b: "(break)", ex: "ek`i → একই (breaks ek+i), pOt`h → পোতহ (breaks th)", test: "ek`i" },
                      { f: "Dari / Punctuation", t: "| and ||", b: "। and ॥", ex: "| → ।, || → ॥", test: "|" },
                    ].map((row, i) => (
                      <tr key={i} style={{ borderBottom: "1px solid rgba(255,255,255,0.04)" }}>
                        <td style={{ padding: "6px 8px", color: "var(--accent-emerald)" }}>{row.f}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-mono)", color: "var(--accent-amber)" }}>{row.t}</td>
                        <td style={{ padding: "6px 8px", fontFamily: "var(--font-bangla)", fontSize: "16px", color: "#ffffff" }}>{row.b}</td>
                        <td style={{ padding: "6px 8px", color: "var(--text-secondary)" }}>{row.ex}</td>
                        <td style={{ padding: "6px 8px" }}>
                          <button
                            onClick={() => onQuickInput(row.test)}
                            style={{
                              padding: "2px 8px",
                              fontSize: "10px",
                              borderRadius: "4px",
                              backgroundColor: "rgba(14, 165, 233, 0.15)",
                              color: "var(--accent-cyan)",
                              border: "1px solid rgba(14, 165, 233, 0.3)",
                              cursor: "pointer",
                            }}
                          >
                            {row.test}
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            {/* Tab 5: Candidates */}
            {guideTab === "candidates" && (
              <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.6" }}>
                <div style={{ marginBottom: "12px", padding: "10px", borderRadius: "6px", backgroundColor: "rgba(0,0,0,0.3)", border: "1px solid rgba(255,255,255,0.05)" }}>
                  <h4 style={{ color: "var(--accent-cyan)", fontWeight: "600", marginBottom: "4px" }}>
                    Deterministic Phonetics vs Dictionary Autocorrect (Strict M3 Scope)
                  </h4>
                  <p>
                    Candidate 0 is always the pure, deterministic phonetic transliteration. Candidates 1..N represent bounded, rule-based orthographic spelling alternatives. Word-specific dictionary overrides (e.g. khoma→ক্ষমা) are strictly deferred to Milestone M6.
                  </p>
                </div>

                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                  <div style={{ padding: "10px", borderRadius: "6px", backgroundColor: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.05)" }}>
                    <div style={{ color: "var(--accent-amber)", fontWeight: "600", marginBottom: "4px" }}>
                      Rule ALT-1: Word-Final High Vowels
                    </div>
                    <p style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                      When a word ends with high vowel 'i' or 'u', generates the alternate long vowel:
                    </p>
                    <ul style={{ paddingLeft: "16px", marginTop: "4px", fontSize: "11px" }}>
                      <li><code>ami</code> → ["আমি", "আমী"]</li>
                      <li><code>tumi</code> → ["তুমি", "তূমি"]</li>
                      <li><code>baRi</code> → ["বাড়ি", "বাড়ী"]</li>
                      <li><code>pakhi</code> → ["পাখি", "পাখী"]</li>
                    </ul>
                  </div>

                  <div style={{ padding: "10px", borderRadius: "6px", backgroundColor: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.05)" }}>
                    <div style={{ color: "var(--accent-amber)", fontWeight: "600", marginBottom: "4px" }}>
                      Rule ALT-2: Sibilant Alternations
                    </div>
                    <p style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                      Initial homophones and medial loanword clusters:
                    </p>
                    <ul style={{ paddingLeft: "16px", marginTop: "4px", fontSize: "11px" }}>
                      <li><code>shanto</code> → ["শান্ত", "সান্ত"]</li>
                      <li><code>kosTo</code> → ["কস্ট", "কষ্ট"] (sT cluster alternate)</li>
                      <li>Candidate Cap: strictly capped at maximum 5 total</li>
                    </ul>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Font Shaping & Complex Conjuncts Verification Panel */}
      <div
        style={{
          marginTop: "24px",
          paddingTop: "20px",
          borderTop: "1px solid var(--border-subtle)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "14px", flexWrap: "wrap", gap: "10px" }}>
          <div>
            <h3 style={{ fontSize: "14px", fontWeight: "600", display: "flex", alignItems: "center", gap: "8px" }}>
              <span>Font Shaping & Complex Conjunct Verification</span>
              <span
                style={{
                  fontSize: "10px",
                  padding: "2px 7px",
                  borderRadius: "4px",
                  backgroundColor: "rgba(16, 185, 129, 0.15)",
                  color: "var(--accent-emerald)",
                  fontWeight: "600",
                  letterSpacing: "0.03em",
                }}
              >
                DirectWrite / HarfBuzz Continuous Runs
              </span>
            </h3>
            <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
              Validates that complex ligatures, two-part vowels, nuktas, and joiners shape natively without broken dotted circles (◌) in the host WebView2 environment.
            </p>
          </div>

          {/* Category filter pills */}
          <div style={{ display: "flex", gap: "6px", flexWrap: "wrap" }}>
            {(["All", "Complex Conjuncts", "Two-Part Vowels", "Nukta Letters", "Khanda-Ta & Joiners"] as const).map((cat) => (
              <button
                key={cat}
                onClick={() => setActiveCategory(cat)}
                style={{
                  padding: "3px 10px",
                  borderRadius: "6px",
                  fontSize: "11px",
                  fontWeight: activeCategory === cat ? "600" : "400",
                  backgroundColor: activeCategory === cat ? "rgba(6, 182, 212, 0.2)" : "rgba(255, 255, 255, 0.04)",
                  color: activeCategory === cat ? "var(--accent-cyan)" : "var(--text-secondary)",
                  border: `1px solid ${activeCategory === cat ? "rgba(6, 182, 212, 0.4)" : "var(--border-subtle)"}`,
                  cursor: "pointer",
                  transition: "all 0.15s ease",
                }}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>

        {/* Shaping Cards Grid */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
            gap: "12px",
          }}
        >
          {SHAPING_SAMPLES.filter((s) => activeCategory === "All" || s.category === activeCategory).map((s) => (
            <div
              key={s.id}
              style={{
                padding: "12px 14px",
                borderRadius: "8px",
                backgroundColor: "rgba(11, 15, 25, 0.7)",
                border: "1px solid var(--border-subtle)",
                display: "flex",
                flexDirection: "column",
                justifyContent: "space-between",
                gap: "8px",
                transition: "border-color 0.2s, transform 0.15s ease",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = "rgba(6, 182, 212, 0.35)";
                e.currentTarget.style.transform = "translateY(-1px)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = "var(--border-subtle)";
                e.currentTarget.style.transform = "translateY(0)";
              }}
            >
              <div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "4px" }}>
                  <span
                    style={{
                      fontSize: "10px",
                      fontWeight: "600",
                      padding: "1px 6px",
                      borderRadius: "4px",
                      backgroundColor:
                        s.category === "Complex Conjuncts"
                          ? "rgba(99, 102, 241, 0.15)"
                          : s.category === "Two-Part Vowels"
                          ? "rgba(245, 158, 11, 0.15)"
                          : s.category === "Nukta Letters"
                          ? "rgba(16, 185, 129, 0.15)"
                          : "rgba(244, 63, 94, 0.15)",
                      color:
                        s.category === "Complex Conjuncts"
                          ? "var(--accent-indigo)"
                          : s.category === "Two-Part Vowels"
                          ? "var(--accent-amber)"
                          : s.category === "Nukta Letters"
                          ? "var(--accent-emerald)"
                          : "var(--accent-rose)",
                    }}
                  >
                    {s.label}
                  </span>
                  <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>
                    {s.category}
                  </span>
                </div>

                {/* Unbroken continuous shaped run */}
                <div
                  style={{
                    fontSize: "26px",
                    fontFamily: "var(--font-bangla)",
                    fontWeight: "500",
                    color: "#ffffff",
                    letterSpacing: "0.02em",
                    lineHeight: "1.25",
                    margin: "4px 0 2px 0",
                  }}
                  title="Unbroken continuous text run shaped natively by host DirectWrite/HarfBuzz"
                >
                  {s.word}
                </div>

                <div style={{ fontSize: "11px", color: "var(--accent-cyan)", fontFamily: "var(--font-bangla)", marginBottom: "4px" }}>
                  {s.breakdown}
                </div>

                <div
                  style={{
                    fontSize: "10px",
                    fontFamily: "var(--font-mono)",
                    color: "var(--text-muted)",
                    padding: "3px 6px",
                    borderRadius: "4px",
                    backgroundColor: "rgba(0, 0, 0, 0.35)",
                    border: "1px solid rgba(255, 255, 255, 0.05)",
                    overflowX: "auto",
                    whiteSpace: "nowrap",
                  }}
                  title={s.codepoints}
                >
                  {s.codepoints}
                </div>
              </div>

              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "4px", paddingTop: "6px", borderTop: "1px solid rgba(255, 255, 255, 0.04)" }}>
                <span style={{ fontSize: "10px", color: "var(--text-muted)", fontStyle: "italic", maxWidth: "160px", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={s.note}>
                  {s.note}
                </span>

                <div style={{ display: "flex", gap: "6px" }}>
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(s.word);
                      setCopiedId(s.id);
                      setTimeout(() => setCopiedId(null), 1500);
                    }}
                    style={{
                      padding: "3px 8px",
                      fontSize: "10px",
                      borderRadius: "4px",
                      backgroundColor: copiedId === s.id ? "rgba(16, 185, 129, 0.2)" : "rgba(255, 255, 255, 0.05)",
                      color: copiedId === s.id ? "var(--accent-emerald)" : "var(--text-secondary)",
                      border: `1px solid ${copiedId === s.id ? "rgba(16, 185, 129, 0.4)" : "var(--border-subtle)"}`,
                      cursor: "pointer",
                      transition: "all 0.15s ease",
                    }}
                    title="Copy word to clipboard"
                  >
                    {copiedId === s.id ? "Copied!" : "Copy"}
                  </button>

                  {onAppendCommitted && (
                    <button
                      onClick={() => onAppendCommitted(s.word)}
                      style={{
                        padding: "3px 8px",
                        fontSize: "10px",
                        borderRadius: "4px",
                        backgroundColor: "rgba(6, 182, 212, 0.15)",
                        color: "var(--accent-cyan)",
                        border: "1px solid rgba(6, 182, 212, 0.3)",
                        cursor: "pointer",
                        transition: "all 0.15s ease",
                      }}
                      title="Append this shaped sample to the committed output buffer"
                    >
                      + Insert
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
