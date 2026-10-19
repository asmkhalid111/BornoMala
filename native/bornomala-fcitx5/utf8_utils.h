#ifndef BORNOMALA_FCITX5_UTF8_UTILS_H
#define BORNOMALA_FCITX5_UTF8_UTILS_H

#include <string>
#include <algorithm>
#include <cstddef>
#include <cstdint>

namespace bornomala {

/**
 * @brief Convert a Unicode codepoint index to a UTF-8 byte offset.
 *
 * Borno Core measures cursor position in Unicode codepoints (char count).
 * Fcitx 5 Text::setCursor() requires byte offsets.
 * Precondition: utf8_str is well-formed UTF-8 (guaranteed by Borno Core Rust String).
 *
 * @param utf8_str Valid UTF-8 string.
 * @param codepoint_idx 0-based character (codepoint) index.
 * @return Byte offset within [0, utf8_str.length()]. Clamps to string length if out-of-bounds.
 */
inline size_t CodepointToByteOffset(const std::string& utf8_str, size_t codepoint_idx) {
    size_t byte_idx = 0;
    size_t current_cp = 0;
    const size_t len = utf8_str.length();

    while (byte_idx < len && current_cp < codepoint_idx) {
        unsigned char c = static_cast<unsigned char>(utf8_str[byte_idx]);
        size_t char_len = 1;
        if ((c & 0x80) == 0) {
            char_len = 1;
        } else if ((c & 0xE0) == 0xC0) {
            char_len = 2;
        } else if ((c & 0xF0) == 0xE0) {
            char_len = 3;
        } else if ((c & 0xF8) == 0xF0) {
            char_len = 4;
        }

        if (byte_idx + char_len > len) {
            // Malformed trailing sequence protection
            byte_idx = len;
            break;
        }
        byte_idx += char_len;
        current_cp++;
    }

    return std::min(byte_idx, len);
}

} // namespace bornomala

#endif // BORNOMALA_FCITX5_UTF8_UTILS_H
