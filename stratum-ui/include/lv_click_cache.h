#include "ui_export_marker.h"
#include "lvgl.h"
#include "uthash.h"

typedef struct click_cache_entry
{
    lv_obj_t *obj;      // key
    bool was_clickable; // original state
    UT_hash_handle hh;  // makes this struct hashable
} click_cache_entry_t;

// Make every object clickable, caching its original state
UI_EXPORT void make_all_clickable(void);

// Revert to original clickability using cache
UI_EXPORT void revert_clickability(void);
