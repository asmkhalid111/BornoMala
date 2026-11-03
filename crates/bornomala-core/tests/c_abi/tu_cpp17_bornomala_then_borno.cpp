#include "bornomala.h"
#include "borno.h"

int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;

static_assert(BORNO_ERR_INVALID_CONFIG == -5, "BORNO_ERR_INVALID_CONFIG mismatch");

