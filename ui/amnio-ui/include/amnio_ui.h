#ifndef AMNIO_UI_H
#define AMNIO_UI_H

#include "lvgl.h"
#include <stdint.h>

#ifdef _WIN32
#define AMNIO_API __declspec(dllexport) // Windows DLL Export
#else
#define AMNIO_API
#endif

#define LVGL_SCREEN_WIDTH 480
#define LVGL_SCREEN_HEIGHT 320

#ifdef __cplusplus
extern "C"
{
#endif

    // ✅ Function declarations (No implementation here)
    AMNIO_API void lvgl_setup(void);
    AMNIO_API void lvgl_update(void);
    AMNIO_API uint16_t *get_lvgl_framebuffer(void);

#ifdef __cplusplus
}
#endif

#endif // AMNIO_UI_H
