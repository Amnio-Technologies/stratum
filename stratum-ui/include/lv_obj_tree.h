#pragma once

#include <stdint.h>
#include <stdbool.h>
#include "ui_export_marker.h"

typedef struct
{
    uintptr_t ptr;
    uintptr_t parent_ptr;
    const char *class_name;
    int16_t x, y, w, h;
    bool hidden;
    void *debug_id;
} FlatNode;

typedef void (*tree_send_cb_t)(void *user_data, const FlatNode *nodes, size_t count);
UI_EXPORT void register_tree_send_callback(tree_send_cb_t cb, void *user_data);
UI_EXPORT void export_tree(void);