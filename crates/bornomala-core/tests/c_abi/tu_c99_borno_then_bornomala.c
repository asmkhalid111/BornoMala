#include "borno.h"
#include "bornomala.h"

/* Explicit function pointer assignment to verify declaration visibility */
int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;

