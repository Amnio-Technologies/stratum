#include "lv_obj_tree.h"
#include "lvgl.h" // required for lv_obj_* and lv_area_t
#include <stdlib.h>
#include <string.h>
#include "lvgl/src/core/lv_obj_class_private.h"
#include "stratum_ui.h"

// Static storage for the callback and its user data
static tree_send_cb_t tree_send_cb = NULL;
static void *tree_send_user_data = NULL;

// Register the Rust logging callback. Call this before lvgl_setup().
UI_EXPORT void register_tree_send_callback(tree_send_cb_t cb, void *user_data)
{
    tree_send_cb = cb;
    tree_send_user_data = user_data;
}

// Internal recursive fill
static void fill_flat_nodes(lv_obj_t *obj, uintptr_t parent_ptr, FlatNode *out, size_t *i)
{
    FlatNode *n = &out[*i];
    (*i)++;

    n->ptr = (uintptr_t)obj;
    n->parent_ptr = parent_ptr;

    lv_obj_class_t *class = lv_obj_get_class(lv_screen_active());
    const char *name = class ? ((const struct _lv_obj_class_t *)class)->name : "unknown";
    n->class_name = name;

    lv_area_t coords;
    lv_obj_get_coords(obj, &coords);
    n->x = coords.x1;
    n->y = coords.y1;
    n->w = coords.x2 - coords.x1;
    n->h = coords.y2 - coords.y1;

    n->hidden = lv_obj_has_flag(obj, LV_OBJ_FLAG_HIDDEN);

    // Read debug_id from user_data
    // TODO ADD THIS
    // n->debug_id = lv_obj_get_user_data(obj).num;
    n->debug_id = 0;

    // Recurse on children
    uint32_t count = lv_obj_get_child_cnt(obj);
    for (uint32_t j = 0; j < count; ++j)
    {
        fill_flat_nodes(lv_obj_get_child(obj, j), (uintptr_t)obj, out, i);
    }
}

static size_t count_all_objects(lv_obj_t *obj)
{
    size_t total = 1;
    uint32_t count = lv_obj_get_child_cnt(obj);
    for (uint32_t j = 0; j < count; ++j)
    {
        total += count_all_objects(lv_obj_get_child(obj, j));
    }
    return total;
}

void lvscope_export_tree(void)
{
    lv_obj_t *root = lv_scr_act();
    size_t total = count_all_objects(root);

    FlatNode *nodes = malloc(sizeof(FlatNode) * total);
    if (!nodes)
    {
        return;
    }

    size_t i = 0;
    fill_flat_nodes(root, 0, nodes, &i);

    if (tree_send_cb)
    {
        tree_send_cb(nodes, total, tree_send_user_data);
    }

    free(nodes);
}