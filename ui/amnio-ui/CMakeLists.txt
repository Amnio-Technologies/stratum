# Set minimum required CMake version
cmake_minimum_required(VERSION 3.16)

# Define Project
project(amnio-ui C)

# Find All LVGL Source Files
file(GLOB_RECURSE LVGL_SOURCES
    ${PROJECT_SOURCE_DIR}/include/lvgl/src/*.c
)

add_definitions(-DLV_CONF_INCLUDE_SIMPLE) 
add_definitions(-DLV_CONF_PATH="${PROJECT_SOURCE_DIR}/include/lv_conf.h")

include_directories(
    ${PROJECT_SOURCE_DIR}/include        # amnio_ui.h
    ${PROJECT_SOURCE_DIR}/include/lvgl   # LVGL headers
    ${CMAKE_SOURCE_DIR}/../../common/include  # Rust-generated headers
)

# Exclude LVGL Demos and Tests
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/demos/.*")
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/tests/.*")

add_library(amnio-ui STATIC
    ${PROJECT_SOURCE_DIR}/src/amnio_ui.c
    ${LVGL_SOURCES}  
)

# Ensure Position-Independent Code (PIC) for Static Linking
set_property(TARGET amnio-ui PROPERTY POSITION_INDEPENDENT_CODE ON)

message(STATUS "CMake Build Type: ${CMAKE_BUILD_TYPE}")
