#include "stratum_ui.h"
#include "extern_log.h"
#include "lvgl.h"
#include <stdio.h>
#include "fonts/jetbrains_mono_nl_regular_12.h"

// LVGL framebuffer (RGB565 format)
static uint16_t *lvgl_framebuffer = NULL;

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

    // Apply JetBrains Mono font style
    lv_obj_set_style_text_font(elapsed_label, &jetbrains_mono_nl_regular_12, LV_PART_MAIN);
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

AMNIO_API size_t lvgl_get_required_framebuffer_size(void)
{
    return LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT * sizeof(uint16_t);
}

AMNIO_API void lvgl_register_external_buffer(uint16_t *buffer, size_t buffer_bytes)
{
    size_t expected = lvgl_get_required_framebuffer_size();
    if (buffer_bytes < expected) {
        ui_logf(LOG_ERROR, "Buffer too small! Need at least %zu bytes.", expected);
        // abort();
        return;
    }

    lvgl_framebuffer = buffer;
}