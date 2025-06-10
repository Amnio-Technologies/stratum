#include "lvlens_flush_area.h"

// ----------------------------------------------------------------------------
// Internal storage for the user callback + user data
// ----------------------------------------------------------------------------
static flush_area_cb g_flush_area_cb = NULL;
static void *g_flush_area_ud = NULL;

void register_flush_area_cb(flush_area_cb cb, void *ud)
{
    g_flush_area_cb = cb;
    g_flush_area_ud = ud;
}

void clear_flush_area_cb(void)
{
    register_flush_area_cb(NULL, NULL);
}

/**
 * Call this from your LVGL flush callback (before or after the real flush)
 * to notify the registered user callback of the region being drawn.
 */
void lvlens_invoke_flush_area_cb(const lv_area_t *area)
{
    if (g_flush_area_cb)
    {
        g_flush_area_cb(g_flush_area_ud, area);
    }
}
