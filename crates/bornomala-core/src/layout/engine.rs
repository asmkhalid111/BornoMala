//! State machine and composition processor for Borno fixed keyboard layouts.

use crate::layout::schema::{LayoutDefinition, PhysicalKey};
use crate::unicode;

/// Composition state machine for fixed layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositionState {
    #[default]
    Idle,
    LinkPending,
}

/// Dedicated processing engine for fixed keyboard layouts.
#[derive(Debug, Clone)]
pub struct FixedLayoutEngine {
    layout: &'static LayoutDefinition,
    state: CompositionState,
}

impl FixedLayoutEngine {
    /// Create a new layout engine instance bound to a specific layout.
    pub fn new(layout: &'static LayoutDefinition) -> Self {
        Self {
            layout,
            state: CompositionState::Idle,
        }
    }

    /// Return the active composition state.
    pub fn state(&self) -> CompositionState {
        self.state
    }

    /// Reset composition state to Idle.
    pub fn reset_state(&mut self) {
        self.state = CompositionState::Idle;
    }

    /// Returns a reference to the active layout definition.
    pub fn layout(&self) -> &'static LayoutDefinition {
        self.layout
    }

    /// Set a new active layout, resetting composition state.
    pub fn set_layout(&mut self, layout: &'static LayoutDefinition) {
        self.layout = layout;
        self.state = CompositionState::Idle;
    }

    /// Process a keystroke producing `text` from physical key `key`.
    ///
    /// Updates `buffer` and `cursor_pos` in place according to the layout composition rules.
    pub fn process_keystroke(
        &mut self,
        buffer: &mut String,
        cursor_pos: &mut usize,
        key: PhysicalKey,
        text: &str,
    ) {
        let is_link_key = self.layout.rules.link_key == Some(key) && text == "্";

        if is_link_key {
            self.handle_link_key(buffer, cursor_pos);
        } else {
            self.handle_character_input(buffer, cursor_pos, text);
        }
    }

    /// Handle link key (Hasanta '্') keystroke.
    fn handle_link_key(&mut self, buffer: &mut String, cursor_pos: &mut usize) {
        let chars: Vec<char> = buffer.chars().collect();
        let cur = (*cursor_pos).min(chars.len());

        if self.state == CompositionState::LinkPending {
            // Repeated link key: consecutive hasantas collapse into single hasanta (M2 rule)
            // Buffer remains identical, state remains LinkPending
            return;
        }

        // Insert Hasanta '্' at current cursor
        let mut new_chars = Vec::with_capacity(chars.len() + 1);
        new_chars.extend_from_slice(&chars[..cur]);
        new_chars.push('্');
        new_chars.extend_from_slice(&chars[cur..]);

        let raw_new: String = new_chars.into_iter().collect();
        let normalized = unicode::normalize_nfc(&raw_new);
        let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur + 1, &normalized);

        *buffer = normalized;
        *cursor_pos = new_cursor;
        self.state = CompositionState::LinkPending;
    }

    /// Handle non-link character input.
    fn handle_character_input(&mut self, buffer: &mut String, cursor_pos: &mut usize, text: &str) {
        let chars: Vec<char> = buffer.chars().collect();
        let cur = (*cursor_pos).min(chars.len());

        if self.state == CompositionState::LinkPending {
            // Preceding character at cur - 1 is Hasanta '্'
            let rules = &self.layout.rules;

            // Subcase 1: Independent vowel composition via link key (National & Bijoy)
            if rules.supports_link_vowel_composition {
                if let Some(independent_vowel) = rules.link_vowel_map.get(text) {
                    // Replace preceding Hasanta '্' with independent vowel
                    let mut new_chars = Vec::with_capacity(chars.len());
                    if cur > 0 {
                        new_chars.extend_from_slice(&chars[..cur - 1]);
                    }
                    for ch in independent_vowel.chars() {
                        new_chars.push(ch);
                    }
                    new_chars.extend_from_slice(&chars[cur..]);

                    let raw_new: String = new_chars.into_iter().collect();
                    let normalized = unicode::normalize_nfc(&raw_new);
                    let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur, &normalized);

                    *buffer = normalized;
                    *cursor_pos = new_cursor;
                    self.state = CompositionState::Idle;
                    return;
                }
            }

            // Subcase 2: Leading virama at start of text followed by a consonant or punctuation
            // Orthographic Policy: Discard defective leading virama to prevent dotted circle ◌্ক
            if cur == 1 && chars.len() == 1 && chars[0] == '্' {
                // Buffer was solely the leading virama: replace it completely
                let normalized = unicode::normalize_nfc(text);
                *cursor_pos = normalized.chars().count();
                *buffer = normalized;
                self.state = CompositionState::Idle;
                return;
            }

            // Subcase 3: Virama immediately following punctuation (e.g. Dari '।')
            if cur >= 2 && chars[cur - 1] == '্' && is_punctuation(chars[cur - 2]) {
                // Post-punctuation virama dropped
                let mut new_chars = Vec::with_capacity(chars.len() + text.chars().count());
                new_chars.extend_from_slice(&chars[..cur - 1]);
                for ch in text.chars() {
                    new_chars.push(ch);
                }
                new_chars.extend_from_slice(&chars[cur..]);

                let raw_new: String = new_chars.into_iter().collect();
                let normalized = unicode::normalize_nfc(&raw_new);
                let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur - 1 + text.chars().count(), &normalized);

                *buffer = normalized;
                *cursor_pos = new_cursor;
                self.state = CompositionState::Idle;
                return;
            }

            // Subcase 4: In Probhat or non-vowel-mapping layouts, if a vowel sign follows Hasanta:
            // M2 sanitization replaces Hasanta with dependent vowel sign (e.g. ক + / + া -> কা)
            if !rules.supports_link_vowel_composition && is_vowel_sign(text) {
                let mut new_chars = Vec::with_capacity(chars.len());
                if cur > 0 {
                    new_chars.extend_from_slice(&chars[..cur - 1]);
                }
                for ch in text.chars() {
                    new_chars.push(ch);
                }
                new_chars.extend_from_slice(&chars[cur..]);

                let raw_new: String = new_chars.into_iter().collect();
                let normalized = unicode::normalize_nfc(&raw_new);
                let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur, &normalized);

                *buffer = normalized;
                *cursor_pos = new_cursor;
                self.state = CompositionState::Idle;
                return;
            }

            // Subcase 5: Regular conjunct ligature formation (e.g. ক + ্ + ষ -> ক্ষ)
            let mut new_chars = Vec::with_capacity(chars.len() + text.chars().count());
            new_chars.extend_from_slice(&chars[..cur]);
            for ch in text.chars() {
                new_chars.push(ch);
            }
            new_chars.extend_from_slice(&chars[cur..]);

            let raw_new: String = new_chars.into_iter().collect();
            let sanitized = unicode::sanitize_composition(&raw_new);
            let normalized = unicode::normalize_nfc(&sanitized);
            let target_cur = cur + text.chars().count();
            let new_cursor = unicode::map_cursor_after_normalization(&raw_new, target_cur, &normalized);

            *buffer = normalized;
            *cursor_pos = new_cursor;
            self.state = CompositionState::Idle;
        } else {
            // Idle State: direct character insertion at cursor
            let mut new_chars = Vec::with_capacity(chars.len() + text.chars().count());
            new_chars.extend_from_slice(&chars[..cur]);
            for ch in text.chars() {
                new_chars.push(ch);
            }
            new_chars.extend_from_slice(&chars[cur..]);

            let raw_new: String = new_chars.into_iter().collect();
            let sanitized = unicode::sanitize_composition(&raw_new);
            let normalized = unicode::normalize_nfc(&sanitized);
            let target_cur = cur + text.chars().count();
            let new_cursor = unicode::map_cursor_after_normalization(&raw_new, target_cur, &normalized);

            *buffer = normalized;
            *cursor_pos = new_cursor;
            self.state = CompositionState::Idle;
        }
    }

    /// Process Backspace in fixed layout mode.
    ///
    /// Implements codepoint deconstruction with automatic state reversion to LinkPending
    /// if the preceding character after deletion is a Hasanta '্'.
    pub fn handle_backspace(&mut self, buffer: &mut String, cursor_pos: &mut usize) -> bool {
        let chars: Vec<char> = buffer.chars().collect();
        let cur = *cursor_pos;

        if cur == 0 || chars.is_empty() {
            return false;
        }

        if self.state == CompositionState::LinkPending {
            // Backspace in LinkPending deletes Hasanta '্', keeping base consonant intact
            let mut new_chars = Vec::with_capacity(chars.len() - 1);
            new_chars.extend_from_slice(&chars[..cur - 1]);
            new_chars.extend_from_slice(&chars[cur..]);

            let raw_new: String = new_chars.into_iter().collect();
            let normalized = unicode::normalize_nfc(&raw_new);
            let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur - 1, &normalized);

            *buffer = normalized;
            *cursor_pos = new_cursor;
            self.state = CompositionState::Idle;
            return true;
        }

        // Idle State: delete single codepoint before cursor
        let mut new_chars = Vec::with_capacity(chars.len() - 1);
        new_chars.extend_from_slice(&chars[..cur - 1]);
        new_chars.extend_from_slice(&chars[cur..]);

        let raw_new: String = new_chars.into_iter().collect();
        let normalized = unicode::normalize_nfc(&raw_new);
        let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur - 1, &normalized);

        *buffer = normalized;
        *cursor_pos = new_cursor;

        // State-Aware Link Reversion:
        // If the character immediately preceding the new cursor is Hasanta '্',
        // automatically transition back to LinkPending state!
        let updated_chars: Vec<char> = buffer.chars().collect();
        if new_cursor > 0 && new_cursor <= updated_chars.len() && updated_chars[new_cursor - 1] == '্' {
            self.state = CompositionState::LinkPending;
        } else {
            self.state = CompositionState::Idle;
        }

        true
    }

    /// Process Delete key forward in fixed layout mode.
    pub fn handle_delete(&mut self, buffer: &mut String, cursor_pos: &mut usize) -> bool {
        let chars: Vec<char> = buffer.chars().collect();
        let cur = *cursor_pos;

        if cur >= chars.len() {
            // Cursor at end: nothing to delete forward
            return false;
        }

        // Delete single codepoint after cursor
        let mut new_chars = Vec::with_capacity(chars.len() - 1);
        new_chars.extend_from_slice(&chars[..cur]);
        new_chars.extend_from_slice(&chars[cur + 1..]);

        let raw_new: String = new_chars.into_iter().collect();
        let normalized = unicode::normalize_nfc(&raw_new);
        let new_cursor = unicode::map_cursor_after_normalization(&raw_new, cur, &normalized);

        *buffer = normalized;
        *cursor_pos = new_cursor;

        // If in LinkPending and deleted character was after cursor, cancel link state
        if self.state == CompositionState::LinkPending {
            self.state = CompositionState::Idle;
        }

        true
    }

    /// Cancel link state when cursor moves.
    pub fn handle_cursor_movement(&mut self) {
        if self.state == CompositionState::LinkPending {
            self.state = CompositionState::Idle;
        }
    }
}

/// Helper checking if a character is punctuation.
fn is_punctuation(ch: char) -> bool {
    ch == '।' || ch == '॥' || ch == '.' || ch == ',' || ch == ';' || ch == ':' || ch == '?' || ch == '!'
}

/// Helper checking if a string consists of a dependent vowel sign (kar).
fn is_vowel_sign(s: &str) -> bool {
    let mut chars = s.chars();
    if let Some(ch) = chars.next() {
        if chars.next().is_none() {
            return matches!(ch, 'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ');
        }
    }
    false
}
