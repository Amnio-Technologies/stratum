# Set minimum required CMake version
cmake_minimum_required(VERSION 3.16)

# Define Project
project(stratum-ui C)

# Find All LVGL Source Files
file(GLOB_RECURSE LVGL_SOURCES
    ${PROJECT_SOURCE_DIR}/include/lvgl/src/*.c
)

add_definitions(-DLV_CONF_INCLUDE_SIMPLE) 
add_definitions(-DLV_CONF_PATH="${PROJECT_SOURCE_DIR}/include/lv_conf.h")

include_directories(
    ${PROJECT_SOURCE_DIR}/include        # stratum_ui.h, extern_log.h etc
    ${PROJECT_SOURCE_DIR}/include/lvgl   # LVGL headers
    ${PROJECT_SOURCE_DIR}/src/fonts
    ${CMAKE_SOURCE_DIR}/../../common/include  # Rust-generated headers
)

# Exclude LVGL Demos and Tests
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/demos/.*")
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/tests/.*")

file(GLOB FONT_SOURCES
    ${PROJECT_SOURCE_DIR}/fonts/*.c
)

add_library(stratum-ui STATIC
    ${PROJECT_SOURCE_DIR}/src/stratum_ui.c
    ${PROJECT_SOURCE_DIR}/src/extern_log.c
    ${FONT_SOURCES}
    ${LVGL_SOURCES}
)


# Ensure Position-Independent Code (PIC) for Static Linking
set_property(TARGET stratum-ui PROPERTY POSITION_INDEPENDENT_CODE ON)

message(STATUS "CMake Build Type: ${CMAKE_BUILD_TYPE}")
