#ifndef stratum_ui_H
#define stratum_ui_H

#include <stdint.h>
#include <stdio.h>

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

    void ui_log(LogLevel level, const char *msg);

    AMNIO_API void lvgl_setup(void);
    AMNIO_API void lvgl_update(void);
    AMNIO_API uint16_t *get_lvgl_framebuffer(void);
    AMNIO_API uint32_t get_lvgl_display_width(void);
    AMNIO_API uint32_t get_lvgl_display_height(void);
    AMNIO_API void lvgl_advance_timer(uint32_t dt_ms);
    AMNIO_API size_t lvgl_get_required_framebuffer_size(void);
    AMNIO_API void lvgl_register_external_buffer(uint16_t *buffer, size_t buffer_bytes);

#ifdef __cplusplus
}
#endif

#endif // stratum_ui_H
