import React, { useState, useEffect, useRef, useCallback } from "react";
import {
  BornoConfig,
  PersistenceStatus,
  StartupMode,
  FixedLayoutId,
  CandidateOrientation,
  ThemeId,
  ModeToggleKey,
} from "../types";

interface SettingsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  config: BornoConfig | null;
  persistenceStatus: PersistenceStatus | null;
  configPath: string;
  onSave: (config: BornoConfig) => Promise<void>;
  onReset: () => Promise<void>;
  onExport?: () => Promise<string>;
  onImport?: (jsonOrPath: string) => Promise<BornoConfig>;
}

type SettingsTab = "typing" | "appearance" | "shortcuts" | "backup";

export const SettingsDrawer: React.FC<SettingsDrawerProps> = ({
  isOpen,
  onClose,
  config,
  persistenceStatus,
  configPath,
  onSave,
  onReset,
  onExport,
  onImport,
}) => {
  // Navigation tab state
  const [activeTab, setActiveTab] = useState<SettingsTab>("typing");

  // Form state (v1.2.0)
  const [formData, setFormData] = useState<BornoConfig>({
    schema_version: "1.2.0",
    default_startup_mode: "Phonetic",
    preferred_fixed_layout: "National",
    candidate_display_limit: 5,
    auto_commit_on_mode_switch: true,
    enable_dictionary_assistance: true,
    enable_word_completion: true,
    max_dictionary_suggestions: 3,
    auto_convert_period_to_dari: false,
    candidate_orientation: "horizontal",
    candidate_font_size: 14,
    active_theme: "dark-glass",
    mode_toggle_key: "F12",
  });

  const [isSaving, setIsSaving] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [saveSuccessMsg, setSaveSuccessMsg] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Modals & prompts
  const [showDiscardPrompt, setShowDiscardPrompt] = useState(false);
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  // File import ref
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Focus management refs
  const drawerRef = useRef<HTMLDivElement>(null);
  const triggerElementRef = useRef<HTMLElement | null>(null);

  // Synchronize form with incoming config
  useEffect(() => {
    if (config) {
      setFormData({
        schema_version: config.schema_version ?? "1.2.0",
        default_startup_mode: config.default_startup_mode ?? "Phonetic",
        preferred_fixed_layout: config.preferred_fixed_layout ?? "National",
        candidate_display_limit: config.candidate_display_limit ?? 5,
        auto_commit_on_mode_switch: config.auto_commit_on_mode_switch ?? true,
        enable_dictionary_assistance: config.enable_dictionary_assistance ?? true,
        enable_word_completion: config.enable_word_completion ?? true,
        max_dictionary_suggestions: config.max_dictionary_suggestions ?? 3,
        auto_convert_period_to_dari: config.auto_convert_period_to_dari ?? false,
        candidate_orientation: config.candidate_orientation ?? "horizontal",
        candidate_font_size: config.candidate_font_size ?? 14,
        active_theme: config.active_theme ?? "dark-glass",
        mode_toggle_key: config.mode_toggle_key ?? "F12",
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
        formData.max_dictionary_suggestions !== config.max_dictionary_suggestions ||
        formData.auto_convert_period_to_dari !== config.auto_convert_period_to_dari ||
        formData.candidate_orientation !== config.candidate_orientation ||
        formData.candidate_font_size !== config.candidate_font_size ||
        formData.active_theme !== config.active_theme ||
        formData.mode_toggle_key !== config.mode_toggle_key)
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
    } else {
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
      setSaveSuccessMsg("Configuration restored to factory defaults.");
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

  const handleExportClick = async () => {
    try {
      setErrorMessage(null);
      const jsonStr = onExport ? await onExport() : JSON.stringify(formData, null, 2);
      const blob = new Blob([jsonStr], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `bornomala-config-v${formData.schema_version}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setSaveSuccessMsg("Settings exported successfully.");
      setTimeout(() => setSaveSuccessMsg(null), 3000);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(`Export failed: ${msg}`);
    }
  };

  const handleImportFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (event) => {
      const content = event.target?.result as string;
      if (!content) return;
      try {
        setErrorMessage(null);
        if (onImport) {
          const imported = await onImport(content);
          setFormData(imported);
        } else {
          const parsed = JSON.parse(content);
          setFormData(parsed);
        }
        setSaveSuccessMsg("Configuration imported and verified successfully!");
        setTimeout(() => setSaveSuccessMsg(null), 4000);
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        setErrorMessage(`Import error: ${msg}`);
      }
    };
    reader.readAsText(file);
    // Reset file input so re-selecting same file fires change
    e.target.value = "";
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-heading"
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
          width: "500px",
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
            padding: "18px 24px",
            borderBottom: "1px solid var(--border-subtle)",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "22px" }}>⚙️</span>
            <div>
              <h2
                id="settings-heading"
                style={{ fontSize: "17px", fontWeight: "700", color: "var(--text-primary)" }}
              >
                Application Settings
              </h2>
              <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "2px" }}>
                Configuration & Customization (Schema v1.2.0)
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

        {/* Tab Navigation */}
        <div
          role="tablist"
          style={{
            display: "flex",
            borderBottom: "1px solid var(--border-subtle)",
            backgroundColor: "rgba(0, 0, 0, 0.2)",
            padding: "4px 12px 0 12px",
            gap: "4px",
          }}
        >
          {[
            { id: "typing", label: "Typing & Punctuation", icon: "⌨️" },
            { id: "appearance", label: "Appearance", icon: "🎨" },
            { id: "shortcuts", label: "Shortcuts", icon: "⚡" },
            { id: "backup", label: "Backup & Diagnostics", icon: "💾" },
          ].map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                role="tab"
                aria-selected={isActive}
                onClick={() => setActiveTab(tab.id as SettingsTab)}
                style={{
                  flex: 1,
                  padding: "10px 8px",
                  fontSize: "12px",
                  fontWeight: isActive ? "600" : "500",
                  color: isActive ? "var(--accent-cyan)" : "var(--text-secondary)",
                  backgroundColor: isActive ? "var(--bg-secondary)" : "transparent",
                  borderTop: isActive ? "2px solid var(--accent-cyan)" : "2px solid transparent",
                  borderLeft: "none",
                  borderRight: "none",
                  borderBottom: "none",
                  borderRadius: "6px 6px 0 0",
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: "6px",
                  transition: "all 0.15s ease",
                  outline: "none",
                }}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            );
          })}
        </div>

        {/* Content Body */}
        <form
          onSubmit={handleSave}
          style={{
            padding: "20px 24px",
            flex: 1,
            display: "flex",
            flexDirection: "column",
            gap: "20px",
          }}
        >
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
              Saving is locked to preserve your configuration. Resetting will restore v1.2.0 defaults (with automatic backup).
            </div>
          )}

          {/* TAB 1: TYPING & PUNCTUATION */}
          {activeTab === "typing" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
              <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-cyan)" }}>
                Punctuation & Auto-Conversion
              </h3>

              {/* Auto-Dari on single period */}
              <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "14px" }}>
                <div>
                  <label
                    htmlFor="setting-auto-dari"
                    style={{ display: "block", fontSize: "13px", fontWeight: "600", color: "var(--text-primary)", cursor: "pointer" }}
                  >
                    Auto-Convert Single Period to Dáñṛi (।)
                  </label>
                  <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px", lineHeight: "1.4" }}>
                    When enabled, typing a single period at the end of a sentence converts to Bengali Dáñṛi (।) on space or Enter.
                    Numeric values (<code>3.14</code>, <code>৩.১৪</code>) and domain strings (<code>borno.org</code>) are preserved as literal ASCII periods.
                  </p>
                  <div style={{ marginTop: "6px", fontSize: "11px", color: "var(--text-secondary)", backgroundColor: "rgba(0,0,0,0.2)", padding: "4px 8px", borderRadius: "4px" }}>
                    <strong>Note:</strong> Double period (<code>..</code>) always produces Dáñṛi (।) and quadruple period (<code>....</code>) produces Double Dáñṛi (॥) regardless of this setting.
                  </div>
                </div>
                <button
                  id="setting-auto-dari"
                  type="button"
                  role="switch"
                  aria-checked={formData.auto_convert_period_to_dari}
                  onClick={() => setFormData({ ...formData, auto_convert_period_to_dari: !formData.auto_convert_period_to_dari })}
                  style={{
                    width: "44px",
                    height: "24px",
                    borderRadius: "12px",
                    backgroundColor: formData.auto_convert_period_to_dari ? "var(--accent-emerald)" : "rgba(255, 255, 255, 0.15)",
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
                      left: formData.auto_convert_period_to_dari ? "23px" : "3px",
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

              {/* Section Divider */}
              <div style={{ borderTop: "1px solid var(--border-subtle)", paddingTop: "14px", display: "flex", flexDirection: "column", gap: "16px" }}>
                <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-indigo)" }}>
                  Dictionary & Predictive Suggestions
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
                      Enables offline lexicon lookups, casual vowel ambiguity (e.g. <code>tomake</code> → <code>তোমাকে</code> in Slot 1), and ণ-ত্ব বিধান (<code>bornomala</code> → <code>বর্ণমালা</code>).
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
                      Suggests full dictionary completions (e.g. &apos;বাং&apos; → বাংলাদেশ) for inputs with 2+ characters.
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

                {/* Candidate Display Limit Slider */}
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
                    aria-valuemin={1}
                    aria-valuemax={9}
                    aria-valuenow={formData.candidate_display_limit}
                    aria-valuetext={`${formData.candidate_display_limit} candidates`}
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
                    <span>5 (Default)</span>
                    <span>9 (Max)</span>
                  </div>
                  <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                    Slices presentation window capacity without perturbing linguistic ranking.
                  </p>
                </div>

                {/* Max Dictionary Suggestions Slider */}
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
                    aria-valuemin={1}
                    aria-valuemax={5}
                    aria-valuenow={formData.max_dictionary_suggestions}
                    aria-valuetext={`${formData.max_dictionary_suggestions} suggestions`}
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
                    <span>3 (Default)</span>
                    <span>5 (Max)</span>
                  </div>
                  <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                    Caps secondary ambiguity and completion matches. Candidate Slot 0 is strictly immutable.
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: APPEARANCE & THEMES */}
          {activeTab === "appearance" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
              <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-cyan)" }}>
                Visual Theme & Styling
              </h3>

              {/* Theme Selection Cards */}
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "10px" }}>
                {[
                  {
                    id: "dark-glass" as ThemeId,
                    label: "Dark Glass",
                    desc: "Slate & Cyan",
                    bg: "#0f172a",
                    border: "rgba(56, 189, 248, 0.4)",
                    text: "#f8fafc",
                  },
                  {
                    id: "light-glass" as ThemeId,
                    label: "Light Glass",
                    desc: "Pearl & Azure",
                    bg: "#f8fafc",
                    border: "rgba(2, 132, 199, 0.4)",
                    text: "#0f172a",
                  },
                  {
                    id: "high-contrast" as ThemeId,
                    label: "High Contrast",
                    desc: "Pure Black & Yellow",
                    bg: "#000000",
                    border: "#facc15",
                    text: "#ffffff",
                  },
                ].map((t) => {
                  const isSelected = formData.active_theme === t.id;
                  return (
                    <button
                      key={t.id}
                      type="button"
                      onClick={() => setFormData({ ...formData, active_theme: t.id })}
                      style={{
                        padding: "12px 8px",
                        borderRadius: "8px",
                        backgroundColor: t.bg,
                        border: isSelected ? `2px solid ${t.border}` : "1px solid var(--border-subtle)",
                        boxShadow: isSelected ? "0 0 12px rgba(56, 189, 248, 0.35)" : "none",
                        cursor: "pointer",
                        display: "flex",
                        flexDirection: "column",
                        alignItems: "center",
                        gap: "6px",
                        transition: "all 0.15s ease",
                      }}
                    >
                      <span style={{ fontSize: "12px", fontWeight: "700", color: t.text }}>
                        {t.label}
                      </span>
                      <span style={{ fontSize: "10px", color: isSelected ? "var(--accent-cyan)" : "var(--text-muted)" }}>
                        {t.desc}
                      </span>
                      {isSelected && (
                        <span style={{ fontSize: "10px", fontWeight: "700", color: "var(--accent-emerald)" }}>
                          ✓ Active
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>

              {/* Candidate Font Size Slider */}
              <div style={{ marginTop: "10px" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                  <label
                    htmlFor="setting-font-size"
                    style={{ fontSize: "13px", fontWeight: "600", color: "var(--text-primary)" }}
                  >
                    Candidate Font Size
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
                    {formData.candidate_font_size} pt
                  </span>
                </div>
                <input
                  id="setting-font-size"
                  type="range"
                  min={10}
                  max={24}
                  aria-valuemin={10}
                  aria-valuemax={24}
                  aria-valuenow={formData.candidate_font_size}
                  aria-valuetext={`${formData.candidate_font_size} points`}
                  value={formData.candidate_font_size}
                  onChange={(e) => setFormData({ ...formData, candidate_font_size: parseInt(e.target.value, 10) })}
                  style={{
                    width: "100%",
                    accentColor: "var(--accent-cyan)",
                    cursor: "pointer",
                  }}
                />
                <div style={{ display: "flex", justifyContent: "space-between", fontSize: "10px", color: "var(--text-muted)", marginTop: "2px" }}>
                  <span>10 pt (Compact)</span>
                  <span>14 pt (Default)</span>
                  <span>24 pt (Large)</span>
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "4px" }}>
                  Scales candidate tile font size. In Windows TSF, mapped via Win32 <code>MulDiv</code>.
                </p>
              </div>

              {/* Candidate Orientation Radio Group */}
              <div style={{ marginTop: "10px" }}>
                <label style={{ display: "block", fontSize: "13px", fontWeight: "600", marginBottom: "8px", color: "var(--text-primary)" }}>
                  Candidate Window Orientation
                </label>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                  {[
                    { id: "horizontal" as CandidateOrientation, label: "Horizontal Bar", desc: "Inline horizontal tiles (Default)" },
                    { id: "vertical" as CandidateOrientation, label: "Vertical Stack", desc: "Traditional stacked list" },
                  ].map((o) => {
                    const isSelected = formData.candidate_orientation === o.id;
                    return (
                      <button
                        key={o.id}
                        type="button"
                        onClick={() => setFormData({ ...formData, candidate_orientation: o.id })}
                        style={{
                          padding: "10px 12px",
                          borderRadius: "8px",
                          backgroundColor: isSelected ? "rgba(14, 165, 233, 0.2)" : "rgba(11, 15, 25, 0.7)",
                          border: isSelected ? "1px solid var(--accent-cyan)" : "1px solid var(--border-subtle)",
                          cursor: "pointer",
                          textAlign: "left",
                        }}
                      >
                        <div style={{ fontSize: "13px", fontWeight: "600", color: isSelected ? "var(--accent-cyan)" : "var(--text-primary)" }}>
                          {o.label}
                        </div>
                        <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                          {o.desc}
                        </div>
                      </button>
                    );
                  })}
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "6px" }}>
                  <strong>Linux Fcitx 5 Note:</strong> Candidate appearance and layout orientation are host-controlled by your desktop Fcitx 5 theme.
                </p>
              </div>
            </div>
          )}

          {/* TAB 3: SHORTCUTS & KEYBOARDS */}
          {activeTab === "shortcuts" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
              <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-cyan)" }}>
                Keyboard Shortcuts & Layouts
              </h3>

              {/* Mode Toggle Hotkey Selector */}
              <div>
                <label style={{ display: "block", fontSize: "13px", fontWeight: "600", marginBottom: "8px", color: "var(--text-primary)" }}>
                  Primary Input Mode Toggle Key
                </label>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                  {[
                    { id: "F12" as ModeToggleKey, label: "F12 Key (Default)", desc: "Standard Avro toggle shortcut" },
                    { id: "F4" as ModeToggleKey, label: "F4 Key", desc: "Compact layout & media key friendly" },
                  ].map((k) => {
                    const isSelected = formData.mode_toggle_key === k.id;
                    return (
                      <button
                        key={k.id}
                        type="button"
                        onClick={() => setFormData({ ...formData, mode_toggle_key: k.id })}
                        style={{
                          padding: "10px 12px",
                          borderRadius: "8px",
                          backgroundColor: isSelected ? "rgba(14, 165, 233, 0.2)" : "rgba(11, 15, 25, 0.7)",
                          border: isSelected ? "1px solid var(--accent-cyan)" : "1px solid var(--border-subtle)",
                          cursor: "pointer",
                          textAlign: "left",
                        }}
                      >
                        <div style={{ fontSize: "13px", fontWeight: "600", color: isSelected ? "var(--accent-cyan)" : "var(--text-primary)" }}>
                          {k.label}
                        </div>
                        <div style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "2px" }}>
                          {k.desc}
                        </div>
                      </button>
                    );
                  })}
                </div>
                <p style={{ fontSize: "11px", color: "var(--text-muted)", marginTop: "6px" }}>
                  <strong>Laptop Advisory:</strong> Some laptop keyboards require pressing the <code>Fn</code> key (e.g. <code>Fn+F12</code>) unless Fn lock is active.
                </p>
              </div>

              {/* Preferred Fixed Layout */}
              <div style={{ marginTop: "6px" }}>
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
                  Preferred layout applied when switching into Fixed mode. Active preedit is never disrupted.
                </p>
              </div>

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
                  Applied during initial cold startup.
                </p>
              </div>

              {/* Auto-commit on Mode Switch */}
              <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "12px", paddingTop: "4px" }}>
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
          )}

          {/* TAB 4: BACKUP & DIAGNOSTICS */}
          {activeTab === "backup" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
              <h3 style={{ fontSize: "13px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--accent-cyan)" }}>
                Export, Import & Recovery
              </h3>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px" }}>
                {/* Export Button */}
                <button
                  type="button"
                  onClick={handleExportClick}
                  style={{
                    padding: "10px 14px",
                    borderRadius: "8px",
                    backgroundColor: "rgba(14, 165, 233, 0.15)",
                    color: "var(--accent-cyan)",
                    border: "1px solid rgba(14, 165, 233, 0.35)",
                    fontSize: "12px",
                    fontWeight: "600",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: "8px",
                  }}
                >
                  <span>📤</span>
                  <span>Export Settings (JSON)</span>
                </button>

                {/* Import Button */}
                <button
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  style={{
                    padding: "10px 14px",
                    borderRadius: "8px",
                    backgroundColor: "rgba(99, 102, 241, 0.15)",
                    color: "var(--accent-indigo)",
                    border: "1px solid rgba(99, 102, 241, 0.35)",
                    fontSize: "12px",
                    fontWeight: "600",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: "8px",
                  }}
                >
                  <span>📥</span>
                  <span>Import Settings (JSON)</span>
                </button>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".json,application/json"
                  style={{ display: "none" }}
                  onChange={handleImportFileChange}
                />
              </div>

              {/* Reset to Factory Defaults */}
              <div style={{ marginTop: "4px" }}>
                <button
                  id="settings-reset-btn"
                  type="button"
                  onClick={() => setShowResetConfirm(true)}
                  disabled={isResetting}
                  style={{
                    width: "100%",
                    padding: "10px 14px",
                    borderRadius: "8px",
                    backgroundColor: "rgba(244, 63, 94, 0.12)",
                    color: "var(--accent-rose)",
                    border: "1px solid rgba(244, 63, 94, 0.3)",
                    fontWeight: "600",
                    fontSize: "12px",
                    cursor: isResetting ? "not-allowed" : "pointer",
                    transition: "all 0.2s ease",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: "8px",
                  }}
                >
                  <span>🔄</span>
                  <span>Restore Factory Defaults</span>
                </button>
              </div>

              {/* Storage Diagnostics Info Box */}
              <div style={{ marginTop: "12px", paddingTop: "14px", borderTop: "1px solid var(--border-subtle)" }}>
                <h4 style={{ fontSize: "12px", fontWeight: "700", textTransform: "uppercase", letterSpacing: "0.05em", color: "var(--text-secondary)", marginBottom: "8px" }}>
                  Storage & Diagnostics
                </h4>

                <div style={{ backgroundColor: "rgba(11, 15, 25, 0.7)", padding: "12px", borderRadius: "8px", border: "1px solid var(--border-subtle)", display: "flex", flexDirection: "column", gap: "8px" }}>
                  <div>
                    <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>
                      Schema Version:
                    </div>
                    <div style={{ fontSize: "12px", fontWeight: "600", color: "var(--accent-cyan)", fontFamily: "var(--font-mono)" }}>
                      v{formData.schema_version} (Whitelisted: 1.0.0, 1.1.0, 1.2.0)
                    </div>
                  </div>

                  <div>
                    <div style={{ fontSize: "11px", color: "var(--text-muted)", marginBottom: "2px" }}>
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
                  </div>

                  <div style={{ display: "flex", alignItems: "center", gap: "8px", marginTop: "2px" }}>
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
            </div>
          )}

          {/* Action Bar */}
          <div
            style={{
              marginTop: "auto",
              paddingTop: "18px",
              borderTop: "1px solid var(--border-subtle)",
              display: "flex",
              gap: "10px",
            }}
          >
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
                  ? `Warning: The configuration file on disk has an unsupported version (v${unsupportedVersion}). Resetting will overwrite this file with v1.2.0 defaults (a backup will be created). Are you sure you want to proceed?`
                  : "Are you sure you want to reset all settings to defaults? Your current preferences on disk and in memory will be overwritten with v1.2.0 defaults."}
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
