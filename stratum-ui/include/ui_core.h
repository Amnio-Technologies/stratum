#pragma once

#include <stdint.h>
#include <stdbool.h>
#include "ui_export_marker.h"

#define LVGL_SCREEN_WIDTH 320
#define LVGL_SCREEN_HEIGHT 240

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
