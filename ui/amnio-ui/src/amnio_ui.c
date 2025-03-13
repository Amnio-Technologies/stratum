#include "amnio_ui.h"
#include <stdio.h>
#include <stdlib.h>

// LVGL framebuffer (RGB565 format)
static uint16_t lvgl_framebuffer[LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT];

static lv_display_t *global_display = NULL; // Store reference to the display
static lv_obj_t *elapsed_label = NULL;      // Store reference to the label
static uint32_t elapsed_seconds = 0;        // Track elapsed seconds

void my_flush_cb(lv_display_t *display, const lv_area_t *area, uint8_t *px_map)
{
    uint16_t *src = (uint16_t *)px_map; // Convert to 16-bit color
    for (int y = area->y1; y <= area->y2; y++)
    {
        for (int x = area->x1; x <= area->x2; x++)
        {
            lvgl_framebuffer[y * LVGL_SCREEN_WIDTH + x] = *src++; // Copy pixel to framebuffer
        }
    }
    lv_display_flush_ready(display); // Notify LVGL flush is done
}

#ifdef _WIN32
int vasprintf(char **strp, const char *fmt, va_list args)
{
    if (!strp || !fmt)
        return -1; // ✅ Prevent null pointer issues

    // ✅ Get required buffer size (+1 for null terminator)
    int size = _vscprintf(fmt, args) + 1;
    if (size <= 0)
    {
        *strp = NULL;
        return -1;
    }

    // ✅ Allocate the required buffer
    char *buffer = (char *)malloc(size);
    if (!buffer)
    {
        *strp = NULL;
        return -1;
    }

    // ✅ Format the string into the allocated buffer
    int written = vsnprintf(buffer, size, fmt, args);
    if (written < 0 || written >= size)
    {
        free(buffer);
        *strp = NULL;
        return -1;
    }

    *strp = buffer;
    return written; // ✅ Return actual string length (excluding null terminator)
}
#endif

void ui_logf(LogLevel log_level, const char *fmt, ...)
{
    char *log_message;
    va_list args;

    va_start(args, fmt);
    int result = vasprintf(&log_message, fmt, args);
    va_end(args);

    if (result == -1)
    {
        return; // Allocation failed
    }

    ui_log(log_level, log_message);
    free(log_message);
}

void update_elapsed_time(lv_timer_t *timer)
{
    elapsed_seconds++;

    ui_logf(LOG_INFO, "Updating Elapsed Time: %u sec", elapsed_seconds);

    if (elapsed_label)
    {
        char buffer[32];
        snprintf(buffer, sizeof(buffer), "Elapsed: %u sec", elapsed_seconds);
        lv_label_set_text(elapsed_label, buffer);
    }
}

void lv_example_get_started_1(void)
{
    lv_obj_t *screen = lv_screen_active();
    if (!screen)
        return;

    lv_obj_set_style_bg_color(screen, lv_color_hex(0x003a57), LV_PART_MAIN);

    // Create the label for elapsed time
    elapsed_label = lv_label_create(screen);
    lv_label_set_text(elapsed_label, "Elapsed: 0 sec");
    lv_obj_set_style_text_color(elapsed_label, lv_color_hex(0xffffff), LV_PART_MAIN);
    lv_obj_align(elapsed_label, LV_ALIGN_CENTER, 0, 0);

    // Create a timer to update elapsed time every 1 second (1000ms)
    lv_timer_create(update_elapsed_time, 1000, NULL);
}

AMNIO_API void lvgl_setup(void)
{
    lv_init();
    static lv_color_t buf[LVGL_SCREEN_WIDTH * 10]; // Display buffer
    global_display = lv_display_create(LVGL_SCREEN_WIDTH, LVGL_SCREEN_HEIGHT);
    lv_display_set_flush_cb(global_display, my_flush_cb);
    lv_display_set_buffers(global_display, buf, NULL, sizeof(buf), LV_DISPLAY_RENDER_MODE_PARTIAL);
    lv_example_get_started_1();
}

AMNIO_API void lvgl_update(void)
{
    lv_timer_handler();
}

AMNIO_API uint16_t *get_lvgl_framebuffer(void)
{
    return lvgl_framebuffer;
}

AMNIO_API uint32_t get_lvgl_display_width(void)
{
    return LVGL_SCREEN_WIDTH;
}

AMNIO_API uint32_t get_lvgl_display_height(void)
{
    return LVGL_SCREEN_HEIGHT;
}

AMNIO_API void lvgl_advance_timer(uint32_t dt_ms)
{
    lv_tick_inc(dt_ms);
}
