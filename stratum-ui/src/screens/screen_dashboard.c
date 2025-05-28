#include "screen_dashboard.h"
#include "../components/component_output_card.h"
#include "lvgl.h"

static lv_obj_t *screen;

// helper to build the 3-tile flex row
static lv_obj_t *create_outut_card_row(lv_obj_t *parent)
{
    // 1) make a full-width container, fixed height
    lv_obj_t *row = lv_obj_create(parent);
    lv_obj_set_size(row, LV_PCT(100), LV_SIZE_CONTENT);

    // 1) Make the background transparent
    lv_obj_set_style_bg_opa(row, LV_OPA_TRANSP, LV_PART_MAIN);

    // 2) Remove the rounded corners (radius) if you want a sharp box
    lv_obj_set_style_radius(row, 0, LV_PART_MAIN);

    // 3) Remove border if you don’t want that either
    lv_obj_set_style_border_width(row, 0, LV_PART_MAIN);

    // 2) no scrolling or scrollbars
    lv_obj_clear_flag(row, LV_OBJ_FLAG_SCROLLABLE);
    lv_obj_set_scrollbar_mode(row, LV_SCROLLBAR_MODE_OFF);

    // 3) edge padding so tiles never hug the screen
    lv_obj_set_style_pad_left(row, 5, 0);
    lv_obj_set_style_pad_right(row, 5, 0);

    // 4) turn on flex-row layout
    lv_obj_set_layout(row, LV_LAYOUT_FLEX);
    lv_obj_set_flex_flow(row, LV_FLEX_FLOW_ROW);
    lv_obj_set_flex_align(row,
                          LV_FLEX_ALIGN_START,  // justify-content: flex-start
                          LV_FLEX_ALIGN_CENTER, // align-items: center
                          LV_FLEX_ALIGN_START); // align-content: flex-start

    // 5) gap between tiles = 3px
    lv_obj_set_style_pad_column(row, 3, 0);

    // 6) add exactly 3 tiles, each flex-grows to share the space
    for (int i = 0; i < 3; i++)
    {
        lv_obj_t *tile = component_output_card_create(row);
        lv_obj_set_flex_grow(tile, 1);
        // optional: ensure they don’t shrink below 60px width
        lv_obj_set_style_min_width(tile, 60, 0);
    }

    // you can align row inside its parent if needed:
    lv_obj_align(row, LV_ALIGN_TOP_MID, 0, 40);

    return row;
}

void screen_dashboard_create(void)
{
    screen = lv_obj_create(NULL);

    // black background
    lv_obj_set_style_bg_color(screen, lv_color_hex(0x000000), LV_PART_MAIN);
    lv_obj_clear_flag(screen, LV_OBJ_FLAG_SCROLLABLE);

    // build the tile row
    create_outut_card_row(screen);

    // finally, load it
    lv_scr_load(screen);
}