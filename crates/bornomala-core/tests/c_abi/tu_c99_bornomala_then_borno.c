#include "bornomala.h"
#include "borno.h"

int32_t (*fn_configure)(BornoEngineHandle*, const char*) = &borno_engine_configure_json;

