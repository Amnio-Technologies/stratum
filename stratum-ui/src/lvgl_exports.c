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

void lvgl_obj_set_shown(lv_obj_t *obj, bool shown)
{
    if (shown)
    {
        // lv_obj_clear_flag(obj, LV_OBJ_FLAG_HIDDEN);
        // Restore full opacity:
        lv_obj_clear_flag(obj, LV_OBJ_FLAG_HIDDEN); // ensure not hidden
        lv_obj_set_style_opa(obj, LV_OPA_100, 0);
        // And restore children similarly (or let default style cascade).
        // lv_obj_t *child;
        // child = lv_obj_get_child(obj, NULL);
        // while (child)
        // {
        //     lv_obj_set_style_opa(child, LV_OPA_100, 0);
        //     child = lv_obj_get_child(obj, child);
        // }
    }
    else
    {
        // lv_obj_add_flag(obj, LV_OBJ_FLAG_HIDDEN);
        lv_obj_set_style_opa(obj, LV_OPA_0, 0);
    }
}