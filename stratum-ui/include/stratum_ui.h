#ifndef stratum_ui_H
#define stratum_ui_H

#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>

#ifdef _WIN32
#define AMNIO_API __declspec(dllexport) // Windows DLL Export
#else
#define AMNIO_API
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
    AMNIO_API void register_ui_log_callback(ui_log_cb_t cb, void *user_data);

    AMNIO_API void lvgl_setup(void);
    AMNIO_API void lvgl_teardown(void);
    AMNIO_API void lvgl_update(uint32_t dt_ms);
    AMNIO_API uint16_t *get_lvgl_framebuffer(void);
    AMNIO_API uint32_t get_lvgl_display_width(void);
    AMNIO_API uint32_t get_lvgl_display_height(void);
    AMNIO_API size_t lvgl_get_required_framebuffer_size(void);
    AMNIO_API void lvgl_register_external_buffer(uint16_t *buffer, size_t buffer_bytes);

    /// SPI-send callback type.  is_data==false → command, true → pixel data.
    typedef void (*ui_spi_send_cb_t)(bool is_data, const uint8_t *data, size_t len);

    /// Called by LVGL's flush_cb to push bytes out.  Must be registered
    /// by the platform code *before* lvgl_setup().
    AMNIO_API void lvgl_register_spi_send_cb(ui_spi_send_cb_t cb);

#ifdef __cplusplus
}
#endif

#endif // stratum_ui_H
