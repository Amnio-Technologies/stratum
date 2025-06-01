#include "lvgl.h"
#include "lvgl_exports.h"

char *lvgl_label_text(const lv_obj_t *label)
{
    return lv_label_get_text(label);
}

lv_obj_t *lvgl_obj_at_point(int32_t x, int32_t y)
{
    lv_obj_t *screen = lv_scr_act();
    lv_point_t point = {.x = x, .y = y};

    return lv_indev_search_obj(screen, &point);
}