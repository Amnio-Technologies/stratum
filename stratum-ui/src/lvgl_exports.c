#include "lvgl.h"
#include "lvgl_exports.h"

char *lvgl_label_text(const lv_obj_t *label)
{
    return lv_label_get_text(label);
}