#ifndef BORNOMALA_FCITX5_KEY_MAPPER_H
#define BORNOMALA_FCITX5_KEY_MAPPER_H

#include <cstdint>
#include <fcitx-utils/key.h>
#include <fcitx-utils/keysym.h>
#include "bornomala.h"

namespace bornomala {

constexpr uint32_t BORNO_KEY_INVALID = 0xFFFFFFFF;

/**
 * @brief Map Linux evdev scancode to Borno physical key index (0..47).
 * @param evdev_code Hardware scancode from Linux input subsystem (e.g. KEY_A = 30).
 * @return Physical key index [0, 47], or BORNO_KEY_INVALID if unmapped.
 */
uint32_t EvdevToBornoPhysicalKey(uint32_t evdev_code);

/**
 * @brief Map XKB / Wayland / X11 keycode to Borno physical key index (0..47).
 * Standard XKB keycodes have an offset of +8 relative to evdev scancodes.
 * @param xkb_keycode Keycode reported by X11 / Wayland (e.g. keyEvent.origKey().code()).
 * @return Physical key index [0, 47], or BORNO_KEY_INVALID if unmapped.
 */
uint32_t XkbKeycodeToBornoPhysicalKey(uint32_t xkb_keycode);

/**
 * @brief Extract Borno modifier bitflags from Fcitx KeyStates.
 * @param states Bitset of Fcitx KeyState flags.
 * @return Bitwise OR of BORNO_MOD_SHIFT, BORNO_MOD_ALTGR, BORNO_MOD_CTRL, BORNO_MOD_ALT.
 */
uint32_t ExtractBornoModifiers(fcitx::KeyStates states);

/**
 * @brief Check if a key event contains system or application accelerator modifiers (Ctrl, Alt, Super)
 * that should cause the engine to pass the event through unconsumed.
 * Note: AltGr (Mod5) is NOT considered an accelerator shortcut modifier.
 */
bool IsShortcutModifier(fcitx::KeyStates states);

/**
 * @brief Convert an XKB keysym to Unicode codepoint for Phonetic mode.
 * Handles ASCII letters (case-sensitive), digits, and printable shifted/unshifted punctuation.
 * @param sym XKB KeySym (e.g. from keyEvent.key().sym()).
 * @return Unicode codepoint (UTF-32), or 0 if non-printable / control.
 */
uint32_t KeySymToPhoneticChar(fcitx::KeySym sym);

} // namespace bornomala

#endif // BORNOMALA_FCITX5_KEY_MAPPER_H
