#include "key_mapper.h"
#include <linux/input-event-codes.h>

namespace bornomala {

uint32_t EvdevToBornoPhysicalKey(uint32_t evdev_code) {
    switch (evdev_code) {
        // Number row
        case KEY_GRAVE:      return BORNO_KEY_BACKQUOTE;
        case KEY_1:          return BORNO_KEY_DIGIT1;
        case KEY_2:          return BORNO_KEY_DIGIT2;
        case KEY_3:          return BORNO_KEY_DIGIT3;
        case KEY_4:          return BORNO_KEY_DIGIT4;
        case KEY_5:          return BORNO_KEY_DIGIT5;
        case KEY_6:          return BORNO_KEY_DIGIT6;
        case KEY_7:          return BORNO_KEY_DIGIT7;
        case KEY_8:          return BORNO_KEY_DIGIT8;
        case KEY_9:          return BORNO_KEY_DIGIT9;
        case KEY_0:          return BORNO_KEY_DIGIT0;
        case KEY_MINUS:      return BORNO_KEY_MINUS;
        case KEY_EQUAL:      return BORNO_KEY_EQUAL;

        // Top letter row
        case KEY_Q:          return BORNO_KEY_Q;
        case KEY_W:          return BORNO_KEY_W;
        case KEY_E:          return BORNO_KEY_E;
        case KEY_R:          return BORNO_KEY_R;
        case KEY_T:          return BORNO_KEY_T;
        case KEY_Y:          return BORNO_KEY_Y;
        case KEY_U:          return BORNO_KEY_U;
        case KEY_I:          return BORNO_KEY_I;
        case KEY_O:          return BORNO_KEY_O;
        case KEY_P:          return BORNO_KEY_P;
        case KEY_LEFTBRACE:  return BORNO_KEY_BRACKET_LEFT;
        case KEY_RIGHTBRACE: return BORNO_KEY_BRACKET_RIGHT;
        case KEY_BACKSLASH:  return BORNO_KEY_BACKSLASH;

        // Home row
        case KEY_A:          return BORNO_KEY_A;
        case KEY_S:          return BORNO_KEY_S;
        case KEY_D:          return BORNO_KEY_D;
        case KEY_F:          return BORNO_KEY_F;
        case KEY_G:          return BORNO_KEY_G;
        case KEY_H:          return BORNO_KEY_H;
        case KEY_J:          return BORNO_KEY_J;
        case KEY_K:          return BORNO_KEY_K;
        case KEY_L:          return BORNO_KEY_L;
        case KEY_SEMICOLON:  return BORNO_KEY_SEMICOLON;
        case KEY_APOSTROPHE: return BORNO_KEY_QUOTE;

        // Bottom row
        case KEY_Z:          return BORNO_KEY_Z;
        case KEY_X:          return BORNO_KEY_X;
        case KEY_C:          return BORNO_KEY_C;
        case KEY_V:          return BORNO_KEY_V;
        case KEY_B:          return BORNO_KEY_B;
        case KEY_N:          return BORNO_KEY_N;
        case KEY_M:          return BORNO_KEY_M;
        case KEY_COMMA:      return BORNO_KEY_COMMA;
        case KEY_DOT:        return BORNO_KEY_PERIOD;
        case KEY_SLASH:      return BORNO_KEY_SLASH;

        // Space
        case KEY_SPACE:      return BORNO_KEY_SPACE;

        default:
            return BORNO_KEY_INVALID;
    }
}

uint32_t XkbKeycodeToBornoPhysicalKey(uint32_t xkb_keycode) {
    // Linux evdev scancodes are mapped to XKB keycodes by adding 8
    if (xkb_keycode >= 8) {
        return EvdevToBornoPhysicalKey(xkb_keycode - 8);
    }
    return BORNO_KEY_INVALID;
}

uint32_t ExtractBornoModifiers(fcitx::KeyStates states) {
    uint32_t mods = 0;
    if (states.test(fcitx::KeyState::Shift)) {
        mods |= BORNO_MOD_SHIFT;
    }
    if (states.test(fcitx::KeyState::Mod5)) {
        mods |= BORNO_MOD_ALTGR;
    }
    if (states.test(fcitx::KeyState::Ctrl)) {
        mods |= BORNO_MOD_CTRL;
    }
    if (states.test(fcitx::KeyState::Alt)) {
        mods |= BORNO_MOD_ALT;
    }
    return mods;
}

bool IsShortcutModifier(fcitx::KeyStates states) {
    // Super (Windows key) shortcuts pass through directly
    if (states.test(fcitx::KeyState::Super) || states.test(fcitx::KeyState::Super2)) {
        return true;
    }
    // Ctrl or Alt shortcuts pass through, UNLESS it's an AltGr (Mod5) keystroke
    if (states.test(fcitx::KeyState::Ctrl) || states.test(fcitx::KeyState::Alt)) {
        if (!states.test(fcitx::KeyState::Mod5)) {
            return true;
        }
    }
    return false;
}

uint32_t KeySymToPhoneticChar(fcitx::KeySym sym) {
    // Check if it's an ASCII printable character
    if (sym >= 0x20 && sym <= 0x7E) {
        return static_cast<uint32_t>(sym);
    }

    // Use Fcitx 5 built-in helper for other unicode characters (e.g. currency signs, smart quotes)
    uint32_t ucs = fcitx::Key::keySymToUnicode(sym);
    if (ucs >= 0x20 && ucs != 0x7F) {
        return ucs;
    }

    return 0;
}

} // namespace bornomala
