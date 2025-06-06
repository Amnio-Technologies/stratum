#include "lvlens_registry.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// The global hash head. Initially NULL.
static lvlens_meta_t *g_lvlens_hash = NULL;

void lvlens_register(lv_obj_t *obj,
                     const char *file,
                     int line,
                     const char *helper_name)
{
    if (!obj)
    {
        return;
    }

    lvlens_meta_t *entry = NULL;

    // 1) Try to find an existing entry for this obj:
    HASH_FIND_PTR(g_lvlens_hash, &obj, entry);

    if (entry)
    {
        // Overwrite existing metadata
        entry->file = file;
        entry->line = line;
        entry->helper_name = helper_name;
    }
    else
    {
        // Create a new entry
        entry = (lvlens_meta_t *)malloc(sizeof(lvlens_meta_t));
        if (!entry)
        {
            // malloc failed; optionally handle this
            return;
        }
        entry->obj = obj;
        entry->file = file;
        entry->line = line;
        entry->helper_name = helper_name;
        // Add to hash
        HASH_ADD_PTR(g_lvlens_hash, obj, entry);
    }
}

bool lvlens_get_metadata(lv_obj_t *obj, lvlens_meta_t *out_meta)
{
    if (!obj || !out_meta)
    {
        return false;
    }

    lvlens_meta_t *entry = NULL;
    HASH_FIND_PTR(g_lvlens_hash, &obj, entry);

    if (!entry)
    {
        return false;
    }

    // Copy the found metadata into out_meta
    out_meta->obj = entry->obj;
    out_meta->file = entry->file;
    out_meta->line = entry->line;
    out_meta->helper_name = entry->helper_name;
    return true;
}

/**
 * Iterate over every (key-value) pair in g_lvlens_hash and print it.
 */
void lvlens_dump_registry(void)
{
    lvlens_meta_t *entry, *tmp;

    printf("---- LVLENS REGISTRY DUMP ----\n");
    HASH_ITER(hh, g_lvlens_hash, entry, tmp)
    {
        // entry->obj is the lv_obj_t* key
        // entry->file, entry->line, entry->helper_name are the stored metadata
        printf("  obj=%p   file=\"%s\"   line=%d   helper=\"%s\"\n",
               (void *)entry->obj,
               entry->file,
               entry->line,
               entry->helper_name);
    }
    printf("---- END DUMP ----\n");
}
