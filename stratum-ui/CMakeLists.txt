cmake_minimum_required(VERSION 3.16)
project(stratum-ui C)

# -------- Options --------
set(STRATUM_TARGET "desktop" CACHE STRING "Build target: desktop or firmware")
message(STATUS "🎯 Building for: ${STRATUM_TARGET}")

# -------- Platform-specific flags --------
if(STRATUM_TARGET STREQUAL "desktop")
    add_definitions(-DSTRATUM_TARGET_DESKTOP)
elseif(STRATUM_TARGET STREQUAL "firmware")
    add_definitions(-DSTRATUM_TARGET_FIRMWARE)
endif()

# -------- LVGL Sources --------
file(GLOB_RECURSE LVGL_SOURCES
    ${PROJECT_SOURCE_DIR}/include/lvgl/src/*.c
)
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/demos/.*")
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/tests/.*")

# -------- Font Sources --------
file(GLOB FONT_SOURCES
    ${PROJECT_SOURCE_DIR}/fonts/*.c
    ${PROJECT_SOURCE_DIR}/src/fonts/*.c
)

# -------- LVGL Configuration --------
add_definitions(-DLV_CONF_INCLUDE_SIMPLE)
add_definitions(-DLV_CONF_PATH="${PROJECT_SOURCE_DIR}/include/lv_conf.h")

# -------- Include Directories --------
include_directories(
    ${PROJECT_SOURCE_DIR}/include           # stratum_ui.h, extern_log.h, lv_conf.h
    ${PROJECT_SOURCE_DIR}/include/lvgl      # LVGL headers
    ${PROJECT_SOURCE_DIR}/fonts             # generated font headers
    ${PROJECT_SOURCE_DIR}/src/fonts         # generated font C sources
    ${CMAKE_SOURCE_DIR}/../../common/include  # Rust-generated headers
)

# -------- Library --------
add_library(stratum-ui STATIC
    ${PROJECT_SOURCE_DIR}/src/stratum_ui.c
    ${PROJECT_SOURCE_DIR}/src/extern_log.c
    ${FONT_SOURCES}
    ${LVGL_SOURCES}
)

# -------- Properties --------
set_property(TARGET stratum-ui PROPERTY POSITION_INDEPENDENT_CODE ON)

message(STATUS "📦 CMake Build Type: ${CMAKE_BUILD_TYPE}")
