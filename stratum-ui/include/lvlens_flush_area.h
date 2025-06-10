#pragma once

#include "lvgl.h"
#include "ui_export_marker.h"

typedef void (*flush_area_cb)(void *ud, const lv_area_t *area);
UI_EXPORT void register_flush_area_cb(flush_area_cb cb, void *ud);

UI_EXPORT void clear_flush_area_cb(void);

void lvlens_invoke_flush_area_cb(const lv_area_t *area);