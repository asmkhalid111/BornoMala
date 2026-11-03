#include "bornomala.h"
#include <stdio.h>

int main(void) {
    BornoEngineHandle* engine = borno_engine_new();
    if (engine == NULL) {
        fprintf(stderr, "FAIL: borno_engine_new returned NULL\n");
        return 1;
    }

    /* Valid minimal configuration conforming to schema 1.2.0 */
    const char* json_cfg = "{\"schema_version\":\"1.2.0\"}";
    int32_t rc = borno_engine_configure_json(engine, json_cfg);
    if (rc != BORNO_OK) {
        fprintf(stderr, "FAIL: borno_engine_configure_json returned error %d\n", rc);
        borno_engine_free(engine);
        return 1;
    }

    borno_engine_free(engine);
    printf("Level 2 C linkage test passed.\n");
    return 0;
}

