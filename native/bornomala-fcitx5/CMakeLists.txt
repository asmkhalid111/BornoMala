cmake_minimum_required(VERSION 3.16)
project(bornomala-fcitx5 VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

find_package(PkgConfig REQUIRED)
find_package(Fcitx5Core 5.1.0 REQUIRED)
find_package(Fcitx5Utils REQUIRED)

include(GNUInstallDirs)

# ==============================================================================
# Locate libbornomala_core.a Static Library
# ==============================================================================
# Resolution Precedence:
# 1. Explicit user override: -DBORNOMALA_CORE_STATIC=/path/to/libbornomala_core.a
#    - Must exist on disk; if missing, fails immediately without silent fallback.
# 2. Configured CARGO_TARGET_DIR environment variable (if set and non-empty):
#    - Checked before default workspace target to prevent stale artifacts from shadowing.
#    - Handles both absolute paths and workspace-relative paths.
# 3. Standard Cargo workspace target: <workspace_root>/target/release/libbornomala_core.a
# 4. Member sub-crate target: <workspace_root>/crates/bornomala-core/target/release/libbornomala_core.a
# ==============================================================================

if(DEFINED BORNOMALA_CORE_STATIC AND NOT "${BORNOMALA_CORE_STATIC}" STREQUAL "")
    # Precedence 1: Explicit user-supplied path
    if(NOT EXISTS "${BORNOMALA_CORE_STATIC}")
        message(FATAL_ERROR 
            "Explicit BORNOMALA_CORE_STATIC path does not exist:\n"
            "  ${BORNOMALA_CORE_STATIC}\n"
            "Please ensure the library is built before specifying this path."
        )
    endif()
    message(STATUS "Using explicitly provided libbornomala_core.a: ${BORNOMALA_CORE_STATIC}")
else()
    # Automatic Discovery
    set(WORKSPACE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../..")
    get_filename_component(WORKSPACE_ROOT "${WORKSPACE_ROOT}" ABSOLUTE)

    set(CHECKED_LOCATIONS "")

    # Precedence 2: CARGO_TARGET_DIR environment variable (if set)
    if(DEFINED ENV{CARGO_TARGET_DIR} AND NOT "$ENV{CARGO_TARGET_DIR}" STREQUAL "")
        if(IS_ABSOLUTE "$ENV{CARGO_TARGET_DIR}")
            set(RESOLVED_TARGET_DIR "$ENV{CARGO_TARGET_DIR}")
        else()
            get_filename_component(RESOLVED_TARGET_DIR "${WORKSPACE_ROOT}/$ENV{CARGO_TARGET_DIR}" ABSOLUTE)
        endif()
        set(ENV_TARGET "${RESOLVED_TARGET_DIR}/release/libbornomala_core.a")
        list(APPEND CHECKED_LOCATIONS "CARGO_TARGET_DIR: ${ENV_TARGET}")
        if(EXISTS "${ENV_TARGET}")
            set(BORNOMALA_CORE_STATIC "${ENV_TARGET}")
        endif()
    endif()

    # Precedence 3: Standard Cargo workspace target (repository root target/)
    if(NOT BORNOMALA_CORE_STATIC)
        set(WORKSPACE_TARGET "${WORKSPACE_ROOT}/target/release/libbornomala_core.a")
        list(APPEND CHECKED_LOCATIONS "Workspace Root:   ${WORKSPACE_TARGET}")
        if(EXISTS "${WORKSPACE_TARGET}")
            set(BORNOMALA_CORE_STATIC "${WORKSPACE_TARGET}")
        endif()
    endif()

    # Precedence 4: Sub-crate member target (crates/bornomala-core/target/)
    if(NOT BORNOMALA_CORE_STATIC)
        set(CRATE_TARGET "${WORKSPACE_ROOT}/crates/bornomala-core/target/release/libbornomala_core.a")
        list(APPEND CHECKED_LOCATIONS "Member Sub-crate: ${CRATE_TARGET}")
        if(EXISTS "${CRATE_TARGET}")
            set(BORNOMALA_CORE_STATIC "${CRATE_TARGET}")
        endif()
    endif()

    # If still not found, emit detailed error with build instructions
    if(NOT BORNOMALA_CORE_STATIC)
        string(REPLACE ";" "\n  - " CHECKED_LIST "${CHECKED_LOCATIONS}")
        message(FATAL_ERROR 
            "Could not locate libbornomala_core.a static library.\n"
            "Checked locations:\n"
            "  - ${CHECKED_LIST}\n"
            "Please build bornomala-core using:\n"
            "  cargo build --release -p bornomala-core\n"
            "  (or: RUSTFLAGS=\"-C relocation-model=pic\" cargo build --release -p bornomala-core)\n"
            "Or specify the library path directly via:\n"
            "  cmake -DBORNOMALA_CORE_STATIC=/path/to/libbornomala_core.a"
        )
    endif()

    message(STATUS "Discovered libbornomala_core.a: ${BORNOMALA_CORE_STATIC}")
endif()

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/../../crates/bornomala-core/include
)

# Shared module for Fcitx 5 addon
add_library(bornomala MODULE
    bornomala.cpp
    key_mapper.cpp
)

target_link_libraries(bornomala PRIVATE
    Fcitx5::Core
    Fcitx5::Utils
    ${BORNOMALA_CORE_STATIC}
    pthread
    dl
    m
)

set_target_properties(bornomala PROPERTIES
    PREFIX ""
    OUTPUT_NAME "bornomala"
)

# Test executables
enable_testing()

add_executable(test_c_linkage
    tests/test_c_linkage.cpp
)
target_link_libraries(test_c_linkage PRIVATE
    ${BORNOMALA_CORE_STATIC}
    pthread
    dl
    m
)
add_test(NAME CLinkageTest COMMAND test_c_linkage)

add_executable(test_key_mapper
    tests/test_key_mapper.cpp
    key_mapper.cpp
)
target_link_libraries(test_key_mapper PRIVATE
    Fcitx5::Utils
)
add_test(NAME KeyMapperTest COMMAND test_key_mapper)

add_executable(test_engine_adapter
    tests/test_engine_adapter.cpp
)
target_link_libraries(test_engine_adapter PRIVATE
    ${BORNOMALA_CORE_STATIC}
    pthread
    dl
    m
)
add_test(NAME EngineAdapterTest COMMAND test_engine_adapter)

# Configure metadata and input method files
configure_file(bornomala-addon.conf.in bornomala.conf @ONLY)
configure_file(bornomala.conf.in bornomala-im.conf @ONLY)

# Installation targets
install(TARGETS bornomala DESTINATION ${CMAKE_INSTALL_LIBDIR}/fcitx5)
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/bornomala.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/addon)
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/bornomala-im.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod RENAME bornomala.conf)
