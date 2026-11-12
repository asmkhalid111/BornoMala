#include <iostream>
#include <linux/input-event-codes.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/keysymgen.h>
#include "../key_mapper.h"
#include "test_check.h"

using namespace bornomala;

void TestPhysicalKeyMappings() {
    std::cout << "  Testing 48 physical key evdev & XKB keycode mappings...\n";

    struct KeyPair {
        uint32_t evdev;
        uint32_t borno;
    };

    const KeyPair pairs[] = {
        // Row 1
        {KEY_GRAVE, BORNO_KEY_BACKQUOTE},
        {KEY_1, BORNO_KEY_DIGIT1},
        {KEY_2, BORNO_KEY_DIGIT2},
        {KEY_3, BORNO_KEY_DIGIT3},
        {KEY_4, BORNO_KEY_DIGIT4},
        {KEY_5, BORNO_KEY_DIGIT5},
        {KEY_6, BORNO_KEY_DIGIT6},
        {KEY_7, BORNO_KEY_DIGIT7},
        {KEY_8, BORNO_KEY_DIGIT8},
        {KEY_9, BORNO_KEY_DIGIT9},
        {KEY_0, BORNO_KEY_DIGIT0},
        {KEY_MINUS, BORNO_KEY_MINUS},
        {KEY_EQUAL, BORNO_KEY_EQUAL},

        // Row 2
        {KEY_Q, BORNO_KEY_Q},
        {KEY_W, BORNO_KEY_W},
        {KEY_E, BORNO_KEY_E},
        {KEY_R, BORNO_KEY_R},
        {KEY_T, BORNO_KEY_T},
        {KEY_Y, BORNO_KEY_Y},
        {KEY_U, BORNO_KEY_U},
        {KEY_I, BORNO_KEY_I},
        {KEY_O, BORNO_KEY_O},
        {KEY_P, BORNO_KEY_P},
        {KEY_LEFTBRACE, BORNO_KEY_BRACKET_LEFT},
        {KEY_RIGHTBRACE, BORNO_KEY_BRACKET_RIGHT},
        {KEY_BACKSLASH, BORNO_KEY_BACKSLASH},

        // Row 3
        {KEY_A, BORNO_KEY_A},
        {KEY_S, BORNO_KEY_S},
        {KEY_D, BORNO_KEY_D},
        {KEY_F, BORNO_KEY_F},
        {KEY_G, BORNO_KEY_G},
        {KEY_H, BORNO_KEY_H},
        {KEY_J, BORNO_KEY_J},
        {KEY_K, BORNO_KEY_K},
        {KEY_L, BORNO_KEY_L},
        {KEY_SEMICOLON, BORNO_KEY_SEMICOLON},
        {KEY_APOSTROPHE, BORNO_KEY_QUOTE},

        // Row 4
        {KEY_Z, BORNO_KEY_Z},
        {KEY_X, BORNO_KEY_X},
        {KEY_C, BORNO_KEY_C},
        {KEY_V, BORNO_KEY_V},
        {KEY_B, BORNO_KEY_B},
        {KEY_N, BORNO_KEY_N},
        {KEY_M, BORNO_KEY_M},
        {KEY_COMMA, BORNO_KEY_COMMA},
        {KEY_DOT, BORNO_KEY_PERIOD},
        {KEY_SLASH, BORNO_KEY_SLASH},

        // Row 5
        {KEY_SPACE, BORNO_KEY_SPACE}
    };

    BORNO_CHECK(sizeof(pairs) / sizeof(pairs[0]) == 48 && "Must test all 48 physical keys");

    for (const auto& p : pairs) {
        uint32_t mapped = EvdevToBornoPhysicalKey(p.evdev);
        BORNO_CHECK(mapped == p.borno && "EvdevToBornoPhysicalKey failed for key");

        uint32_t xkb_mapped = XkbKeycodeToBornoPhysicalKey(p.evdev + 8);
        BORNO_CHECK(xkb_mapped == p.borno && "XkbKeycodeToBornoPhysicalKey failed for key");
    }

    // Test non-character / out of range keys
    BORNO_CHECK(EvdevToBornoPhysicalKey(KEY_ESC) == BORNO_KEY_INVALID);
    BORNO_CHECK(EvdevToBornoPhysicalKey(KEY_LEFTCTRL) == BORNO_KEY_INVALID);
    BORNO_CHECK(EvdevToBornoPhysicalKey(KEY_F1) == BORNO_KEY_INVALID);
    BORNO_CHECK(XkbKeycodeToBornoPhysicalKey(0) == BORNO_KEY_INVALID);
    BORNO_CHECK(XkbKeycodeToBornoPhysicalKey(7) == BORNO_KEY_INVALID);

    std::cout << "    All 48 physical keys verified successfully.\n";
}

void TestPhoneticKeySyms() {
    std::cout << "  Testing Phonetic KeySym conversion (case sensitivity & shifted punctuation)...\n";

    // Lowercase
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_a) == 'a');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_m) == 'm');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_i) == 'i');

    // Uppercase (must not be downcased!)
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_T) == 'T');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_D) == 'D');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_R) == 'R');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_S) == 'S');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_Z) == 'Z');

    // Shifted punctuation
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_exclam) == '!');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_at) == '@');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_numbersign) == '#');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_dollar) == '$');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_percent) == '%');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_question) == '?');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_less) == '<');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_greater) == '>');
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_asciitilde) == '~');

    // Non-printable control keys must return 0
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_F12) == 0);
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_Escape) == 0);
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_Return) == 0);
    BORNO_CHECK(KeySymToPhoneticChar(FcitxKey_BackSpace) == 0);

    std::cout << "    Phonetic KeySyms verified successfully.\n";
}

void TestModifiersAndShortcuts() {
    std::cout << "  Testing modifier extraction and accelerator shortcut filtering...\n";

    // Modifiers
    BORNO_CHECK(ExtractBornoModifiers(fcitx::KeyStates()) == 0);
    BORNO_CHECK(ExtractBornoModifiers(fcitx::KeyState::Shift) == BORNO_MOD_SHIFT);
    BORNO_CHECK(ExtractBornoModifiers(fcitx::KeyState::Mod5) == BORNO_MOD_ALTGR);
    BORNO_CHECK(ExtractBornoModifiers(fcitx::KeyState::Ctrl) == BORNO_MOD_CTRL);
    BORNO_CHECK(ExtractBornoModifiers(fcitx::KeyState::Alt) == BORNO_MOD_ALT);
    BORNO_CHECK(ExtractBornoModifiers(fcitx::KeyStates{fcitx::KeyState::Shift, fcitx::KeyState::Mod5}) ==
                (BORNO_MOD_SHIFT | BORNO_MOD_ALTGR));

    // Shortcuts: Ctrl, Alt, Super must be detected
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyStates()) == false);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyState::Shift) == false);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyState::Ctrl) == true);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyState::Alt) == true);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyState::Super) == true);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyState::Super2) == true);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyStates{fcitx::KeyState::Ctrl, fcitx::KeyState::Shift}) == true);

    // AltGr (Mod5) should NOT be filtered as a shortcut!
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyState::Mod5) == false);
    BORNO_CHECK(IsShortcutModifier(fcitx::KeyStates{fcitx::KeyState::Shift, fcitx::KeyState::Mod5}) == false);

    std::cout << "    Modifiers and shortcuts verified successfully.\n";
}

int main() {
    std::cout << "[Layer 3] Running Key Mapper Unit Tests...\n";
    TestPhysicalKeyMappings();
    TestPhoneticKeySyms();
    TestModifiersAndShortcuts();
    std::cout << "[Layer 3] Key Mapper Unit Tests PASSED!\n";
    return 0;
}
