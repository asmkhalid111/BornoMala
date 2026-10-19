cmake_minimum_required(VERSION 3.16)
project(bornomala-fcitx5 VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

find_package(PkgConfig REQUIRED)
find_package(Fcitx5Core 5.1.0 REQUIRED)
find_package(Fcitx5Utils REQUIRED)

include(GNUInstallDirs)

# Locate libbornomala_core.a
set(BORNOMALA_CORE_STATIC "/home/asmkhalid/borno-linux-target/release/libbornomala_core.a")
if(NOT EXISTS ${BORNOMALA_CORE_STATIC})
    set(BORNOMALA_CORE_STATIC "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/bornomala-core/target/release/libbornomala_core.a")
endif()

if(NOT EXISTS ${BORNOMALA_CORE_STATIC})
    message(FATAL_ERROR "Could not find libbornomala_core.a. Please build bornomala-core in release profile first.")
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
