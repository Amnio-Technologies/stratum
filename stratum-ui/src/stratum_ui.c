#include "lv_conf.h"
#include "stratum_ui.h"
#include "extern_log.h"
#include "lvgl.h"
#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include "fonts/jetbrains_mono_nl_regular_12.h"
#include "screens/screen_dashboard.h"

// LVGL framebuffer (RGB565 format)
static uint16_t *lvgl_framebuffer = NULL;
static size_t lvgl_buffer_bytes = 0;

static lv_display_t *global_display = NULL;

static lv_obj_t *elapsed_label = NULL;
static uint32_t elapsed_seconds = 0;

static ui_spi_send_cb_t _spi_cb = NULL;

// your existing flush:
void my_flush_cb(lv_display_t *disp,
                 const lv_area_t *area,
                 uint8_t *px_map)
{
    const int32_t w = area->x2 - area->x1 + 1;
    const int32_t h = area->y2 - area->y1 + 1;
    const size_t bytes = (size_t)w * h * sizeof(uint16_t);

    // 1) Copy into our registered framebuffer (if any)
    if (lvgl_framebuffer)
    {
        // src starts at the first pixel of the region
        uint16_t *src = (uint16_t *)px_map;
        for (int32_t row = 0; row < h; row++)
        {
            // compute the start of this scanline in the big framebuffer
            uint16_t *dest = lvgl_framebuffer + ((area->y1 + row) * LVGL_SCREEN_WIDTH) + area->x1;
            // copy exactly w pixels
            memcpy(dest, src, (size_t)w * sizeof(uint16_t));
            src += w;
        }
    }

    // 2) Push out over SPI if callback registered
    if (_spi_cb)
    {
        uint8_t cmd = 0x2C;           // RAMWR
        _spi_cb(false, &cmd, 1);      // command mode
        _spi_cb(true, px_map, bytes); // data mode
    }

    // tell LVGL we're done
    lv_display_flush_ready(disp);
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

    lv_obj_set_style_bg_color(screen, lv_color_hex(0xff3a57), LV_PART_MAIN);

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

UI_EXPORT void lvgl_setup(void)
{
    lv_init();
    static lv_color_t buf[LVGL_SCREEN_WIDTH * 10]; // Display buffer
    global_display = lv_display_create(LVGL_SCREEN_WIDTH, LVGL_SCREEN_HEIGHT);
    lv_display_set_flush_cb(global_display, my_flush_cb);
    lv_display_set_buffers(global_display, buf, NULL, sizeof(buf), LV_DISPLAY_RENDER_MODE_PARTIAL);

    // 👇 Load the actual dashboard screen now
    screen_dashboard_create();
}

UI_EXPORT void lvgl_teardown(void)
{
    // Wipe current screen and all children
    lv_obj_clean(lv_screen_active());

    // Optional: delete all timers (LVGL might auto-delete them with objects)
    lv_timer_t *t;
    while ((t = lv_timer_get_next(NULL)) != NULL)
    {
        lv_timer_del(t);
    }

    elapsed_label = NULL;
    global_display = NULL;
    elapsed_seconds = 0;

    // Don't touch lvgl_framebuffer here unless you're managing it
}

UI_EXPORT void lvgl_update(uint32_t dt_ms)
{
    lv_tick_inc(dt_ms);
    lv_timer_handler();
}

UI_EXPORT uint16_t *get_lvgl_framebuffer(void)
{
    return lvgl_framebuffer;
}

UI_EXPORT uint32_t get_lvgl_display_width(void)
{
    return LVGL_SCREEN_WIDTH;
}

UI_EXPORT uint32_t get_lvgl_display_height(void)
{
    return LVGL_SCREEN_HEIGHT;
}

UI_EXPORT size_t lvgl_get_required_framebuffer_size(void)
{
    return LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT * sizeof(uint16_t);
}

UI_EXPORT void lvgl_register_external_buffer(uint16_t *buffer, size_t buffer_bytes)
{
    size_t expected = lvgl_get_required_framebuffer_size();
    ui_logf(LOG_INFO, "attempting to register buffer: %p", buffer);

    if (buffer_bytes < expected)
    {
        ui_logf(LOG_ERROR, "Buffer too small! Need at least %zu bytes.", expected);
        // abort();
        return;
    }
    ui_logf(LOG_INFO, "registered buffer: %p", buffer);

    lvgl_framebuffer = buffer;
    lvgl_buffer_bytes = buffer_bytes;
}

UI_EXPORT void lvgl_register_spi_send_cb(ui_spi_send_cb_t cb)
{
    _spi_cb = cb;
}
