#include "bornomala.h"

/* Standard C99 compile-time assertion macro (no C11 _Static_assert dependency) */
#define C99_STATIC_ASSERT(cond, name) typedef char static_assertion_##name[(cond) ? 1 : -1]

C99_STATIC_ASSERT(BORNO_OK == 0, check_ok);
C99_STATIC_ASSERT(BORNO_ERR_NULL_PTR == -1, check_null_ptr);
C99_STATIC_ASSERT(BORNO_ERR_INVALID_UTF8 == -2, check_invalid_utf8);
C99_STATIC_ASSERT(BORNO_ERR_PANIC == -3, check_panic);
C99_STATIC_ASSERT(BORNO_ERR_INVALID_ARGUMENT == -4, check_invalid_argument);
C99_STATIC_ASSERT(BORNO_ERR_INVALID_CONFIG == -5, check_invalid_config);
C99_STATIC_ASSERT(BORNO_ERR_UNSUPPORTED_SCHEMA == -6, check_unsupported_schema);

C99_STATIC_ASSERT(BORNO_MODE_PHONETIC == 0, check_mode_phonetic);
C99_STATIC_ASSERT(BORNO_MODE_FIXED_BIJOY == 4, check_mode_bijoy);
C99_STATIC_ASSERT(BORNO_CANDIDATE_PRIMARY_TRANSLITERATION == 0, check_cand_primary);
C99_STATIC_ASSERT(BORNO_CANDIDATE_SPELLING_CORRECTION == 4, check_cand_spelling);

C99_STATIC_ASSERT(BORNO_KEY_BACKQUOTE == 0, check_key_0);
C99_STATIC_ASSERT(BORNO_KEY_SPACE == 47, check_key_space);
C99_STATIC_ASSERT(BORNO_KEY_COUNT == 48, check_key_count);

