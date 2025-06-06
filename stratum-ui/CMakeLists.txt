cmake_minimum_required(VERSION 3.16)
project(stratum-ui C)

# -------- Linker Optimization --------
if(CMAKE_C_COMPILER_ID MATCHES "Clang|GNU")
    message(STATUS "⚡ Using LLD for faster linking")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -fuse-ld=lld")
    set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} -fuse-ld=lld")
    set(CMAKE_MODULE_LINKER_FLAGS "${CMAKE_MODULE_LINKER_FLAGS} -fuse-ld=lld")
endif()

# -------- Options --------
set(STRATUM_TARGET "desktop" CACHE STRING "Build target: desktop or firmware")
option(STRATUM_BUILD_DYNAMIC "Build shared library (.so/.dll) instead of static" OFF)
set(STRATUM_OUTPUT_NAME "" CACHE STRING "Override output library name (no extension)")

message(STATUS "🎯 Building for: ${STRATUM_TARGET}")
message(STATUS "🔗 Dynamic Linking Enabled: ${STRATUM_BUILD_DYNAMIC}")
if(STRATUM_OUTPUT_NAME)
    message(STATUS "📛 Custom Output Name: ${STRATUM_OUTPUT_NAME}")
endif()

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
# Exclude demos and tests
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*/demos/.*")
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*/tests/.*")

# -------- Font Sources --------
file(GLOB FONT_SOURCES
    ${PROJECT_SOURCE_DIR}/fonts/*.c
    ${PROJECT_SOURCE_DIR}/src/fonts/*.c
)

# -------- Build LVGL as OBJECT library --------
add_library(lvgl_objs OBJECT ${LVGL_SOURCES})
set_property(TARGET lvgl_objs PROPERTY POSITION_INDEPENDENT_CODE ON)

# -------- LVGL Configuration --------
add_definitions(-DLV_CONF_INCLUDE_SIMPLE)

# -------- Include Directories --------
include_directories(
    ${PROJECT_SOURCE_DIR}/include            # stratum_ui.h, extern_log.h, lv_conf.h
    ${PROJECT_SOURCE_DIR}/include/lvgl       # LVGL headers
    ${PROJECT_SOURCE_DIR}/fonts              # generated font headers
    ${PROJECT_SOURCE_DIR}/src/fonts          # generated font C sources
    ${CMAKE_SOURCE_DIR}/../../common/include # Rust-generated headers
)

# -------- stratum-ui Library --------
set(LIB_NAME stratum-ui)
if(STRATUM_OUTPUT_NAME)
    set(LIB_NAME "${STRATUM_OUTPUT_NAME}")
endif()

file(GLOB_RECURSE STRATUM_SOURCES CONFIGURE_DEPENDS
    ${PROJECT_SOURCE_DIR}/src/*.c
)

if(STRATUM_BUILD_DYNAMIC)
    add_library(${LIB_NAME} SHARED
        ${STRATUM_SOURCES}
        ${FONT_SOURCES}
        $<TARGET_OBJECTS:lvgl_objs>
    )
else()
    add_library(${LIB_NAME} STATIC
        ${STRATUM_SOURCES}
        ${FONT_SOURCES}
        $<TARGET_OBJECTS:lvgl_objs>
    )
endif()

# TODO: maybe guard this under a compiler flag to not always include shims for release builds
# Force-include our shim header on every TU in this target:
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
  if(CMAKE_C_COMPILER_ID MATCHES "Clang|GNU")
    target_compile_options(${LIB_NAME} PRIVATE
      -include lvlens_shims.h
    )
  elseif(MSVC)
    target_compile_options(${LIB_NAME} PRIVATE
      /FI lvlens_shims.h
    )
  endif()
endif()

# Windows: export all symbols automatically for DLLs
if(WIN32 AND STRATUM_BUILD_DYNAMIC)
    set_target_properties(${LIB_NAME} PROPERTIES WINDOWS_EXPORT_ALL_SYMBOLS ON)
endif()

# Required for shared object safety
set_property(TARGET ${LIB_NAME} PROPERTY POSITION_INDEPENDENT_CODE ON)

message(STATUS "📦 CMake Build Type: ${CMAKE_BUILD_TYPE}")
