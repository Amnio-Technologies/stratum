#include "components/component_output_card.h"
#include "lvgl.h"
#include "fonts/jetbrains_mono_nl_regular_12.h"
#include "fonts/atkinson_regular_20.h"
#include "fonts/atkinson_regular_16.h"
#include "fonts/atkinson_bold_20.h"

lv_obj_t *component_output_card_create(lv_obj_t *parent)
{
    lv_obj_t *tile = lv_obj_create(parent);
    lv_obj_set_size(tile, 100, 160);
    lv_obj_set_style_bg_color(tile, lv_color_hex(0x1D2125), LV_PART_MAIN);
    lv_obj_set_style_radius(tile, 4, 0);
    lv_obj_set_style_border_width(tile, 2, 0);
    lv_obj_set_style_border_color(tile, lv_color_hex(0x323841), 0);
    lv_obj_set_style_pad_all(tile, 8, 0);
    lv_obj_set_layout(tile, LV_LAYOUT_FLEX);
    lv_obj_set_flex_flow(tile, LV_FLEX_FLOW_COLUMN);
    lv_obj_set_flex_align(tile, LV_FLEX_ALIGN_CENTER, LV_FLEX_ALIGN_CENTER, LV_FLEX_ALIGN_CENTER);

    // Status Label: OUTPUT: ON
    lv_obj_t *status = lv_label_create(tile);
    lv_label_set_text(status, "OUTPUT: ON");
    lv_obj_set_style_text_font(status, &jetbrains_mono_nl_regular_12, 0);
    lv_obj_set_style_text_color(status, lv_color_hex(0xFFFFFF), LV_PART_MAIN);

    // Spacer
    // lv_obj_t *spacer = lv_obj_create(tile);
    // lv_obj_set_size(spacer, 1, 6);
    // lv_obj_set_style_bg_opa(spacer, LV_OPA_TRANSP, 0); // invisible spacer

    // Voltage
    lv_obj_t *voltage = lv_label_create(tile);
    lv_label_set_text(voltage, "20.02V");
    lv_obj_set_style_text_color(voltage, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
    lv_obj_set_style_text_font(voltage, &atkinson_regular_20, 0);

    // Current
    lv_obj_t *current = lv_label_create(tile);
    lv_label_set_text(current, "2.011V");
    lv_obj_set_style_text_color(current, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
    lv_obj_set_style_text_font(current, &atkinson_regular_20, 0);

    // Power
    lv_obj_t *power = lv_label_create(tile);
    lv_label_set_text(power, "40W");
    lv_obj_set_style_text_color(power, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
    lv_obj_set_style_text_font(power, &atkinson_regular_16, 0);

    // Bottom label
    lv_obj_t *label = lv_label_create(tile);
    lv_label_set_text(label, "BANANA JACK");
    lv_obj_set_style_text_color(label, lv_color_hex(0xFFFFFF), LV_PART_MAIN);
    lv_obj_set_style_text_font(label, &jetbrains_mono_nl_regular_12, 0);
    lv_obj_set_style_pad_top(label, 6, 0);

    lv_obj_clear_flag(tile, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_scrollbar_mode(tile, LV_SCROLLBAR_MODE_OFF);

    return tile;
}