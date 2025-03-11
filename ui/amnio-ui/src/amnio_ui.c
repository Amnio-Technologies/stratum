#include "amnio_ui.h"

// LVGL framebuffer (RGB565 format)
static uint16_t lvgl_framebuffer[LVGL_SCREEN_WIDTH * LVGL_SCREEN_HEIGHT];

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

void lv_example_get_started_1(void)
{
    lv_obj_set_style_bg_color(lv_screen_active(), lv_color_hex(0x003a57), LV_PART_MAIN);

    lv_obj_t *label = lv_label_create(lv_screen_active());
    lv_label_set_text(label, "Hello world");
    lv_obj_set_style_text_color(label, lv_color_hex(0xffffff), LV_PART_MAIN);
    lv_obj_align(label, LV_ALIGN_CENTER, 0, 0);
}

AMNIO_API void lvgl_setup(void)
{
    lv_init();
    static lv_color_t buf[LVGL_SCREEN_WIDTH * 10]; // Display buffer
    lv_display_t *display = lv_display_create(LVGL_SCREEN_WIDTH, LVGL_SCREEN_HEIGHT);
    lv_display_set_flush_cb(display, my_flush_cb);
    lv_display_set_buffers(display, buf, NULL, sizeof(buf), LV_DISPLAY_RENDER_MODE_PARTIAL);
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