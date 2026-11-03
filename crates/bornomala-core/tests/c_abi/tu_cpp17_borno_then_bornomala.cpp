#include "borno.h"
#include "bornomala.h"

/* Explicit function pointer assignment verifying declaration visibility in C++ */
int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;

static_assert(BORNO_ERR_INVALID_CONFIG == -5, "BORNO_ERR_INVALID_CONFIG mismatch");
static_assert(BORNO_ERR_UNSUPPORTED_SCHEMA == -6, "BORNO_ERR_UNSUPPORTED_SCHEMA mismatch");

