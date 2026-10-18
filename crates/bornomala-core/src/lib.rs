//! # Borno Core
//!
//! Shared input and language engine for the Bangla Input Platform.
//! Provides normalized event handling, composition state machine,
//! and candidate generation.

pub mod translit;
pub mod unicode;
pub mod phonetic;
pub mod layout;
pub mod ffi;
pub mod config;
pub mod dictionary;

pub use config::{BornoConfig, ConfigValidationError, StartupMode};
pub use dictionary::DictionaryEngine;

use serde::{Deserialize, Serialize};

/// Supported engine input modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InputMode {
    #[default]
    Phonetic,
    EnglishPassthrough,
    Fixed(layout::FixedLayoutId),
}

impl InputMode {
    pub fn active_mode_name(&self) -> &'static str {
        match self {
            InputMode::Phonetic => "Phonetic",
            InputMode::EnglishPassthrough => "English Passthrough",
            InputMode::Fixed(layout::FixedLayoutId::National) => "National (Jatiya)",
            InputMode::Fixed(layout::FixedLayoutId::Probhat) => "Probhat",
            InputMode::Fixed(layout::FixedLayoutId::Bijoy) => "Bijoy-Compatible",
        }
    }
}

/// High-level engine operational status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineStatus {
    pub name: String,
    pub version: String,
    pub active_mode: String,
    pub ready: bool,
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self {
            name: "Borno Core".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            active_mode: "Phonetic".to_string(),
            ready: true,
        }
    }
}

/// Normalized input events dispatched into the engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum EngineEvent {
    /// Physical hardware key event (Primary for M4 Fixed Layouts)
    InputKey {
        key: layout::PhysicalKey,
        modifiers: layout::Modifiers,
    },
    /// Printable character entered
    InputChar(char),
    /// Raw string input (e.g. paste or bulk input)
    InputString(String),
    /// Deletion of the input unit preceding the cursor
    Backspace,
    /// Deletion of the input unit following the cursor
    Delete,
    /// Explicit request to commit the current composition
    Commit,
    /// Explicit cancellation of current composition (e.g. Escape)
    Cancel,
    /// Administrative reset of engine state (e.g. focus loss, window close)
    Reset,
    /// Move candidate highlight to next candidate (with wrap-around)
    NextCandidate,
    /// Move candidate highlight to previous candidate (with wrap-around)
    PrevCandidate,
    /// Highlight candidate at specific 0-based index without committing
    HighlightCandidate(usize),
    /// Select and immediately commit candidate at 0-based index
    SelectCandidate(usize),
    /// Move composition cursor one codepoint to the left
    MoveCursorLeft,
    /// Move composition cursor one codepoint to the right
    MoveCursorRight,
    /// Set composition cursor to an explicit codepoint index
    SetCursor(usize),
    /// Switch input mode
    SwitchMode(InputMode),
}

/// Classification of candidate origin and assistance type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateKind {
    /// Deterministic primary transliteration from M3 (Slot 0 Invariant).
    PrimaryTransliteration,
    /// Rule-based phonetic alternation from M3 (e.g. high-vowel alternate).
    PhoneticAlternative,
    /// Exact match verified in the lexicon trie (e.g. from ambiguity rewrite).
    ExactLexiconMatch,
    /// Prefix completion suggested from the lexicon trie.
    PrefixCompletion,
    /// Bounded edit-distance spelling suggestion for unmatched tokens.
    SpellingCorrection,
}

/// Metadata and classification detail for a candidate entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDetail {
    pub text: String,
    pub kind: CandidateKind,
    pub score: u32,
}

/// Snapshot of the engine's persistent ongoing composition state.
/// Notice: committed text is explicitly excluded from persistent state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EngineState {
    /// Active preedit / composition string in Bangla
    pub preedit: String,
    /// Raw uncommitted Latin keystroke buffer
    pub raw_buffer: String,
    /// Composition cursor position measured in Unicode codepoints (0 <= cursor_pos <= preedit.chars().count())
    pub cursor_pos: usize,
    /// Ranked candidate list for the current composition
    pub candidates: Vec<String>,
    /// Parallel candidate metadata list maintaining 1:1 lockstep with candidates
    pub candidate_details: Vec<CandidateDetail>,
    /// Currently highlighted candidate index (None if none highlighted)
    pub selected_candidate_index: Option<usize>,
    /// True if an active composition is in progress
    pub is_composing: bool,
    /// Current input mode
    pub mode: InputMode,
    /// Current engine status
    pub status: EngineStatus,
}

#[derive(Deserialize)]
struct EngineStateHelper {
    pub preedit: String,
    pub raw_buffer: String,
    pub cursor_pos: usize,
    pub candidates: Vec<String>,
    #[serde(default)]
    pub candidate_details: Vec<CandidateDetail>,
    pub selected_candidate_index: Option<usize>,
    pub is_composing: bool,
    pub mode: InputMode,
    pub status: EngineStatus,
}

impl<'de> Deserialize<'de> for EngineState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = EngineStateHelper::deserialize(deserializer)?;
        let mut candidate_details = helper.candidate_details;
        // Enforce lockstep invariant: candidates.len() == candidate_details.len()
        if candidate_details.len() != helper.candidates.len() {
            candidate_details = helper
                .candidates
                .iter()
                .enumerate()
                .map(|(i, s)| CandidateDetail {
                    text: s.clone(),
                    kind: if i == 0 {
                        CandidateKind::PrimaryTransliteration
                    } else {
                        CandidateKind::PhoneticAlternative
                    },
                    score: 0,
                })
                .collect();
        }
        Ok(EngineState {
            preedit: helper.preedit,
            raw_buffer: helper.raw_buffer,
            cursor_pos: helper.cursor_pos,
            candidates: helper.candidates,
            candidate_details,
            selected_candidate_index: helper.selected_candidate_index,
            is_composing: helper.is_composing,
            mode: helper.mode,
            status: helper.status,
        })
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            preedit: String::new(),
            raw_buffer: String::new(),
            cursor_pos: 0,
            candidates: Vec::new(),
            candidate_details: Vec::new(),
            selected_candidate_index: None,
            is_composing: false,
            mode: InputMode::Phonetic,
            status: EngineStatus::default(),
        }
    }
}

/// Ephemeral output effect produced by processing a single engine event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessResult {
    /// Snapshot of the updated persistent engine state
    pub state: EngineState,
    /// Text committed during this specific event step, if any
    pub committed_text: Option<String>,
    /// True if an active composition was aborted/cancelled during this step
    pub cancelled: bool,
    /// True if the engine consumed/handled the event; false if it should pass through to host OS
    pub handled: bool,
}

/// Core stateful input engine instance.
pub struct BornoEngine {
    state: EngineState,
    spans: Vec<phonetic::TokenSpan>,
    source_cursor: usize,
    fixed_engine: layout::FixedLayoutEngine,
    candidate_display_limit: usize,
    auto_commit_on_mode_switch: bool,
    preferred_fixed_layout: layout::FixedLayoutId,
    enable_dictionary_assistance: bool,
    enable_word_completion: bool,
    max_dictionary_suggestions: usize,
}

impl BornoEngine {
    /// Creates a new, ready BornoEngine instance with default preferences.
    pub fn new() -> Self {
        Self {
            state: EngineState::default(),
            spans: Vec::new(),
            source_cursor: 0,
            fixed_engine: layout::FixedLayoutEngine::new(layout::get_national_layout()),
            candidate_display_limit: 5,
            auto_commit_on_mode_switch: true,
            preferred_fixed_layout: layout::FixedLayoutId::National,
            enable_dictionary_assistance: true,
            enable_word_completion: true,
            max_dictionary_suggestions: 3,
        }
    }

    /// Creates a new BornoEngine initialized with preferences from a BornoConfig.
    pub fn with_config(config: &BornoConfig) -> Self {
        let mut state = EngineState::default();
        let fixed_layout_def = layout::get_layout_by_id(config.preferred_fixed_layout);
        let fixed_engine = layout::FixedLayoutEngine::new(fixed_layout_def);

        match config.default_startup_mode {
            StartupMode::Phonetic => {
                state.mode = InputMode::Phonetic;
                state.status.active_mode = "Phonetic".to_string();
            }
            StartupMode::Fixed => {
                state.mode = InputMode::Fixed(config.preferred_fixed_layout);
                state.status.active_mode = state.mode.active_mode_name().to_string();
            }
            StartupMode::EnglishPassthrough => {
                state.mode = InputMode::EnglishPassthrough;
                state.status.active_mode = "English Passthrough".to_string();
            }
        }

        Self {
            state,
            spans: Vec::new(),
            source_cursor: 0,
            fixed_engine,
            candidate_display_limit: config.candidate_display_limit.clamp(1, 9),
            auto_commit_on_mode_switch: config.auto_commit_on_mode_switch,
            preferred_fixed_layout: config.preferred_fixed_layout,
            enable_dictionary_assistance: config.enable_dictionary_assistance,
            enable_word_completion: config.enable_word_completion,
            max_dictionary_suggestions: config.max_dictionary_suggestions.clamp(1, 5),
        }
    }

    /// Sets the maximum number of candidates presented in candidate window.
    pub fn set_candidate_display_limit(&mut self, limit: usize) {
        self.candidate_display_limit = limit.clamp(1, 9);
        if self.state.candidates.len() > self.candidate_display_limit {
            self.state.candidates.truncate(self.candidate_display_limit);
            self.state.candidate_details.truncate(self.candidate_display_limit);
        }
    }

    /// Returns the active candidate display limit.
    pub fn candidate_display_limit(&self) -> usize {
        self.candidate_display_limit
    }

    /// Sets whether switching modes automatically commits active preedit text in NFC.
    pub fn set_auto_commit_on_mode_switch(&mut self, enabled: bool) {
        self.auto_commit_on_mode_switch = enabled;
    }

    /// Returns whether mode switching auto-commits preedit text.
    pub fn auto_commit_on_mode_switch(&self) -> bool {
        self.auto_commit_on_mode_switch
    }

    /// Sets the preferred fixed layout preference for future Fixed mode entries.
    pub fn set_preferred_fixed_layout(&mut self, layout: layout::FixedLayoutId) {
        self.preferred_fixed_layout = layout;
    }

    /// Returns the preferred fixed layout preference.
    pub fn preferred_fixed_layout(&self) -> layout::FixedLayoutId {
        self.preferred_fixed_layout
    }

    /// Returns whether dictionary assistance is enabled.
    pub fn enable_dictionary_assistance(&self) -> bool {
        self.enable_dictionary_assistance
    }

    /// Sets whether dictionary assistance is enabled.
    pub fn set_enable_dictionary_assistance(&mut self, enabled: bool) {
        self.enable_dictionary_assistance = enabled;
    }

    /// Returns whether word completion is enabled.
    pub fn enable_word_completion(&self) -> bool {
        self.enable_word_completion
    }

    /// Sets whether word completion is enabled.
    pub fn set_enable_word_completion(&mut self, enabled: bool) {
        self.enable_word_completion = enabled;
    }

    /// Returns maximum dictionary suggestions allowed.
    pub fn max_dictionary_suggestions(&self) -> usize {
        self.max_dictionary_suggestions
    }

    /// Sets maximum dictionary suggestions allowed (1..=5).
    pub fn set_max_dictionary_suggestions(&mut self, limit: usize) {
        self.max_dictionary_suggestions = limit.clamp(1, 5);
    }

    /// Returns a reference to the active token spans.
    pub fn spans(&self) -> &[phonetic::TokenSpan] {
        &self.spans
    }

    /// Returns the current engine status.
    pub fn status(&self) -> EngineStatus {
        self.state.status.clone()
    }

    /// Returns a reference to the current persistent engine state.
    pub fn state(&self) -> &EngineState {
        &self.state
    }

    /// Resets the engine composition state.
    pub fn reset(&mut self) -> ProcessResult {
        self.process_event(EngineEvent::Reset)
    }

    /// Commits the active preedit composition.
    pub fn commit(&mut self) -> ProcessResult {
        self.process_event(EngineEvent::Commit)
    }

    /// Processes a single normalized engine event and returns the resulting state and output effects.
    pub fn process_event(&mut self, event: EngineEvent) -> ProcessResult {
        match self.state.mode {
            InputMode::EnglishPassthrough => self.process_passthrough_event(event),
            InputMode::Phonetic => self.process_phonetic_event(event),
            InputMode::Fixed(id) => self.process_fixed_event(event, id),
        }
    }

    /// Common mode switch handler honoring the auto_commit_on_mode_switch setting.
    fn handle_switch_mode(&mut self, m: InputMode) -> ProcessResult {
        let committed = if self.auto_commit_on_mode_switch
            && self.state.is_composing
            && !self.state.preedit.is_empty()
            && self.state.preedit != "্"
        {
            Some(unicode::normalize_nfc(&self.state.preedit))
        } else {
            None
        };
        self.clear_composition();
        self.fixed_engine.reset_state();
        self.state.mode = m;
        self.state.status.active_mode = m.active_mode_name().to_string();
        if let InputMode::Fixed(id) = m {
            self.fixed_engine.set_layout(layout::get_layout_by_id(id));
        }
        ProcessResult {
            state: self.state.clone(),
            committed_text: committed,
            cancelled: false,
            handled: true,
        }
    }

    /// Event processing when in English Passthrough mode.
    fn process_passthrough_event(&mut self, event: EngineEvent) -> ProcessResult {
        match event {
            EngineEvent::InputKey { key, modifiers } => {
                if (modifiers.ctrl || modifiers.alt) && !modifiers.altgr {
                    return ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    };
                }
                if key == layout::PhysicalKey::Space {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: Some(" ".to_string()),
                        cancelled: false,
                        handled: true,
                    }
                } else if let Some(ch) = physical_key_to_ascii(key, modifiers.shift) {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: Some(ch.to_string()),
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::InputChar(c) => ProcessResult {
                state: self.state.clone(),
                committed_text: Some(c.to_string()),
                cancelled: false,
                handled: true,
            },
            EngineEvent::InputString(s) => ProcessResult {
                state: self.state.clone(),
                committed_text: Some(s),
                cancelled: false,
                handled: true,
            },
            EngineEvent::SwitchMode(m) => self.handle_switch_mode(m),
            EngineEvent::Reset => {
                self.clear_composition();
                self.fixed_engine.reset_state();
                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: true,
                }
            }
            _ => ProcessResult {
                state: self.state.clone(),
                committed_text: None,
                cancelled: false,
                handled: false,
            },
        }
    }

    /// Event processing when in Phonetic mode.
    fn process_phonetic_event(&mut self, event: EngineEvent) -> ProcessResult {
        match event {
            EngineEvent::InputKey { key, modifiers } => {
                if (modifiers.ctrl || modifiers.alt) && !modifiers.altgr {
                    return ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    };
                }
                if modifiers.shift && modifiers.altgr {
                    return ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    };
                }
                if key == layout::PhysicalKey::Space {
                    self.process_phonetic_event(EngineEvent::InputChar(' '))
                } else if let Some(ch) = physical_key_to_ascii(key, modifiers.shift) {
                    self.process_phonetic_event(EngineEvent::InputChar(ch))
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::InputChar(c) => {
                if c == '\n' || c == '\r' {
                    if self.state.is_composing {
                        self.commit_active()
                    } else {
                        ProcessResult {
                            state: self.state.clone(),
                            committed_text: None,
                            cancelled: false,
                            handled: false,
                        }
                    }
                } else if c == ' ' {
                    if self.state.is_composing {
                        let mut res = self.commit_active();
                        if let Some(ref mut text) = res.committed_text {
                            text.push(' ');
                        } else {
                            res.committed_text = Some(" ".to_string());
                        }
                        res
                    } else {
                        ProcessResult {
                            state: self.state.clone(),
                            committed_text: Some(" ".to_string()),
                            cancelled: false,
                            handled: true,
                        }
                    }
                } else {
                    self.insert_char_at_cursor(c);
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                }
            }
            EngineEvent::InputString(s) => {
                for c in s.chars() {
                    self.insert_char_at_cursor(c);
                }
                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: true,
                }
            }
            EngineEvent::Backspace => {
                if !self.state.is_composing || self.state.raw_buffer.is_empty() {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else if self.source_cursor == 0 && self.state.cursor_pos == 0 {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else {
                    self.delete_before_cursor();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                }
            }
            EngineEvent::Delete => {
                if !self.state.is_composing || self.state.raw_buffer.is_empty() || self.source_cursor >= self.state.raw_buffer.len() {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else {
                    self.delete_after_cursor();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                }
            }
            EngineEvent::Commit => {
                if self.state.is_composing {
                    self.commit_active()
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::Cancel => {
                if self.state.is_composing {
                    self.clear_composition();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: true,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::Reset => {
                self.clear_composition();
                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: true,
                }
            }
            EngineEvent::NextCandidate => {
                if self.state.candidates.is_empty() {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else {
                    let len = self.state.candidates.len();
                    self.state.selected_candidate_index = Some(match self.state.selected_candidate_index {
                        Some(idx) => (idx + 1) % len,
                        None => 0,
                    });
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                }
            }
            EngineEvent::PrevCandidate => {
                if self.state.candidates.is_empty() {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else {
                    let len = self.state.candidates.len();
                    self.state.selected_candidate_index = Some(match self.state.selected_candidate_index {
                        Some(idx) => (idx + len - 1) % len,
                        None => len - 1,
                    });
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                }
            }
            EngineEvent::HighlightCandidate(idx) => {
                if idx < self.state.candidates.len() {
                    self.state.selected_candidate_index = Some(idx);
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::SelectCandidate(idx) => {
                if idx < self.state.candidates.len() {
                    let text = self.state.candidates[idx].clone();
                    self.clear_composition();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: Some(text),
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::MoveCursorLeft => {
                if self.state.is_composing {
                    if self.state.cursor_pos > 0 {
                        self.state.cursor_pos -= 1;
                        self.source_cursor = self.map_target_to_source(self.state.cursor_pos);
                    }
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::MoveCursorRight => {
                if self.state.is_composing {
                    let max = self.state.preedit.chars().count();
                    if self.state.cursor_pos < max {
                        self.state.cursor_pos += 1;
                        self.source_cursor = self.map_target_to_source(self.state.cursor_pos);
                    }
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::SetCursor(pos) => {
                if self.state.is_composing {
                    let max = self.state.preedit.chars().count();
                    let clamped = pos.min(max);
                    self.state.cursor_pos = clamped;
                    self.source_cursor = self.map_target_to_source(clamped);
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::SwitchMode(m) => self.handle_switch_mode(m),
        }
    }

    /// Event processing when in Fixed layout mode.
    fn process_fixed_event(&mut self, event: EngineEvent, layout_id: layout::FixedLayoutId) -> ProcessResult {
        if self.fixed_engine.layout().metadata.id != layout_id.id_str() {
            self.fixed_engine.set_layout(layout::get_layout_by_id(layout_id));
        }

        match event {
            EngineEvent::InputKey { key, modifiers } => {
                // Application accelerators (Ctrl / Alt without AltGr)
                if (modifiers.ctrl || modifiers.alt) && !modifiers.altgr {
                    return ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    };
                }

                // Unsupported layer (Shift + AltGr)
                if modifiers.shift && modifiers.altgr {
                    return ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    };
                }

                if key == layout::PhysicalKey::Space {
                    if self.state.is_composing {
                        let committed_base = if self.state.preedit == "্" {
                            String::new()
                        } else {
                            unicode::normalize_nfc(&self.state.preedit)
                        };
                        self.clear_composition();
                        self.fixed_engine.reset_state();
                        let committed = if committed_base.is_empty() {
                            " ".to_string()
                        } else {
                            format!("{committed_base} ")
                        };
                        return ProcessResult {
                            state: self.state.clone(),
                            committed_text: Some(committed),
                            cancelled: false,
                            handled: true,
                        };
                    } else {
                        return ProcessResult {
                            state: self.state.clone(),
                            committed_text: Some(" ".to_string()),
                            cancelled: false,
                            handled: true,
                        };
                    }
                }

                let mapping = match self.fixed_engine.layout().keys.get(&key) {
                    Some(m) => m,
                    None => return ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    },
                };

                let text = if modifiers.altgr {
                    match mapping.altgr.as_deref() {
                        Some(t) => t,
                        None => return ProcessResult {
                            state: self.state.clone(),
                            committed_text: None,
                            cancelled: false,
                            handled: false,
                        },
                    }
                } else if modifiers.shift {
                    &mapping.shifted
                } else {
                    &mapping.unshifted
                };

                self.state.is_composing = true;
                self.fixed_engine.process_keystroke(&mut self.state.preedit, &mut self.state.cursor_pos, key, text);
                if self.state.preedit.is_empty() {
                    self.state.is_composing = false;
                }

                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: true,
                }
            }
            EngineEvent::InputChar(c) => {
                if c == '\n' || c == '\r' {
                    if self.state.is_composing {
                        self.commit_active()
                    } else {
                        ProcessResult {
                            state: self.state.clone(),
                            committed_text: None,
                            cancelled: false,
                            handled: false,
                        }
                    }
                } else if c == ' ' {
                    if self.state.is_composing {
                        let committed_base = if self.state.preedit == "্" {
                            String::new()
                        } else {
                            unicode::normalize_nfc(&self.state.preedit)
                        };
                        self.clear_composition();
                        self.fixed_engine.reset_state();
                        let committed = if committed_base.is_empty() {
                            " ".to_string()
                        } else {
                            format!("{committed_base} ")
                        };
                        ProcessResult {
                            state: self.state.clone(),
                            committed_text: Some(committed),
                            cancelled: false,
                            handled: true,
                        }
                    } else {
                        ProcessResult {
                            state: self.state.clone(),
                            committed_text: Some(" ".to_string()),
                            cancelled: false,
                            handled: true,
                        }
                    }
                } else {
                    self.state.is_composing = true;
                    let text = c.to_string();
                    self.fixed_engine.process_keystroke(&mut self.state.preedit, &mut self.state.cursor_pos, layout::PhysicalKey::Backquote, &text);
                    if self.state.preedit.is_empty() {
                        self.state.is_composing = false;
                    }
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                }
            }
            EngineEvent::InputString(s) => {
                for c in s.chars() {
                    self.state.is_composing = true;
                    let text = c.to_string();
                    self.fixed_engine.process_keystroke(&mut self.state.preedit, &mut self.state.cursor_pos, layout::PhysicalKey::Backquote, &text);
                }
                if self.state.preedit.is_empty() {
                    self.state.is_composing = false;
                }
                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: true,
                }
            }
            EngineEvent::Backspace => {
                if !self.state.is_composing {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else {
                    let handled = self.fixed_engine.handle_backspace(&mut self.state.preedit, &mut self.state.cursor_pos);
                    if self.state.preedit.is_empty() {
                        self.state.is_composing = false;
                    }
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled,
                    }
                }
            }
            EngineEvent::Delete => {
                if !self.state.is_composing {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                } else {
                    let handled = self.fixed_engine.handle_delete(&mut self.state.preedit, &mut self.state.cursor_pos);
                    if self.state.preedit.is_empty() {
                        self.state.is_composing = false;
                    }
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled,
                    }
                }
            }
            EngineEvent::MoveCursorLeft => {
                if self.state.is_composing {
                    if self.state.cursor_pos > 0 {
                        self.state.cursor_pos -= 1;
                    }
                    self.fixed_engine.handle_cursor_movement();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::MoveCursorRight => {
                if self.state.is_composing {
                    let max = self.state.preedit.chars().count();
                    if self.state.cursor_pos < max {
                        self.state.cursor_pos += 1;
                    }
                    self.fixed_engine.handle_cursor_movement();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::SetCursor(pos) => {
                if self.state.is_composing {
                    let max = self.state.preedit.chars().count();
                    self.state.cursor_pos = pos.min(max);
                    self.fixed_engine.handle_cursor_movement();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::Commit => {
                if self.state.is_composing {
                    let text = if self.state.preedit == "্" {
                        None
                    } else if !self.state.preedit.is_empty() {
                        Some(unicode::normalize_nfc(&self.state.preedit))
                    } else {
                        None
                    };
                    self.clear_composition();
                    self.fixed_engine.reset_state();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: text,
                        cancelled: false,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::Cancel => {
                if self.state.is_composing {
                    self.clear_composition();
                    self.fixed_engine.reset_state();
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: true,
                        handled: true,
                    }
                } else {
                    ProcessResult {
                        state: self.state.clone(),
                        committed_text: None,
                        cancelled: false,
                        handled: false,
                    }
                }
            }
            EngineEvent::Reset => {
                self.clear_composition();
                self.fixed_engine.reset_state();
                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: true,
                }
            }
            EngineEvent::SwitchMode(m) => self.handle_switch_mode(m),
            // Candidates are unused in fixed direct layout typing
            EngineEvent::NextCandidate | EngineEvent::PrevCandidate |
            EngineEvent::HighlightCandidate(_) | EngineEvent::SelectCandidate(_) => {
                ProcessResult {
                    state: self.state.clone(),
                    committed_text: None,
                    cancelled: false,
                    handled: false,
                }
            }
        }
    }

    /// Internal commit logic for active composition.
    fn commit_active(&mut self) -> ProcessResult {
        let text = if let Some(idx) = self.state.selected_candidate_index {
            if idx < self.state.candidates.len() {
                self.state.candidates[idx].clone()
            } else if !self.state.candidates.is_empty() {
                self.state.candidates[0].clone()
            } else {
                self.state.preedit.clone()
            }
        } else if !self.state.candidates.is_empty() {
            self.state.candidates[0].clone()
        } else {
            self.state.preedit.clone()
        };

        let committed = if text.is_empty() {
            None
        } else {
            Some(unicode::normalize_nfc(&text))
        };

        self.clear_composition();
        ProcessResult {
            state: self.state.clone(),
            committed_text: committed,
            cancelled: false,
            handled: true,
        }
    }

    /// Clears composition buffers and restores Idle state.
    fn clear_composition(&mut self) {
        self.state.preedit.clear();
        self.state.raw_buffer.clear();
        self.state.cursor_pos = 0;
        self.source_cursor = 0;
        self.state.candidates.clear();
        self.state.candidate_details.clear();
        self.state.selected_candidate_index = None;
        self.state.is_composing = false;
        self.spans.clear();
    }

    /// Maps a target codepoint cursor offset to its corresponding source byte offset in raw_buffer.
    pub fn map_target_to_source(&self, target_pos: usize) -> usize {
        if target_pos == 0 || self.spans.is_empty() {
            return 0;
        }
        let max_target = self.state.preedit.chars().count();
        if target_pos >= max_target {
            return self.state.raw_buffer.len();
        }
        for span in &self.spans {
            if target_pos >= span.target_range.start && target_pos <= span.target_range.end {
                if span.target_range.start == span.target_range.end {
                    return span.source_range.start;
                }
                let ratio = (target_pos - span.target_range.start) as f32
                    / (span.target_range.end - span.target_range.start) as f32;
                let source_len = span.source_range.end - span.source_range.start;
                let offset = (ratio * source_len as f32).round() as usize;
                return span.source_range.start + offset;
            }
        }
        self.state.raw_buffer.len()
    }

    /// Inserts a character into the raw buffer at or after cursor, then updates composition.
    fn insert_char_at_cursor(&mut self, c: char) {
        let mut chars: Vec<char> = self.state.raw_buffer.chars().collect();
        let idx = self.source_cursor.min(chars.len());
        chars.insert(idx, c);
        self.state.raw_buffer = chars.into_iter().collect();
        self.source_cursor = idx + c.len_utf8();
        self.update_composition();
        self.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(
            self.source_cursor,
            &self.state.raw_buffer,
            &self.spans,
            &self.state.preedit,
        );
    }

    /// Deletes a character in raw_buffer preceding the cursor position.
    fn delete_before_cursor(&mut self) {
        if self.source_cursor > 0 && !self.state.raw_buffer.is_empty() {
            let mut chars: Vec<char> = self.state.raw_buffer.chars().collect();
            let remove_idx = self.source_cursor.saturating_sub(1);
            if remove_idx < chars.len() {
                chars.remove(remove_idx);
            }
            self.state.raw_buffer = chars.into_iter().collect();
            self.source_cursor = self.source_cursor.saturating_sub(1);
            self.update_composition();
            self.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(
                self.source_cursor,
                &self.state.raw_buffer,
                &self.spans,
                &self.state.preedit,
            );
        }
    }

    /// Deletes a character in raw_buffer following the cursor position.
    fn delete_after_cursor(&mut self) {
        if self.source_cursor < self.state.raw_buffer.len() {
            let mut chars: Vec<char> = self.state.raw_buffer.chars().collect();
            if self.source_cursor < chars.len() {
                chars.remove(self.source_cursor);
            }
            self.state.raw_buffer = chars.into_iter().collect();
            self.update_composition();
            self.state.cursor_pos = phonetic::PhoneticEngine::map_cursor_to_target(
                self.source_cursor,
                &self.state.raw_buffer,
                &self.spans,
                &self.state.preedit,
            );
        }
    }

    /// Re-evaluates preedit and candidate list from the raw buffer using PhoneticEngine and DictionaryEngine.
    fn update_composition(&mut self) {
        if self.state.raw_buffer.is_empty() {
            self.clear_composition();
            return;
        }

        let (normalized, spans, m3_candidates) = phonetic::PhoneticEngine::transliterate(&self.state.raw_buffer);

        let m3_alternates = if m3_candidates.len() > 1 {
            m3_candidates[1..].to_vec()
        } else {
            Vec::new()
        };

        let dict_engine = dictionary::DictionaryEngine::global();
        let (candidates, candidate_details) = dict_engine.query_candidates(
            &self.state.raw_buffer,
            &normalized,
            m3_alternates,
            self.enable_dictionary_assistance,
            self.enable_word_completion,
            self.max_dictionary_suggestions,
            self.candidate_display_limit,
        );

        self.state.preedit = normalized;
        self.state.is_composing = !self.state.preedit.is_empty();
        self.state.candidates = candidates;
        self.state.candidate_details = candidate_details;
        self.spans = spans;
    }
}

/// Helper mapping standard ANSI physical keys to US QWERTY characters.
pub fn physical_key_to_ascii(key: layout::PhysicalKey, shift: bool) -> Option<char> {
    use layout::PhysicalKey::*;
    let ch = match (key, shift) {
        (Backquote, false) => '`', (Backquote, true) => '~',
        (Digit1, false) => '1', (Digit1, true) => '!',
        (Digit2, false) => '2', (Digit2, true) => '@',
        (Digit3, false) => '3', (Digit3, true) => '#',
        (Digit4, false) => '4', (Digit4, true) => '$',
        (Digit5, false) => '5', (Digit5, true) => '%',
        (Digit6, false) => '6', (Digit6, true) => '^',
        (Digit7, false) => '7', (Digit7, true) => '&',
        (Digit8, false) => '8', (Digit8, true) => '*',
        (Digit9, false) => '9', (Digit9, true) => '(',
        (Digit0, false) => '0', (Digit0, true) => ')',
        (Minus, false) => '-', (Minus, true) => '_',
        (Equal, false) => '=', (Equal, true) => '+',
        (KeyQ, false) => 'q', (KeyQ, true) => 'Q',
        (KeyW, false) => 'w', (KeyW, true) => 'W',
        (KeyE, false) => 'e', (KeyE, true) => 'E',
        (KeyR, false) => 'r', (KeyR, true) => 'R',
        (KeyT, false) => 't', (KeyT, true) => 'T',
        (KeyY, false) => 'y', (KeyY, true) => 'Y',
        (KeyU, false) => 'u', (KeyU, true) => 'U',
        (KeyI, false) => 'i', (KeyI, true) => 'I',
        (KeyO, false) => 'o', (KeyO, true) => 'O',
        (KeyP, false) => 'p', (KeyP, true) => 'P',
        (BracketLeft, false) => '[', (BracketLeft, true) => '{',
        (BracketRight, false) => ']', (BracketRight, true) => '}',
        (Backslash, false) => '\\', (Backslash, true) => '|',
        (KeyA, false) => 'a', (KeyA, true) => 'A',
        (KeyS, false) => 's', (KeyS, true) => 'S',
        (KeyD, false) => 'd', (KeyD, true) => 'D',
        (KeyF, false) => 'f', (KeyF, true) => 'F',
        (KeyG, false) => 'g', (KeyG, true) => 'G',
        (KeyH, false) => 'h', (KeyH, true) => 'H',
        (KeyJ, false) => 'j', (KeyJ, true) => 'J',
        (KeyK, false) => 'k', (KeyK, true) => 'K',
        (KeyL, false) => 'l', (KeyL, true) => 'L',
        (Semicolon, false) => ';', (Semicolon, true) => ':',
        (Quote, false) => '\'', (Quote, true) => '"',
        (KeyZ, false) => 'z', (KeyZ, true) => 'Z',
        (KeyX, false) => 'x', (KeyX, true) => 'X',
        (KeyC, false) => 'c', (KeyC, true) => 'C',
        (KeyV, false) => 'v', (KeyV, true) => 'V',
        (KeyB, false) => 'b', (KeyB, true) => 'B',
        (KeyN, false) => 'n', (KeyN, true) => 'N',
        (KeyM, false) => 'm', (KeyM, true) => 'M',
        (Comma, false) => ',', (Comma, true) => '<',
        (Period, false) => '.', (Period, true) => '>',
        (Slash, false) => '/', (Slash, true) => '?',
        (Space, _) => ' ',
    };
    Some(ch)
}

impl Default for BornoEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initial_status() {
        let engine = BornoEngine::new();
        let status = engine.status();
        assert_eq!(status.name, "Borno Core");
        assert_eq!(status.version, "0.1.0");
        assert!(status.ready);
        assert!(engine.state().preedit.is_empty());
        assert!(engine.state().candidates.is_empty());
        assert!(!engine.state().is_composing);
        assert_eq!(engine.state().cursor_pos, 0);
        assert_eq!(engine.state().selected_candidate_index, None);
    }

    #[test]
    fn test_deterministic_k_and_ka_interaction() {
        let mut engine = BornoEngine::new();

        // Type 'k'
        let res = engine.process_event(EngineEvent::InputChar('k'));
        assert_eq!(res.state.raw_buffer, "k");
        assert_eq!(res.state.preedit, "ক");
        assert!(res.state.is_composing);
        assert_eq!(res.state.cursor_pos, 1);
        assert_eq!(res.committed_text, None);
        assert!(res.handled);

        // Type 'a'
        let res = engine.process_event(EngineEvent::InputChar('a'));
        assert_eq!(res.state.raw_buffer, "ka");
        assert_eq!(res.state.preedit, "কা");
        assert!(res.state.is_composing);
        assert_eq!(res.state.cursor_pos, 2);
        assert_eq!(res.committed_text, None);
        assert!(res.handled);
    }

    #[test]
    fn test_backspace_handling() {
        let mut engine = BornoEngine::new();
        engine.process_event(EngineEvent::InputChar('k'));
        engine.process_event(EngineEvent::InputChar('a'));
        assert_eq!(engine.state().preedit, "কা");

        let res = engine.process_event(EngineEvent::Backspace);
        assert_eq!(res.state.raw_buffer, "k");
        assert_eq!(res.state.preedit, "ক");
        assert!(res.state.is_composing);

        let res = engine.process_event(EngineEvent::Backspace);
        assert_eq!(res.state.raw_buffer, "");
        assert_eq!(res.state.preedit, "");
        assert!(!res.state.is_composing);
        assert!(res.state.candidates.is_empty());
    }

    #[test]
    fn test_commit_flow() {
        let mut engine = BornoEngine::new();
        engine.process_event(EngineEvent::InputChar('a'));
        engine.process_event(EngineEvent::InputChar('m'));
        engine.process_event(EngineEvent::InputChar('i'));
        assert_eq!(engine.state().preedit, "আমি");

        let res = engine.process_event(EngineEvent::Commit);
        assert_eq!(res.committed_text, Some("আমি".to_string()));
        assert_eq!(res.state.preedit, "");
        assert_eq!(res.state.raw_buffer, "");
        assert!(!res.state.is_composing);
        // Engine state itself does NOT retain committed_text
        assert_eq!(engine.state().preedit, "");
    }

    #[test]
    fn test_space_commits_with_space() {
        let mut engine = BornoEngine::new();
        engine.process_event(EngineEvent::InputChar('k'));
        engine.process_event(EngineEvent::InputChar('a'));

        let res = engine.process_event(EngineEvent::InputChar(' '));
        assert_eq!(res.committed_text, Some("কা ".to_string()));
        assert_eq!(res.state.preedit, "");
        assert_eq!(res.state.raw_buffer, "");
        assert!(!res.state.is_composing);
    }

    #[test]
    fn test_reset_event() {
        let mut engine = BornoEngine::new();
        engine.process_event(EngineEvent::InputChar('k'));
        assert!(!engine.state().preedit.is_empty());

        let res = engine.process_event(EngineEvent::Reset);
        assert_eq!(res.state.preedit, "");
        assert_eq!(res.state.raw_buffer, "");
        assert!(res.state.candidates.is_empty());
        assert!(!res.state.is_composing);
        assert_eq!(res.committed_text, None);
        assert!(!res.cancelled);
        assert!(res.handled);
    }
}
