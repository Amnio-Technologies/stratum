#include "lv_click_cache.h"
#include <stdlib.h>

static click_cache_entry_t *g_click_cache = NULL;

// Internal: recursively walk the tree under `obj`
static void cache_and_make_clickable(lv_obj_t *obj)
{
    // Cache original flag
    click_cache_entry_t *entry = malloc(sizeof(*entry));
    entry->obj = obj;
    entry->was_clickable = lv_obj_has_flag(obj, LV_OBJ_FLAG_CLICKABLE);
    HASH_ADD_PTR(g_click_cache, obj, entry);

    // Force clickable
    lv_obj_add_flag(obj, LV_OBJ_FLAG_CLICKABLE);

    // Recurse children
    uint32_t cnt = lv_obj_get_child_cnt(obj);
    for (uint32_t i = 0; i < cnt; ++i)
    {
        cache_and_make_clickable(lv_obj_get_child(obj, i));
    }
}

UI_EXPORT void make_all_clickable(void)
{
    // Clear any existing cache
    click_cache_entry_t *e, *tmp;
    HASH_ITER(hh, g_click_cache, e, tmp)
    {
        HASH_DEL(g_click_cache, e);
        free(e);
    }

    // Walk from active screen
    lv_obj_t *root = lv_scr_act();
    cache_and_make_clickable(root);
}

UI_EXPORT void revert_clickability(void)
{
    click_cache_entry_t *entry, *tmp;

    // Iterate over cached entries
    HASH_ITER(hh, g_click_cache, entry, tmp)
    {
        if (!entry->was_clickable)
        {
            // If it was originally non-clickable, clear the flag
            lv_obj_clear_flag(entry->obj, LV_OBJ_FLAG_CLICKABLE);
        }
        // Remove from hash and free
        HASH_DEL(g_click_cache, entry);
        free(entry);
    }
}
