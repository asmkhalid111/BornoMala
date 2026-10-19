export type InputMode =
  | "Phonetic"
  | "EnglishPassthrough"
  | { Fixed: "National" | "Probhat" | "Bijoy" };

export type StartupMode = "Phonetic" | "Fixed" | "EnglishPassthrough";

export type FixedLayoutId = "National" | "Probhat" | "Bijoy";

export type CandidateOrientation = "vertical" | "horizontal";

export type ThemeId = "dark-glass" | "light-glass" | "high-contrast";

export type ModeToggleKey = "F12" | "F4";

export interface BornoConfig {
  schema_version: string;
  default_startup_mode: StartupMode;
  preferred_fixed_layout: FixedLayoutId;
  candidate_display_limit: number;
  auto_commit_on_mode_switch: boolean;
  enable_dictionary_assistance: boolean;
  enable_word_completion: boolean;
  max_dictionary_suggestions: number;
  auto_convert_period_to_dari: boolean;
  candidate_orientation: CandidateOrientation;
  candidate_font_size: number;
  active_theme: ThemeId;
  mode_toggle_key: ModeToggleKey;
}

export type PersistenceStatus =
  | { type: "Active" }
  | { type: "ReadOnlyMissingWriteFailed"; payload: string }
  | { type: "ReadOnlyPermissionDenied"; payload: string }
  | { type: "LockedUnsupportedVersion"; payload: { version: string } };

export interface ConfigResponse {
  config: BornoConfig;
  status: PersistenceStatus;
  config_path: string;
  backup_created?: string | null;
}

export type PhysicalKey =
  // Row E
  | "Backquote"
  | "Digit1"
  | "Digit2"
  | "Digit3"
  | "Digit4"
  | "Digit5"
  | "Digit6"
  | "Digit7"
  | "Digit8"
  | "Digit9"
  | "Digit0"
  | "Minus"
  | "Equal"
  // Row D
  | "KeyQ"
  | "KeyW"
  | "KeyE"
  | "KeyR"
  | "KeyT"
  | "KeyY"
  | "KeyU"
  | "KeyI"
  | "KeyO"
  | "KeyP"
  | "BracketLeft"
  | "BracketRight"
  | "Backslash"
  // Row C
  | "KeyA"
  | "KeyS"
  | "KeyD"
  | "KeyF"
  | "KeyG"
  | "KeyH"
  | "KeyJ"
  | "KeyK"
  | "KeyL"
  | "Semicolon"
  | "Quote"
  // Row B
  | "KeyZ"
  | "KeyX"
  | "KeyC"
  | "KeyV"
  | "KeyB"
  | "KeyN"
  | "KeyM"
  | "Comma"
  | "Period"
  | "Slash"
  // Row A
  | "Space";

export interface Modifiers {
  shift: boolean;
  altgr: boolean;
  ctrl: boolean;
  alt: boolean;
}

export interface KeyMapping {
  unshifted: string;
  shifted: string;
  altgr?: string | null;
}

export interface LayoutMetadata {
  id: string;
  name: string;
  version: string;
  standard?: string | null;
  author: string;
  description: string;
  supported_layers: ("Normal" | "Shift" | "AltGr")[];
}

export interface LayoutRules {
  link_key?: PhysicalKey | null;
  supports_link_vowel_composition: boolean;
  link_vowel_map: Record<string, string>;
}

export interface LayoutDefinition {
  schema_version: string;
  metadata: LayoutMetadata;
  rules: LayoutRules;
  keys: Record<PhysicalKey, KeyMapping>;
}

export interface EngineStatus {
  name: string;
  version: string;
  active_mode: string;
  ready: boolean;
}

export type EngineEvent =
  | { type: "InputKey"; payload: { key: PhysicalKey; modifiers: Modifiers } }
  | { type: "InputChar"; payload: string }
  | { type: "InputString"; payload: string }
  | { type: "Backspace" }
  | { type: "Delete" }
  | { type: "Commit" }
  | { type: "Cancel" }
  | { type: "Reset" }
  | { type: "NextCandidate" }
  | { type: "PrevCandidate" }
  | { type: "HighlightCandidate"; payload: number }
  | { type: "SelectCandidate"; payload: number }
  | { type: "MoveCursorLeft" }
  | { type: "MoveCursorRight" }
  | { type: "SetCursor"; payload: number }
  | { type: "SwitchMode"; payload: InputMode };

export type CandidateKind =
  | "PrimaryTransliteration"
  | "PhoneticAlternative"
  | "ExactLexiconMatch"
  | "PrefixCompletion"
  | "SpellingCorrection";

export interface CandidateDetail {
  text: string;
  kind: CandidateKind;
  score: number;
}

export interface EngineState {
  preedit: string;
  raw_buffer: string;
  cursor_pos: number;
  candidates: string[];
  candidate_details?: CandidateDetail[];
  selected_candidate_index: number | null;
  is_composing: boolean;
  mode: InputMode;
  status: EngineStatus;
}

export interface ProcessResult {
  state: EngineState;
  committed_text: string | null;
  cancelled: boolean;
  handled: boolean;
}

export interface DeterministicTestResult {
  success: boolean;
  input_sequence: string;
  expected_preedit: string;
  actual_preedit: string;
  candidates: string[];
  message: string;
}

export interface EventLogItem {
  id: string;
  time: string;
  action: string;
  rawBuffer: string;
  preedit: string;
  cursorPos: number;
  committed: string | null;
  cancelled: boolean;
  latencyMs: number;
}
