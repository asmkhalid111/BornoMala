/**
 * @file borno.h
 * @brief C-API declarations for Borno Core Bangla Input Platform.
 *
 * All string pointers returned by these APIs are allocated by the Rust core
 * and MUST be freed by calling borno_string_free() or the respective struct
 * cleanup functions (borno_status_free_members, borno_state_free_members,
 * borno_process_result_free_members).
 */

#ifndef BORNO_H
#define BORNO_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Status and Error Codes */
#define BORNO_OK                     0
#define BORNO_ERR_NULL_PTR          -1
#define BORNO_ERR_INVALID_UTF8      -2
#define BORNO_ERR_PANIC             -3
#define BORNO_ERR_INVALID_ARGUMENT  -4
#define BORNO_ERR_INVALID_CONFIG    -5
#define BORNO_ERR_UNSUPPORTED_SCHEMA -6

/* Input Modes */
#define BORNO_MODE_PHONETIC          0
#define BORNO_MODE_PASSTHROUGH       1
#define BORNO_MODE_FIXED_NATIONAL    2
#define BORNO_MODE_FIXED_PROBHAT     3
#define BORNO_MODE_FIXED_BIJOY       4

/* Candidate Kinds */
#define BORNO_CANDIDATE_PRIMARY_TRANSLITERATION  0
#define BORNO_CANDIDATE_PHONETIC_ALTERNATIVE    1
#define BORNO_CANDIDATE_EXACT_LEXICON_MATCH     2
#define BORNO_CANDIDATE_PREFIX_COMPLETION       3
#define BORNO_CANDIDATE_SPELLING_CORRECTION     4

/* Physical Key Identifiers (0 to 47) */
#define BORNO_KEY_BACKQUOTE     0
#define BORNO_KEY_DIGIT1        1
#define BORNO_KEY_DIGIT2        2
#define BORNO_KEY_DIGIT3        3
#define BORNO_KEY_DIGIT4        4
#define BORNO_KEY_DIGIT5        5
#define BORNO_KEY_DIGIT6        6
#define BORNO_KEY_DIGIT7        7
#define BORNO_KEY_DIGIT8        8
#define BORNO_KEY_DIGIT9        9
#define BORNO_KEY_DIGIT0        10
#define BORNO_KEY_MINUS         11
#define BORNO_KEY_EQUAL         12
#define BORNO_KEY_Q             13
#define BORNO_KEY_W             14
#define BORNO_KEY_E             15
#define BORNO_KEY_R             16
#define BORNO_KEY_T             17
#define BORNO_KEY_Y             18
#define BORNO_KEY_U             19
#define BORNO_KEY_I             20
#define BORNO_KEY_O             21
#define BORNO_KEY_P             22
#define BORNO_KEY_BRACKET_LEFT  23
#define BORNO_KEY_BRACKET_RIGHT 24
#define BORNO_KEY_BACKSLASH     25
#define BORNO_KEY_A             26
#define BORNO_KEY_S             27
#define BORNO_KEY_D             28
#define BORNO_KEY_F             29
#define BORNO_KEY_G             30
#define BORNO_KEY_H             31
#define BORNO_KEY_J             32
#define BORNO_KEY_K             33
#define BORNO_KEY_L             34
#define BORNO_KEY_SEMICOLON     35
#define BORNO_KEY_QUOTE         36
#define BORNO_KEY_Z             37
#define BORNO_KEY_X             38
#define BORNO_KEY_C             39
#define BORNO_KEY_V             40
#define BORNO_KEY_B             41
#define BORNO_KEY_N             42
#define BORNO_KEY_M             43
#define BORNO_KEY_COMMA         44
#define BORNO_KEY_PERIOD        45
#define BORNO_KEY_SLASH         46
#define BORNO_KEY_SPACE         47
#define BORNO_KEY_COUNT         48

/* Modifier Bitflags */
#define BORNO_MOD_SHIFT         0x01
#define BORNO_MOD_ALTGR         0x02
#define BORNO_MOD_CTRL          0x04
#define BORNO_MOD_ALT           0x08

/* Event Types */
#define BORNO_EVENT_INPUT_CHAR           0
#define BORNO_EVENT_INPUT_STRING         1
#define BORNO_EVENT_BACKSPACE            2
#define BORNO_EVENT_DELETE               3
#define BORNO_EVENT_COMMIT               4
#define BORNO_EVENT_CANCEL               5
#define BORNO_EVENT_RESET                6
#define BORNO_EVENT_NEXT_CANDIDATE       7
#define BORNO_EVENT_PREV_CANDIDATE       8
#define BORNO_EVENT_HIGHLIGHT_CANDIDATE  9
#define BORNO_EVENT_SELECT_CANDIDATE     10
#define BORNO_EVENT_SET_CURSOR           11
#define BORNO_EVENT_MOVE_CURSOR_LEFT     12
#define BORNO_EVENT_MOVE_CURSOR_RIGHT    13
#define BORNO_EVENT_SWITCH_MODE          14

/**
 * @brief Opaque engine handle.
 */
typedef struct BornoEngineHandle BornoEngineHandle;

/**
 * @brief Normalized event representation for C-ABI.
 */
typedef struct BornoEventC {
    int32_t     event_type;
    uint32_t    char_codepoint;     /**< Unicode codepoint for INPUT_CHAR */
    const char* string_payload;     /**< Borrowed null-terminated UTF-8 for INPUT_STRING */
    size_t      index_payload;      /**< Index for HIGHLIGHT/SELECT/SET_CURSOR */
    int32_t     mode_payload;       /**< 0: Phonetic, 1: EnglishPassthrough for SWITCH_MODE */
} BornoEventC;

/**
 * @brief High-level engine operational status.
 */
typedef struct BornoStatusC {
    char* name;
    char* version;
    char* active_mode;
    bool  ready;
} BornoStatusC;

/**
 * @brief Snapshot of the persistent engine state.
 */
typedef struct BornoStateC {
    char*   preedit;                  /**< Active composition string (UTF-8) */
    char*   raw_buffer;               /**< Raw Latin input buffer (UTF-8) */
    size_t  cursor_pos;               /**< Cursor position measured in Unicode codepoints */
    size_t  candidates_count;         /**< Number of candidates */
    char**  candidates;               /**< Array of UTF-8 candidate strings */
    int32_t selected_candidate_index; /**< -1 if None, otherwise 0 <= idx < count */
    bool    is_composing;             /**< True if active composition in progress */
    int32_t mode;                     /**< 0: Phonetic, 1: EnglishPassthrough */
} BornoStateC;

/**
 * @brief Ephemeral result produced by processing an event.
 */
typedef struct BornoProcessResultC {
    BornoStateC state;                /**< Snapshot of persistent engine state */
    char*       committed_text;       /**< Committed text (UTF-8), or NULL if none */
    bool        cancelled;            /**< True if composition was cancelled */
    bool        handled;              /**< True if engine consumed the event */
} BornoProcessResultC;

/* ========================================================================= */
/* Memory Management                                                         */
/* ========================================================================= */

/**
 * @brief Free a UTF-8 string allocated by the Rust engine.
 * Passing NULL is a safe no-op.
 */
void borno_string_free(char* ptr);

/**
 * @brief Free the dynamically allocated fields inside a BornoStatusC struct.
 * Does not free the container struct itself.
 */
void borno_status_free_members(BornoStatusC* status);

/**
 * @brief Free the dynamically allocated fields inside a BornoStateC struct.
 * Does not free the container struct itself.
 */
void borno_state_free_members(BornoStateC* state);

/**
 * @brief Free the dynamically allocated fields inside a BornoProcessResultC struct.
 * Does not free the container struct itself.
 */
void borno_process_result_free_members(BornoProcessResultC* result);

/* ========================================================================= */
/* Engine Lifecycle                                                          */
/* ========================================================================= */

/**
 * @brief Create a new engine instance.
 * @return Pointer to opaque BornoEngineHandle, or NULL on error.
 */
BornoEngineHandle* borno_engine_new(void);

/**
 * @brief Destroy an engine instance.
 * Passing NULL is a safe no-op.
 */
void borno_engine_free(BornoEngineHandle* handle);

/**
 * @brief Reconfigure the engine in-place using a JSON configuration string.
 *
 * Supports schemas v1.0.0, v1.1.0, and v1.2.0. In-place reconfiguration updates
 * settings and layout preferences without mutating active composition or preedit text.
 *
 * @param handle Valid engine handle.
 * @param json_utf8 Null-terminated UTF-8 JSON configuration string.
 * @return BORNO_OK on success, BORNO_ERR_INVALID_CONFIG (-5) on malformed JSON or validation
 *         failure, BORNO_ERR_UNSUPPORTED_SCHEMA (-6) on unsupported schema, or standard FFI error.
 */
int32_t borno_engine_configure_json(BornoEngineHandle* handle, const char* json_utf8);

/* ========================================================================= */
/* Engine State & Event Dispatch                                             */
/* ========================================================================= */

/**
 * @brief Query engine status.
 * @param handle Valid engine handle.
 * @param out_status Pointer to BornoStatusC struct to fill.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_get_status(const BornoEngineHandle* handle, BornoStatusC* out_status);

/**
 * @brief Process a normalized engine event and retrieve full process result.
 * @param handle Valid engine handle.
 * @param event Pointer to BornoEventC struct.
 * @param out_result Pointer to BornoProcessResultC struct to fill with results.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_process_event(
    BornoEngineHandle* handle,
    const BornoEventC* event,
    BornoProcessResultC* out_result
);

/**
 * @brief Convenience helper: Process a single Unicode codepoint input (UTF-32).
 * @param handle Valid engine handle.
 * @param codepoint Unicode character value (e.g. 'k' = 0x6B).
 * @param out_state Pointer to BornoStateC struct to fill with updated state.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_process_char(BornoEngineHandle* handle, uint32_t codepoint, BornoStateC* out_state);

/**
 * @brief Convenience helper: Process Backspace.
 * @param handle Valid engine handle.
 * @param out_state Pointer to BornoStateC struct to fill with updated state.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_backspace(BornoEngineHandle* handle, BornoStateC* out_state);

/**
 * @brief Convenience helper: Process Commit event.
 * @param handle Valid engine handle.
 * @param out_state Pointer to BornoStateC struct to fill with updated state.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_commit(BornoEngineHandle* handle, BornoStateC* out_state);

/**
 * @brief Convenience helper: Reset composition state.
 * @param handle Valid engine handle.
 * @param out_state Pointer to BornoStateC struct to fill with updated state.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_reset(BornoEngineHandle* handle, BornoStateC* out_state);

/**
 * @brief Process a physical key event directly via C-ABI.
 * @param handle Valid engine handle.
 * @param key_code ANSI physical key identifier (0 to 47).
 * @param modifiers Bitflags: bit 0: Shift, bit 1: AltGr, bit 2: Ctrl, bit 3: Alt.
 * @param out_result Pointer to BornoProcessResultC struct to fill with results.
 * @return BORNO_OK on success, or a negative error code.
 */
int32_t borno_engine_process_key(
    BornoEngineHandle* handle,
    uint32_t key_code,
    uint32_t modifiers,
    BornoProcessResultC* out_result
);

/**
 * @brief Retrieves the CandidateKind for the candidate at index from active engine state.
 *
 * @param handle Valid engine handle.
 * @param index 0-based candidate index.
 * @return 0 to 4 corresponding to CandidateKind, or -1 if index is out of bounds or handle is null.
 */
int32_t borno_engine_get_candidate_kind(const BornoEngineHandle* handle, size_t index);

/* ========================================================================= */
/* Direct UTF-8 Unicode FFI Verification Utility                             */
/* ========================================================================= */

/**
 * @brief Direct UTF-8 string round-trip test function.
 * Validates well-formed UTF-8 and returns an allocated duplicate with byte count.
 * Used to verify FFI memory safety and multi-byte Bangla Unicode preservation.
 *
 * @param input Null-terminated UTF-8 input string.
 * @param out_str Output pointer receiving the allocated copy.
 * @param out_len Output pointer receiving the byte length.
 * @return BORNO_OK on success, or BORNO_ERR_INVALID_UTF8 / BORNO_ERR_NULL_PTR.
 */
int32_t borno_ffi_echo_utf8(const char* input, char** out_str, size_t* out_len);

#ifdef __cplusplus
}
#endif

#endif /* BORNO_H */
