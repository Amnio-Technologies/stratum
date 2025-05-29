#pragma once

#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include "lv_obj_tree.h"

#ifdef _WIN32
#define UI_EXPORT __declspec(dllexport) // Windows DLL Export
#else
#define UI_EXPORT
#endif

#define LVGL_SCREEN_WIDTH 320
#define LVGL_SCREEN_HEIGHT 240

#ifdef __cplusplus
extern "C"
{
#endif

    typedef enum
    {
        LOG_TRACE = 0,
        LOG_DEBUG = 1,
        LOG_INFO = 2,
        LOG_WARN = 3,
        LOG_ERROR = 4
    } LogLevel;

    // Callback registration for UI logging
    typedef void (*ui_log_cb_t)(void *user_data, LogLevel level, const char *msg);
    UI_EXPORT void register_ui_log_callback(ui_log_cb_t cb, void *user_data);

    UI_EXPORT void lvgl_setup(void);
    UI_EXPORT void lvgl_teardown(void);
    UI_EXPORT void lvgl_update(uint32_t dt_ms);
    UI_EXPORT uint16_t *get_lvgl_framebuffer(void);
    UI_EXPORT uint32_t get_lvgl_display_width(void);
    UI_EXPORT uint32_t get_lvgl_display_height(void);
    UI_EXPORT size_t lvgl_get_required_framebuffer_size(void);
    UI_EXPORT void lvgl_register_external_buffer(uint16_t *buffer, size_t buffer_bytes);

    /// SPI-send callback type.  is_data==false → command, true → pixel data.
    typedef void (*ui_spi_send_cb_t)(bool is_data, const uint8_t *data, size_t len);

    /// Called by LVGL's flush_cb to push bytes out.  Must be registered
    /// by the platform code *before* lvgl_setup().
    UI_EXPORT void lvgl_register_spi_send_cb(ui_spi_send_cb_t cb);

    typedef void (*tree_send_cb_t)(const FlatNode *nodes, size_t count, void *user_data);
    UI_EXPORT void register_tree_send_callback(tree_send_cb_t cb, void *user_data);

#ifdef __cplusplus
}
#endif
