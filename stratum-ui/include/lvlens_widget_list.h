#pragma once

//
// lvlens_widget_list.h
//
// A single‐argument X‐macro list of LVGL “create” functions to intercept.
// Whenever you add or remove a widget here, the shims in lvlens_shims.h
// adjust automatically.

#define LVLENS_WIDGET_LIST(X, Y)                              \
    /* real == alias */                                       \
    X(lv_obj_create)                                          \
    X(lv_label_create)                                        \
    X(lv_button_create)                                       \
    X(lv_slider_create)                                       \
    X(lv_switch_create)                                       \
    X(lv_checkbox_create)                                     \
    X(lv_dropdown_create)                                     \
    X(lv_textarea_create)                                     \
    X(lv_calendar_create)                                     \
    X(lv_chart_create)                                        \
    X(lv_list_create)                                         \
    X(lv_page_create)                                         \
    X(lv_tabview_create)                                      \
    X(lv_table_create)                                        \
    X(lv_tileview_create)                                     \
    X(lv_canvas_create)                                       \
    X(lv_colorwheel_create)                                   \
    X(lv_spinner_create)                                      \
    X(lv_preload_create)                                      \
    X(lv_imgbtn_create)                                       \
    X(lv_line_create)                                         \
    X(lv_led_create)                                          \
    X(lv_lmeter_create)                                       \
    X(lv_roller_create)                                       \
    X(lv_btnmatrix_create)                                    \
    X(lv_msgbox_create)                                       \
    X(lv_menu_create)                                         \
    X(lv_keyboard_create)                                     \
    X(lv_spinbox_create)                                      \
    X(lv_scale_create)                                        \
    X(lv_objmask_create)                                      \
                                                              \
    /* alias ≠ real — hook lv_img_create → lv_image_create */ \
    Y(lv_img_create, lv_image_create)
