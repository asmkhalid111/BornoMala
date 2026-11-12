#ifndef BORNOMALA_FCITX5_TEST_CHECK_H
#define BORNOMALA_FCITX5_TEST_CHECK_H

#include <cstdlib>
#include <iostream>

/**
 * Release-active assertion macro. Unlike <cassert>, this never compiles out under NDEBUG.
 */
#define BORNO_CHECK(cond)                                                     \
    do {                                                                      \
        if (!(cond)) {                                                        \
            std::cerr << "[FAIL] Check failed: (" #cond ") at "               \
                      << __FILE__ << ":" << __LINE__ << std::endl;            \
            std::exit(1);                                                     \
        }                                                                     \
    } while (0)

#endif // BORNOMALA_FCITX5_TEST_CHECK_H

