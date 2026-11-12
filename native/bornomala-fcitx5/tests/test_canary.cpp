#include <cstdlib>
#include <cstring>
#include <iostream>
#include "test_check.h"

int main() {
    std::cout << "[Canary] Verifying Release-active test check enforcement...\n";

#ifdef NDEBUG
    std::cout << "  Build configuration defines NDEBUG (Release check enforcement active).\n";
#else
    std::cout << "  Build configuration omits NDEBUG.\n";
#endif

    // Verify BORNO_CHECK evaluates unconditionally regardless of NDEBUG.
    int evaluatedCount = 0;
    auto bump = [&evaluatedCount]() {
        ++evaluatedCount;
        return true;
    };
    BORNO_CHECK(bump());
    BORNO_CHECK(evaluatedCount == 1);

    const char* canary = std::getenv("BORNO_TEST_CANARY");
    if (canary != nullptr && std::strcmp(canary, "1") == 0) {
        std::cerr << "[CANARY] Intentional failure triggered because BORNO_TEST_CANARY=1\n";
        BORNO_CHECK(false && "Intentional BORNO_TEST_CANARY=1 verification failure");
    }

    std::cout << "[Canary] Normal mode check PASSED (BORNO_TEST_CANARY not set to 1).\n";
    return 0;
}

