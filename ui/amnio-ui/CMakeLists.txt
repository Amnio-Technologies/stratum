cmake_minimum_required(VERSION 3.15)

# 🔹 Project Name
project(amnio-ui VERSION 1.0 LANGUAGES C CXX)

# 🔹 Set Language Standards
set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 17)

# 🔹 Ensure LVGL uses our custom configuration file
add_definitions(-DLV_CONF_PATH="${PROJECT_SOURCE_DIR}/include/lvgl/lv_conf.h")

# 🔹 Fix include directories to explicitly find `amnio-ui.h` and LVGL headers
include_directories(
    ${PROJECT_SOURCE_DIR}/include        # 🛠️ amnio-ui.h lives here
    ${PROJECT_SOURCE_DIR}/include/lvgl   # 🛠️ LVGL headers live here
)

# 🔹 Find All LVGL Sources (Excluding Demos/Tests)
file(GLOB_RECURSE LVGL_SOURCES
    ${PROJECT_SOURCE_DIR}/include/lvgl/src/*.c
)

# 🔹 Exclude Unnecessary LVGL Files
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/demos/.*")
list(FILTER LVGL_SOURCES EXCLUDE REGEX ".*lvgl/tests/.*")

# 🔹 Add Main Source Files (FIXED: `amnio_ui.c` path)
set(AMNIO_UI_SOURCES
    ${PROJECT_SOURCE_DIR}/src/amnio_ui.c  # ✅ Corrected path
)

# 🔹 Debugging Output: Log Found Sources
message(STATUS "Found LVGL sources: ${LVGL_SOURCES}")
message(STATUS "Found amnio-ui sources: ${AMNIO_UI_SOURCES}")

# 🔹 Create a Shared Library for Rust FFI
add_library(amnio-ui SHARED
    ${AMNIO_UI_SOURCES}
    ${LVGL_SOURCES}
)

# 🔹 Set Output Name (Ensures Correct `.dll` or `.so` Naming)
set_target_properties(amnio-ui PROPERTIES
    OUTPUT_NAME "amnio-ui"
    PREFIX ""  # No "lib" prefix on Windows
)

# 🔹 Windows-Specific Build Settings
if(WIN32)
    target_compile_definitions(amnio-ui PRIVATE -DLV_CONF_INCLUDE_SIMPLE -DAMNIO_UI_EXPORTS)
    set_target_properties(amnio-ui PROPERTIES SUFFIX ".dll")  # Explicitly set `.dll`
else()
    target_compile_definitions(amnio-ui PRIVATE -DLV_CONF_INCLUDE_SIMPLE)
    set_target_properties(amnio-ui PROPERTIES SUFFIX ".so")  # Explicitly set `.so` for Linux/macOS
endif()

# 🔹 Ensure the Library is Installed to the Build Directory
install(TARGETS amnio-ui
    RUNTIME DESTINATION ${CMAKE_BINARY_DIR}  # Ensures DLL/SO lands in build/
)

