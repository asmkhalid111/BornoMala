#pragma once

#ifndef KEY_MAPPER_H
#define KEY_MAPPER_H

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <cstdint>
#include <string>

// Include the BornoMala Core C-ABI contract
#include "../../crates/bornomala-core/include/bornomala.h"

namespace borno {

/**
 * @brief Translates a Windows Virtual-Key code (VK_*) to Borno's 48 physical key codes (0..=47).
 * @param wParam Windows virtual-key code.
 * @param lParam Windows key event lParam (contains scan code, extended key flag).
 * @param out_key Pointer receiving the physical key code (BORNO_KEY_*).
 * @return true if mapped to a physical key 0..=47, false otherwise.
 */
inline bool VirtualKeyToPhysicalKey(WPARAM wParam, LPARAM lParam, uint32_t* out_key) {
    if (!out_key) return false;

    // Digits 0-9
    if (wParam >= '1' && wParam <= '9') {
        *out_key = BORNO_KEY_DIGIT1 + static_cast<uint32_t>(wParam - '1');
        return true;
    }
    if (wParam == '0') {
        *out_key = BORNO_KEY_DIGIT0;
        return true;
    }

    // Letters A-Z
    switch (wParam) {
        case 'Q': *out_key = BORNO_KEY_Q; return true;
        case 'W': *out_key = BORNO_KEY_W; return true;
        case 'E': *out_key = BORNO_KEY_E; return true;
        case 'R': *out_key = BORNO_KEY_R; return true;
        case 'T': *out_key = BORNO_KEY_T; return true;
        case 'Y': *out_key = BORNO_KEY_Y; return true;
        case 'U': *out_key = BORNO_KEY_U; return true;
        case 'I': *out_key = BORNO_KEY_I; return true;
        case 'O': *out_key = BORNO_KEY_O; return true;
        case 'P': *out_key = BORNO_KEY_P; return true;
        case 'A': *out_key = BORNO_KEY_A; return true;
        case 'S': *out_key = BORNO_KEY_S; return true;
        case 'D': *out_key = BORNO_KEY_D; return true;
        case 'F': *out_key = BORNO_KEY_F; return true;
        case 'G': *out_key = BORNO_KEY_G; return true;
        case 'H': *out_key = BORNO_KEY_H; return true;
        case 'J': *out_key = BORNO_KEY_J; return true;
        case 'K': *out_key = BORNO_KEY_K; return true;
        case 'L': *out_key = BORNO_KEY_L; return true;
        case 'Z': *out_key = BORNO_KEY_Z; return true;
        case 'X': *out_key = BORNO_KEY_X; return true;
        case 'C': *out_key = BORNO_KEY_C; return true;
        case 'V': *out_key = BORNO_KEY_V; return true;
        case 'B': *out_key = BORNO_KEY_B; return true;
        case 'N': *out_key = BORNO_KEY_N; return true;
        case 'M': *out_key = BORNO_KEY_M; return true;
        default: break;
    }

    // Punctuation & Symbols
    switch (wParam) {
        case VK_OEM_3:      *out_key = BORNO_KEY_BACKQUOTE;     return true; // ` ~
        case VK_OEM_MINUS:  *out_key = BORNO_KEY_MINUS;         return true; // - _
        case VK_OEM_PLUS:   *out_key = BORNO_KEY_EQUAL;         return true; // = +
        case VK_OEM_4:      *out_key = BORNO_KEY_BRACKET_LEFT;  return true; // [ {
        case VK_OEM_6:      *out_key = BORNO_KEY_BRACKET_RIGHT; return true; // ] }
        case VK_OEM_5:      *out_key = BORNO_KEY_BACKSLASH;     return true; // \ |
        case VK_OEM_1:      *out_key = BORNO_KEY_SEMICOLON;     return true; // ; :
        case VK_OEM_7:      *out_key = BORNO_KEY_QUOTE;         return true; // ' "
        case VK_OEM_COMMA:  *out_key = BORNO_KEY_COMMA;         return true; // , <
        case VK_OEM_PERIOD: *out_key = BORNO_KEY_PERIOD;        return true; // . >
        case VK_OEM_2:      *out_key = BORNO_KEY_SLASH;         return true; // / ?
        case VK_SPACE:      *out_key = BORNO_KEY_SPACE;         return true; // Space
        default: break;
    }

    return false;
}

/**
 * @brief Queries current modifier keys using Win32 GetKeyState.
 * @return Bitflags combination of BORNO_MOD_*.
 */
inline uint32_t GetCurrentModifiers() {
    uint32_t mods = 0;
    if (GetKeyState(VK_SHIFT) < 0) {
        mods |= BORNO_MOD_SHIFT;
    }
    bool ctrl = (GetKeyState(VK_CONTROL) < 0);
    bool alt = (GetKeyState(VK_MENU) < 0);

    // AltGr on Windows keyboard layouts generates both Ctrl and Alt state
    if (ctrl && alt) {
        mods |= BORNO_MOD_ALTGR;
    } else {
        if (ctrl) mods |= BORNO_MOD_CTRL;
        if (alt) mods |= BORNO_MOD_ALT;
    }

    return mods;
}

/**
 * @brief Checks if a keystroke is a candidate selection digit ('1'..'9') within range.
 * @param wParam Virtual-key code.
 * @param candidates_count Available candidates count.
 * @param out_index Pointer receiving 0-based candidate index if valid.
 * @return true if valid candidate digit selection.
 */
inline bool IsCandidateSelectionDigit(WPARAM wParam, size_t candidates_count, size_t* out_index) {
    if (candidates_count == 0 || !out_index) return false;
    if (wParam >= '1' && wParam <= '9') {
        size_t idx = static_cast<size_t>(wParam - '1');
        if (idx < candidates_count) {
            *out_index = idx;
            return true;
        }
    }
    return false;
}

/**
 * @brief Converts a UTF-8 string to a UTF-16 wide string.
 */
inline std::wstring Utf8ToUtf16(const char* utf8_str) {
    if (!utf8_str || utf8_str[0] == '\0') return std::wstring();
    int needed = MultiByteToWideChar(CP_UTF8, 0, utf8_str, -1, nullptr, 0);
    if (needed <= 1) return std::wstring();
    std::wstring result(needed - 1, L'\0');
    MultiByteToWideChar(CP_UTF8, 0, utf8_str, -1, &result[0], needed);
    return result;
}

/**
 * @brief Converts a UTF-16 wide string to a UTF-8 string.
 */
inline std::string Utf16ToUtf8(const wchar_t* utf16_str) {
    if (!utf16_str || utf16_str[0] == L'\0') return std::string();
    int needed = WideCharToMultiByte(CP_UTF8, 0, utf16_str, -1, nullptr, 0, nullptr, nullptr);
    if (needed <= 1) return std::string();
    std::string result(needed - 1, '\0');
    WideCharToMultiByte(CP_UTF8, 0, utf16_str, -1, &result[0], needed, nullptr, nullptr);
    return result;
}

} // namespace borno

#endif // KEY_MAPPER_H
