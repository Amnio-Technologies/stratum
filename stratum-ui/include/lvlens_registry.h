#pragma once

#include <lvgl.h>
#include <stdbool.h>
#include "uthash.h"

/**
 * A struct to hold creation metadata for each lv_obj_t*.
 * We use `obj` as the hash key (i.e., the pointer value).
 */
typedef struct
{
    lv_obj_t *obj;           // key: the created object pointer
    const char *file;        // __FILE__ from the hook
    int line;                // __LINE__ from the hook
    const char *helper_name; // e.g. "lv_chart_create"
    UT_hash_handle hh;       // makes this struct hashable by uthash
} lvlens_meta_t;

/**
 * Called each time a wrapped helper (lv_obj_create, lv_label_create, lv_btn_create)
 * runs. Stores (obj, file, line, helper_name) in a uthash-backed registry.
 * If `obj` already exists in the hash, its metadata is overwritten.
 */
void lvlens_register(lv_obj_t *obj,
                     const char *file,
                     int line,
                     const char *helper_name);

/**
 * Look up metadata for a given object. Returns true if found,
 * and fills out the metadata fields in `out_meta`. Otherwise returns false.
 */
bool lvlens_get_metadata(lv_obj_t *obj,
                         lvlens_meta_t *out_meta);

void lvlens_dump_registry(void);