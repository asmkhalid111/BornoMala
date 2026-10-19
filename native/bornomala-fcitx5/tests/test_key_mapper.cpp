#include <cassert>
#include <iostream>
#include <linux/input-event-codes.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/keysym.h>
#include <fcitx-utils/keysymgen.h>
#include "../key_mapper.h"

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

    assert(sizeof(pairs) / sizeof(pairs[0]) == 48 && "Must test all 48 physical keys");

    for (const auto& p : pairs) {
        uint32_t mapped = EvdevToBornoPhysicalKey(p.evdev);
        assert(mapped == p.borno && "EvdevToBornoPhysicalKey failed for key");
        
        uint32_t xkb_mapped = XkbKeycodeToBornoPhysicalKey(p.evdev + 8);
        assert(xkb_mapped == p.borno && "XkbKeycodeToBornoPhysicalKey failed for key");
    }

    // Test non-character / out of range keys
    assert(EvdevToBornoPhysicalKey(KEY_ESC) == BORNO_KEY_INVALID);
    assert(EvdevToBornoPhysicalKey(KEY_LEFTCTRL) == BORNO_KEY_INVALID);
    assert(EvdevToBornoPhysicalKey(KEY_F1) == BORNO_KEY_INVALID);
    assert(XkbKeycodeToBornoPhysicalKey(0) == BORNO_KEY_INVALID);
    assert(XkbKeycodeToBornoPhysicalKey(7) == BORNO_KEY_INVALID);

    std::cout << "    All 48 physical keys verified successfully.\n";
}

void TestPhoneticKeySyms() {
    std::cout << "  Testing Phonetic KeySym conversion (case sensitivity & shifted punctuation)...\n";

    // Lowercase
    assert(KeySymToPhoneticChar(FcitxKey_a) == 'a');
    assert(KeySymToPhoneticChar(FcitxKey_m) == 'm');
    assert(KeySymToPhoneticChar(FcitxKey_i) == 'i');

    // Uppercase (must not be downcased!)
    assert(KeySymToPhoneticChar(FcitxKey_T) == 'T');
    assert(KeySymToPhoneticChar(FcitxKey_D) == 'D');
    assert(KeySymToPhoneticChar(FcitxKey_R) == 'R');
    assert(KeySymToPhoneticChar(FcitxKey_S) == 'S');
    assert(KeySymToPhoneticChar(FcitxKey_Z) == 'Z');

    // Shifted punctuation
    assert(KeySymToPhoneticChar(FcitxKey_exclam) == '!');
    assert(KeySymToPhoneticChar(FcitxKey_at) == '@');
    assert(KeySymToPhoneticChar(FcitxKey_numbersign) == '#');
    assert(KeySymToPhoneticChar(FcitxKey_dollar) == '$');
    assert(KeySymToPhoneticChar(FcitxKey_percent) == '%');
    assert(KeySymToPhoneticChar(FcitxKey_question) == '?');
    assert(KeySymToPhoneticChar(FcitxKey_less) == '<');
    assert(KeySymToPhoneticChar(FcitxKey_greater) == '>');
    assert(KeySymToPhoneticChar(FcitxKey_asciitilde) == '~');

    // Non-printable control keys must return 0
    assert(KeySymToPhoneticChar(FcitxKey_F12) == 0);
    assert(KeySymToPhoneticChar(FcitxKey_Escape) == 0);
    assert(KeySymToPhoneticChar(FcitxKey_Return) == 0);
    assert(KeySymToPhoneticChar(FcitxKey_BackSpace) == 0);

    std::cout << "    Phonetic KeySyms verified successfully.\n";
}

void TestModifiersAndShortcuts() {
    std::cout << "  Testing modifier extraction and accelerator shortcut filtering...\n";

    // Modifiers
    assert(ExtractBornoModifiers(fcitx::KeyStates()) == 0);
    assert(ExtractBornoModifiers(fcitx::KeyState::Shift) == BORNO_MOD_SHIFT);
    assert(ExtractBornoModifiers(fcitx::KeyState::Mod5) == BORNO_MOD_ALTGR);
    assert(ExtractBornoModifiers(fcitx::KeyState::Ctrl) == BORNO_MOD_CTRL);
    assert(ExtractBornoModifiers(fcitx::KeyState::Alt) == BORNO_MOD_ALT);
    assert(ExtractBornoModifiers(fcitx::KeyStates{fcitx::KeyState::Shift, fcitx::KeyState::Mod5}) == 
           (BORNO_MOD_SHIFT | BORNO_MOD_ALTGR));

    // Shortcuts: Ctrl, Alt, Super must be detected
    assert(IsShortcutModifier(fcitx::KeyStates()) == false);
    assert(IsShortcutModifier(fcitx::KeyState::Shift) == false);
    assert(IsShortcutModifier(fcitx::KeyState::Ctrl) == true);
    assert(IsShortcutModifier(fcitx::KeyState::Alt) == true);
    assert(IsShortcutModifier(fcitx::KeyState::Super) == true);
    assert(IsShortcutModifier(fcitx::KeyState::Super2) == true);
    assert(IsShortcutModifier(fcitx::KeyStates{fcitx::KeyState::Ctrl, fcitx::KeyState::Shift}) == true);

    // AltGr (Mod5) should NOT be filtered as a shortcut!
    assert(IsShortcutModifier(fcitx::KeyState::Mod5) == false);
    assert(IsShortcutModifier(fcitx::KeyStates{fcitx::KeyState::Shift, fcitx::KeyState::Mod5}) == false);

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
