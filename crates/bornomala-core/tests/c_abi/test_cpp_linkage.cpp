#include "bornomala.h"
#include <iostream>

int main() {
    BornoEngineHandle* engine = borno_engine_new();
    if (!engine) {
        std::cerr << "FAIL: borno_engine_new returned NULL\n";
        return 1;
    }

    const char* json_cfg = "{\"schema_version\":\"1.2.0\"}";
    int32_t rc = borno_engine_configure_json(engine, json_cfg);
    if (rc != BORNO_OK) {
        std::cerr << "FAIL: borno_engine_configure_json returned error " << rc << "\n";
        borno_engine_free(engine);
        return 1;
    }

    borno_engine_free(engine);
    std::cout << "Level 2 C++ linkage test passed.\n";
    return 0;
}

