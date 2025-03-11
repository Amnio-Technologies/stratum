# Find all LVGL source files
file(GLOB_RECURSE LVGL_SOURCES
    ${PROJECT_SOURCE_DIR}/include/lvgl/src/*.c
)

include_directories(
    ${PROJECT_SOURCE_DIR}/include        # ✅ amnio_ui.h is here
    ${PROJECT_SOURCE_DIR}/include/lvgl   # ✅ LVGL headers
)

# Exclude Demos and Tests
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/demos/.*")
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/tests/.*")

# Ensure all sources are included
add_library(amnio-ui SHARED
    ${PROJECT_SOURCE_DIR}/src/amnio_ui.c
    ${LVGL_SOURCES}  # ✅ LVGL sources are included in the build
)
