#pragma once

#include "lvgl.h"
#include "ui_export_marker.h"

UI_EXPORT char *lvgl_label_text(const lv_obj_t *label);
UI_EXPORT lv_obj_t *lvgl_obj_at_point(int32_t x, int32_t y);
UI_EXPORT void lvgl_obj_set_shown(lv_obj_t *obj, bool hidden);