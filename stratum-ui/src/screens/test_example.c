#include "screens/test_example.h"

static lv_obj_t *elapsed_label = NULL;
static uint32_t elapsed_seconds = 0;

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

    lv_obj_set_style_bg_color(screen, lv_color_hex(0x000000), LV_PART_MAIN);

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