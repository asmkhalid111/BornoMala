import React, { useState, useEffect, useRef, useCallback } from "react";
import { BornoConfig, PersistenceStatus, StartupMode, FixedLayoutId } from "../types";

interface SettingsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  config: BornoConfig | null;
  persistenceStatus: PersistenceStatus | null;
  configPath: string;
  onSave: (config: BornoConfig) => Promise<void>;
  onReset: () => Promise<void>;
}

export const SettingsDrawer: React.FC<SettingsDrawerProps> = ({
  isOpen,
  onClose,
  config,
  persistenceStatus,
  configPath,
  onSave,
  onReset,
}) => {
  // Form state
  const [formData, setFormData] = useState<BornoConfig>({
    schema_version: "1.1.0",
    default_startup_mode: "Phonetic",
    preferred_fixed_layout: "National",
    candidate_display_limit: 5,
    auto_commit_on_mode_switch: true,
    enable_dictionary_assistance: true,
    enable_word_completion: true,
    max_dictionary_suggestions: 3,
  });

  const [isSaving, setIsSaving] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [saveSuccessMsg, setSaveSuccessMsg] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Modals & prompts
  const [showDiscardPrompt, setShowDiscardPrompt] = useState(false);
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  // Focus management refs
  const drawerRef = useRef<HTMLDivElement>(null);
  const triggerElementRef = useRef<HTMLElement | null>(null);
  const firstInputRef = useRef<HTMLSelectElement>(null);

  // Synchronize form with incoming config
  useEffect(() => {
    if (config) {
      setFormData({
        schema_version: config.schema_version,
        default_startup_mode: config.default_startup_mode,
        preferred_fixed_layout: config.preferred_fixed_layout,
        candidate_display_limit: config.candidate_display_limit,
        auto_commit_on_mode_switch: config.auto_commit_on_mode_switch,
        enable_dictionary_assistance: config.enable_dictionary_assistance ?? true,
        enable_word_completion: config.enable_word_completion ?? true,
        max_dictionary_suggestions: config.max_dictionary_suggestions ?? 3,
      });
    }
  }, [config]);

  // Compute dirty state
  const isDirty = Boolean(
    config &&
      (formData.default_startup_mode !== config.default_startup_mode ||
        formData.preferred_fixed_layout !== config.preferred_fixed_layout ||
        formData.candidate_display_limit !== config.candidate_display_limit ||
        formData.auto_commit_on_mode_switch !== config.auto_commit_on_mode_switch ||
        formData.enable_dictionary_assistance !== config.enable_dictionary_assistance ||
        formData.enable_word_completion !== config.enable_word_completion ||
        formData.max_dictionary_suggestions !== config.max_dictionary_suggestions)
  );

  // Check if persistence is locked
  const isLocked = Boolean(
    persistenceStatus &&
      ("LockedUnsupportedVersion" in persistenceStatus ||
        persistenceStatus.type === "LockedUnsupportedVersion" ||
        persistenceStatus.type === "ReadOnlyPermissionDenied")
  );

  const unsupportedVersion =
    persistenceStatus && "LockedUnsupportedVersion" in persistenceStatus
      ? (persistenceStatus as any).LockedUnsupportedVersion?.version ||
        (persistenceStatus as any).payload?.version
      : persistenceStatus?.type === "LockedUnsupportedVersion"
      ? (persistenceStatus as any).payload?.version
      : null;

  // Handle opening and focus setup
  useEffect(() => {
    if (isOpen) {
      triggerElementRef.current = document.activeElement as HTMLElement;
      setSaveSuccessMsg(null);
      setErrorMessage(null);
      setShowDiscardPrompt(false);
      setShowResetConfirm(false);

      // Focus first input after drawer animation frame
      requestAnimationFrame(() => {
        firstInputRef.current?.focus();
      });
    } else {
      // Focus restoration
      if (triggerElementRef.current) {
        triggerElementRef.current.focus();
      } else {
        document.getElementById("settings-open-btn")?.focus();
      }
    }
  }, [isOpen]);

  // Dismiss attempt with unsaved change protection
  const attemptClose = useCallback(() => {
    if (isDirty) {
      setShowDiscardPrompt(true);
    } else {
      onClose();
    }
  }, [isDirty, onClose]);

  // Keyboard navigation: Escape key & focus trapping
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        if (showResetConfirm) {
          setShowResetConfirm(false);
        } else if (showDiscardPrompt) {
          setShowDiscardPrompt(false);
        } else {
          attemptClose();
        }
        return;
      }

      if (e.key === "Tab") {
        if (!drawerRef.current) return;
        const focusableElements = drawerRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        if (focusableElements.length === 0) return;

        const firstElement = focusableElements[0];
        const lastElement = focusableElements[focusableElements.length - 1];

        if (e.shiftKey) {
          if (document.activeElement === firstElement) {
            e.preventDefault();
            lastElement.focus();
          }
        } else {
          if (document.activeElement === lastElement) {
            e.preventDefault();
            firstElement.focus();
          }
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, showResetConfirm, showDiscardPrompt, attemptClose]);

  if (!isOpen) return null;

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isLocked) return;

    setIsSaving(true);
    setErrorMessage(null);
    setSaveSuccessMsg(null);

    try {
      await onSave(formData);
      setSaveSuccessMsg("Settings saved successfully.");
      setTimeout(() => {
        setSaveSuccessMsg(null);
      }, 4000);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(`Failed to save settings: ${msg}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleConfirmReset = async () => {
    setIsResetting(true);
    setErrorMessage(null);
    setSaveSuccessMsg(null);

    try {
      await onReset();
      setShowResetConfirm(false);
      setSaveSuccessMsg("Configuration reset to defaults.");
      setTimeout(() => {
        setSaveSuccessMsg(null);
      }, 4000);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(`Failed to reset configuration: ${msg}`);
    } finally {
      setIsResetting(false);
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Application Settings"
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100vw",
        height: "100vh",
        backgroundColor: "rgba(0, 0, 0, 0.65)",
        backdropFilter: "blur(6px)",
        WebkitBackdropFilter: "blur(6px)",
        zIndex: 1000,
        display: "flex",
        justifyContent: "flex-end",
        transition: "opacity 0.25s ease",
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          attemptClose();
        }
      }}
    >
      <div
        ref={drawerRef}
        className="glass-panel"
        style={{
          width: "460px",
          maxWidth: "100vw",
          height: "100%",
          backgroundColor: "var(--bg-secondary)",
          borderLeft: "1px solid var(--border-subtle)",
          borderTop: "none",
          borderBottom: "none",
          borderRight: "none",
          borderRadius: 0,
          boxShadow: "-8px 0 32px rgba(0, 0, 0, 0.7)",
          display: "flex",
          flexDirection: "column",
          overflowY: "auto",
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: "20px 24px",
            borderBottom: "1px solid var(--border-subtle)",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "20px" }}>⚙️</span>
            <div>
              <h2 style={{ fontSize: "17px", fontWeight: "700", color: "var(--text-primary)" }}>
                Application Settings
              </h2>
              <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
                Configuration & Persistence (Schema v1.1.0)
              </p>
            </div>
          </div>
          <button
            id="settings-close-btn"
            onClick={attemptClose}
            aria-label="Close settings drawer"
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-secondary)",
              fontSize: "20px",
              cursor: "pointer",
              padding: "6px",
              borderRadius: "6px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            ✕
          </button>
        </div>

        {/* Content Body */}
        <form onSubmit={handleSave} style={{ padding: "24px", flex: 1, display: "flex", flexDirection: "column", gap: "20px" }}>
          {/* Success Banner */}
          {saveSuccessMsg && (
            <div
              role="status"
              aria-live="polite"
              style={{
                padding: "12px 16px",
                borderRadius: "8px",
                backgroundColor: "rgba(16, 185, 129, 0.15)",
                border: "1px solid rgba(16, 185, 129, 0.35)",
                color: "var(--accent-emerald)",
                fontSize: "13px",
                display: "flex",
                alignItems: "center",
                gap: "8px",
              }}
            >
              <span>✓</span>
              <span>{saveSuccessMsg}</span>
            </div>
          )}

          {/* Error Banner */}
          {errorMessage && (
            <div
              role="alert"
              aria-live="assertive"
              style={{
                padding: "12px 16px",
                borderRadius: "8px",
                backgroundColor: "rgba(244, 63, 94, 0.15)",
                border: "1px solid rgba(244, 63, 94, 0.35)",
                color: "var(--accent-rose)",
                fontSize: "13px",
                display: "flex",
                alignItems: "center",
                gap: "8px",
              }}
            >
              <span>⚠️</span>
              <span>{errorMessage}</span>
            </div>
          )}

          {/* Locked Version Alert */}
          {unsupportedVersion && (
            <div
              role="alert"
              style={{
                padding: "12px 16px",
                borderRadius: "8px",
                backgroundColor: "rgba(245, 158, 11, 0.15)",
                border: "1px solid rgba(245, 158, 11, 0.4)",
                color: "var(--accent-amber)",
                fontSize: "12px",
                lineHeight: "1.5",
              }}
            >
              <strong>Unsupported Schema Version Detected:</strong> File on disk is format v{unsupportedVersion}.
              Saving is locked to preserve your configuration. Resetting will restore v1.0.0 defaults (with automatic backup).
            </div>
          )}

          {/* Section: General Preferences */}
          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-cyan)" }}>
              General Typing Preferences
            </h3>

            {/* Default Startup Mode */}
            <div>
              <label
                htmlFor="setting-startup-mode"
                style={{ display: "block", fontSize: "13px", fontWeight: "600", marginBottom: "6px", color: "var(--text-primary)" }}
              >
                Default Startup Mode
              </label>
              <select
                id="setting-startup-mode"
                ref={firstInputRef}
                value={formData.default_startup_mode}
                onChange={(e) => setFormData({ ...formData, default_startup_mode: e.target.value as StartupMode })}
                style={{
                  width: "100%",
                  padding: "9px 12px",
                  borderRadius: "8px",
                  backgroundColor: "rgba(11, 15, 25, 0.8)",
                  border: "1px solid var(--border-subtle)",
                  color: "var(--text-primary)",
                  fontSize: "13px",
                  outline: "none",
                  cursor: "pointer",
                }}
              >
                <option value="Phonetic">Phonetic (Transliteration)</option>
                <option value="Fixed">Fixed Keyboard Layout</option>
                <option value="EnglishPassthrough">English Passthrough</option>
              </select>
              <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                Applied during cold application launch. Does not disrupt ongoing sessions.
              </p>
            </div>

            {/* Preferred Fixed Layout */}
            <div>
              <label
                htmlFor="setting-preferred-layout"
                style={{ display: "block", fontSize: "13px", fontWeight: "600", marginBottom: "6px", color: "var(--text-primary)" }}
              >
                Preferred Fixed Layout
              </label>
              <select
                id="setting-preferred-layout"
                value={formData.preferred_fixed_layout}
                onChange={(e) => setFormData({ ...formData, preferred_fixed_layout: e.target.value as FixedLayoutId })}
                style={{
                  width: "100%",
                  padding: "9px 12px",
                  borderRadius: "8px",
                  backgroundColor: "rgba(11, 15, 25, 0.8)",
                  border: "1px solid var(--border-subtle)",
                  color: "var(--text-primary)",
                  fontSize: "13px",
                  outline: "none",
                  cursor: "pointer",
                }}
              >
                <option value="National">National (Jatiya / BCC)</option>
                <option value="Probhat">Probhat (Standard Phonetic Fixed)</option>
                <option value="Bijoy">Bijoy (Compatible Unicode Layout)</option>
              </select>
              <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                Stored preference for next Fixed mode entry. Never alters active composition.
              </p>
            </div>

            {/* Candidate Display Limit */}
            <div>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                <label
                  htmlFor="setting-candidate-limit"
                  style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}
                >
                  Candidate Display Limit
                </label>
                <span
                  style={{
                    fontSize: "12px",
                    fontWeight: "700",
                    padding: "2px 8px",
                    borderRadius: "6px",
                    backgroundColor: "rgba(14, 165, 233, 0.2)",
                    color: "var(--accent-cyan)",
                    border: "1px solid rgba(14, 165, 233, 0.4)",
                  }}
                >
                  {formData.candidate_display_limit}
                </span>
              </div>
              <input
                id="setting-candidate-limit"
                type="range"
                min={1}
                max={9}
                value={formData.candidate_display_limit}
                onChange={(e) => setFormData({ ...formData, candidate_display_limit: parseInt(e.target.value, 10) })}
                style={{
                  width: "100%",
                  accentColor: "var(--accent-cyan)",
                  cursor: "pointer",
                }}
              />
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "10px", color: "var(--text-muted)", marginTop: "2px" }}>
                <span>1 (Minimal)</span>
                <span>5 (Standard)</span>
                <span>9 (Max)</span>
              </div>
              <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                Truncates candidate tiles displayed in phonetic suggestion window without altering linguistic ranking.
              </p>
            </div>

            {/* Auto-commit on Mode Switch */}
            <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "12px", paddingTop: "8px" }}>
              <div>
                <label
                  htmlFor="setting-auto-commit"
                  style={{ display: "block", fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", cursor: "pointer" }}
                >
                  Auto-commit on Mode Switch
                </label>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                  When enabled, switching modes commits active preedit in NFC; otherwise cancels composition.
                </p>
              </div>
              <button
                id="setting-auto-commit"
                type="button"
                role="switch"
                aria-checked={formData.auto_commit_on_mode_switch}
                onClick={() => setFormData({ ...formData, auto_commit_on_mode_switch: !formData.auto_commit_on_mode_switch })}
                style={{
                  width: "44px",
                  height: "24px",
                  borderRadius: "12px",
                  backgroundColor: formData.auto_commit_on_mode_switch ? "var(--accent-emerald)" : "rgba(255, 255, 255, 0.15)",
                  border: "none",
                  cursor: "pointer",
                  position: "relative",
                  transition: "background-color 0.2s ease",
                  flexShrink: 0,
                  marginTop: "2px",
                }}
              >
                <span
                  style={{
                    position: "absolute",
                    top: "3px",
                    left: formData.auto_commit_on_mode_switch ? "23px" : "3px",
                    width: "18px",
                    height: "18px",
                    borderRadius: "50%",
                    backgroundColor: "#ffffff",
                    transition: "left 0.2s ease",
                    boxShadow: "0 1px 3px rgba(0, 0, 0, 0.4)",
                  }}
                />
              </button>
            </div>
          </div>

          {/* Section: Dictionary & Predictive Lexicon (M6) */}
          <div style={{ marginTop: "12px", paddingTop: "18px", borderTop: "1px solid var(--border-subtle)", display: "flex", flexDirection: "column", gap: "16px" }}>
            <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-indigo)" }}>
              Dictionary & Writing Assistance
            </h3>

            {/* Enable Dictionary Assistance */}
            <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "12px" }}>
              <div>
                <label
                  htmlFor="setting-dictionary-assistance"
                  style={{ display: "block", fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", cursor: "pointer" }}
                >
                  Dictionary Assistance
                </label>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                  Enables offline lexicon lookups, homophone disambiguation (e.g. ক্ষমা, বিজ্ঞান), and spelling correction.
                </p>
              </div>
              <button
                id="setting-dictionary-assistance"
                type="button"
                role="switch"
                aria-checked={formData.enable_dictionary_assistance}
                onClick={() => setFormData({ ...formData, enable_dictionary_assistance: !formData.enable_dictionary_assistance })}
                style={{
                  width: "44px",
                  height: "24px",
                  borderRadius: "12px",
                  backgroundColor: formData.enable_dictionary_assistance ? "var(--accent-emerald)" : "rgba(255, 255, 255, 0.15)",
                  border: "none",
                  cursor: "pointer",
                  position: "relative",
                  transition: "background-color 0.2s ease",
                  flexShrink: 0,
                  marginTop: "2px",
                }}
              >
                <span
                  style={{
                    position: "absolute",
                    top: "3px",
                    left: formData.enable_dictionary_assistance ? "23px" : "3px",
                    width: "18px",
                    height: "18px",
                    borderRadius: "50%",
                    backgroundColor: "#ffffff",
                    transition: "left 0.2s ease",
                    boxShadow: "0 1px 3px rgba(0, 0, 0, 0.4)",
                  }}
                />
              </button>
            </div>

            {/* Enable Word Completion */}
            <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "12px", opacity: formData.enable_dictionary_assistance ? 1 : 0.45 }}>
              <div>
                <label
                  htmlFor="setting-word-completion"
                  style={{ display: "block", fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", cursor: formData.enable_dictionary_assistance ? "pointer" : "not-allowed" }}
                >
                  Predictive Word Completion
                </label>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                  Suggests full dictionary completions (e.g. &apos;বাং&apos; → বাংলাদেশ, বাঙালি) for inputs with 2+ characters.
                </p>
              </div>
              <button
                id="setting-word-completion"
                type="button"
                role="switch"
                disabled={!formData.enable_dictionary_assistance}
                aria-checked={formData.enable_word_completion}
                onClick={() => setFormData({ ...formData, enable_word_completion: !formData.enable_word_completion })}
                style={{
                  width: "44px",
                  height: "24px",
                  borderRadius: "12px",
                  backgroundColor: formData.enable_dictionary_assistance && formData.enable_word_completion ? "var(--accent-emerald)" : "rgba(255, 255, 255, 0.15)",
                  border: "none",
                  cursor: formData.enable_dictionary_assistance ? "pointer" : "not-allowed",
                  position: "relative",
                  transition: "background-color 0.2s ease",
                  flexShrink: 0,
                  marginTop: "2px",
                }}
              >
                <span
                  style={{
                    position: "absolute",
                    top: "3px",
                    left: formData.enable_dictionary_assistance && formData.enable_word_completion ? "23px" : "3px",
                    width: "18px",
                    height: "18px",
                    borderRadius: "50%",
                    backgroundColor: "#ffffff",
                    transition: "left 0.2s ease",
                    boxShadow: "0 1px 3px rgba(0, 0, 0, 0.4)",
                  }}
                />
              </button>
            </div>

            {/* Max Dictionary Suggestions */}
            <div style={{ opacity: formData.enable_dictionary_assistance ? 1 : 0.45 }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                <label
                  htmlFor="setting-max-dict-suggestions"
                  style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}
                >
                  Max Dictionary Suggestions
                </label>
                <span
                  style={{
                    fontSize: "12px",
                    fontWeight: "700",
                    padding: "2px 8px",
                    borderRadius: "6px",
                    backgroundColor: "rgba(99, 102, 241, 0.2)",
                    color: "var(--accent-indigo)",
                    border: "1px solid rgba(99, 102, 241, 0.4)",
                  }}
                >
                  {formData.max_dictionary_suggestions}
                </span>
              </div>
              <input
                id="setting-max-dict-suggestions"
                type="range"
                min={1}
                max={5}
                disabled={!formData.enable_dictionary_assistance}
                value={formData.max_dictionary_suggestions}
                onChange={(e) => setFormData({ ...formData, max_dictionary_suggestions: parseInt(e.target.value, 10) })}
                style={{
                  width: "100%",
                  accentColor: "var(--accent-indigo)",
                  cursor: formData.enable_dictionary_assistance ? "pointer" : "not-allowed",
                }}
              />
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "10px", color: "var(--text-muted)", marginTop: "2px" }}>
                <span>1</span>
                <span>2</span>
                <span>3 (Default)</span>
                <span>4</span>
                <span>5 (Max)</span>
              </div>
              <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                Caps secondary dictionary candidates. Slot 0 is always reserved for the primary transliteration.
              </p>
            </div>
          </div>

          {/* Section: Storage & Diagnostics */}
          <div style={{ marginTop: "12px", paddingTop: "18px", borderTop: "1px solid var(--border-subtle)" }}>
            <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--text-secondary)", marginBottom: "10px" }}>
              Storage & Diagnostics
            </h3>

            <div style={{ backgroundColor: "rgba(11, 15, 25, 0.7)", padding: "12px", borderRadius: "8px", border: "1px solid var(--border-subtle)" }}>
              <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "4px" }}>
                Configuration File Location:
              </div>
              <div
                style={{
                  fontSize: "11px",
                  fontFamily: "var(--font-mono)",
                  color: "var(--text-secondary)",
                  wordBreak: "break-all",
                  backgroundColor: "rgba(0, 0, 0, 0.3)",
                  padding: "6px 8px",
                  borderRadius: "4px",
                }}
              >
                {configPath || "Resolving OS path..."}
              </div>

              <div style={{ display: "flex", alignItems: "center", gap: "8px", marginTop: "10px" }}>
                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Persistence Status:</span>
                <span
                  style={{
                    fontSize: "11px",
                    fontWeight: "600",
                    padding: "2px 8px",
                    borderRadius: "4px",
                    backgroundColor: isLocked ? "rgba(244, 63, 94, 0.2)" : "rgba(16, 185, 129, 0.2)",
                    color: isLocked ? "var(--accent-rose)" : "var(--accent-emerald)",
                  }}
                >
                  {isLocked ? "Read-Only / Locked" : "Active (Writable)"}
                </span>
              </div>
            </div>
          </div>

          {/* Action Bar */}
          <div
            style={{
              marginTop: "auto",
              paddingTop: "20px",
              borderTop: "1px solid var(--border-subtle)",
              display: "flex",
              flexDirection: "column",
              gap: "10px",
            }}
          >
            <div style={{ display: "flex", gap: "10px" }}>
              <button
                id="settings-save-btn"
                type="submit"
                disabled={!isDirty || isLocked || isSaving}
                aria-disabled={!isDirty || isLocked || isSaving}
                style={{
                  flex: 1,
                  padding: "10px 16px",
                  borderRadius: "8px",
                  backgroundColor: isDirty && !isLocked ? "var(--accent-cyan)" : "rgba(255, 255, 255, 0.08)",
                  color: isDirty && !isLocked ? "#ffffff" : "var(--text-muted)",
                  border: "none",
                  fontWeight: "600",
                  fontSize: "13px",
                  cursor: isDirty && !isLocked && !isSaving ? "pointer" : "not-allowed",
                  transition: "all 0.2s ease",
                }}
              >
                {isSaving ? "Saving..." : "Save Settings"}
              </button>

              <button
                type="button"
                onClick={attemptClose}
                style={{
                  padding: "10px 16px",
                  borderRadius: "8px",
                  backgroundColor: "transparent",
                  color: "var(--text-secondary)",
                  border: "1px solid var(--border-subtle)",
                  fontWeight: "500",
                  fontSize: "13px",
                  cursor: "pointer",
                }}
              >
                Cancel
              </button>
            </div>

            <button
              id="settings-reset-btn"
              type="button"
              onClick={() => setShowResetConfirm(true)}
              disabled={isResetting}
              style={{
                padding: "8px 12px",
                borderRadius: "8px",
                backgroundColor: "rgba(244, 63, 94, 0.1)",
                color: "var(--accent-rose)",
                border: "1px solid rgba(244, 63, 94, 0.25)",
                fontWeight: "500",
                fontSize: "12px",
                cursor: isResetting ? "not-allowed" : "pointer",
                transition: "all 0.2s ease",
              }}
            >
              Reset All Settings to Defaults
            </button>
          </div>
        </form>

        {/* Modal: Unsaved Changes Prompt */}
        {showDiscardPrompt && (
          <div
            role="alertdialog"
            aria-modal="true"
            aria-labelledby="discard-title"
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              height: "100%",
              backgroundColor: "rgba(0, 0, 0, 0.8)",
              backdropFilter: "blur(4px)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: "24px",
              zIndex: 1100,
            }}
          >
            <div
              className="glass-panel"
              style={{
                backgroundColor: "var(--bg-secondary)",
                padding: "24px",
                borderRadius: "12px",
                border: "1px solid var(--border-subtle)",
                maxWidth: "360px",
                width: "100%",
              }}
            >
              <h4 id="discard-title" style={{ fontSize: "15px", fontWeight: "700", marginBottom: "8px", color: "var(--text-primary)" }}>
                Unsaved Changes
              </h4>
              <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "20px" }}>
                You have unsaved changes in your settings. Discard edits and close?
              </p>
              <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
                <button
                  type="button"
                  onClick={() => setShowDiscardPrompt(false)}
                  style={{
                    padding: "8px 14px",
                    borderRadius: "6px",
                    backgroundColor: "transparent",
                    color: "var(--text-secondary)",
                    border: "1px solid var(--border-subtle)",
                    fontSize: "12px",
                    cursor: "pointer",
                  }}
                >
                  Keep Editing
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setShowDiscardPrompt(false);
                    onClose();
                  }}
                  style={{
                    padding: "8px 14px",
                    borderRadius: "6px",
                    backgroundColor: "var(--accent-rose)",
                    color: "#ffffff",
                    border: "none",
                    fontWeight: "600",
                    fontSize: "12px",
                    cursor: "pointer",
                  }}
                >
                  Discard & Close
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Modal: Reset Confirmation */}
        {showResetConfirm && (
          <div
            role="alertdialog"
            aria-modal="true"
            aria-labelledby="reset-title"
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              height: "100%",
              backgroundColor: "rgba(0, 0, 0, 0.8)",
              backdropFilter: "blur(4px)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: "24px",
              zIndex: 1100,
            }}
          >
            <div
              className="glass-panel"
              style={{
                backgroundColor: "var(--bg-secondary)",
                padding: "24px",
                borderRadius: "12px",
                border: "1px solid rgba(244, 63, 94, 0.3)",
                maxWidth: "380px",
                width: "100%",
              }}
            >
              <h4 id="reset-title" style={{ fontSize: "15px", fontWeight: "700", marginBottom: "8px", color: "var(--accent-rose)" }}>
                Reset Settings to Factory Defaults
              </h4>
              <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "20px", lineHeight: "1.5" }}>
                {unsupportedVersion
                  ? `Warning: The configuration file on disk has an unsupported version (v${unsupportedVersion}). Resetting will overwrite this file with v1.0.0 defaults (a backup will be created). Are you sure you want to proceed?`
                  : "Are you sure you want to reset all settings to defaults? Your current preferences on disk and in memory will be overwritten."}
              </p>
              <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
                <button
                  type="button"
                  onClick={() => setShowResetConfirm(false)}
                  disabled={isResetting}
                  style={{
                    padding: "8px 14px",
                    borderRadius: "6px",
                    backgroundColor: "transparent",
                    color: "var(--text-secondary)",
                    border: "1px solid var(--border-subtle)",
                    fontSize: "12px",
                    cursor: "pointer",
                  }}
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onClick={handleConfirmReset}
                  disabled={isResetting}
                  style={{
                    padding: "8px 14px",
                    borderRadius: "6px",
                    backgroundColor: "var(--accent-rose)",
                    color: "#ffffff",
                    border: "none",
                    fontWeight: "600",
                    fontSize: "12px",
                    cursor: isResetting ? "not-allowed" : "pointer",
                  }}
                >
                  {isResetting ? "Resetting..." : "Confirm Reset"}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
