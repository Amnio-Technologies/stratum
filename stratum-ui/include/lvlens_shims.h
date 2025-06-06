#pragma once

//=============================================================================
// lvlens_shims.h
//
// Hijacking shims for LVGL’s three core creation functions:
//   • lv_obj_create
//   • lv_button_create
//   • lv_label_create
//
// Each call to these will be redirected to a “hook” that invokes the real LVGL
// function and then registers the returned lv_obj_t* in your uthash‐backed registry.
//=============================================================================

#include "lvlens_registry.h"
#include <stdio.h> // TODO REMOVE ME
// 1) Pull in the real LVGL declarations (prototypes, types, etc.)
#include <lvgl.h>

// 2) Undefine any preexisting macros for our three targets—just in case
#ifdef lv_obj_create
#undef lv_obj_create
#endif
#ifdef lv_button_create
#undef lv_button_create
#endif
#ifdef lv_label_create
#undef lv_label_create
#endif

// 3) “Raw” inline helper prototypes. Each of these calls the genuine LVGL symbol.
//    Because we just #undef’d the macro, `lv_obj_create(parent)` here refers to
//    the real function in LVGL’s library.
static inline lv_obj_t *lvlens_lv_obj_create_raw(lv_obj_t *parent)
{
    return lv_obj_create(parent);
}
static inline lv_obj_t *lvlens_lv_button_create_raw(lv_obj_t *parent)
{
    return lv_button_create(parent);
}
static inline lv_obj_t *lvlens_lv_label_create_raw(lv_obj_t *parent)
{
    return lv_label_create(parent);
}

// 4) Hook‐wrapper declarations: call the “raw” helper, then call lvlens_register(...).
//    We forward “helper_name” exactly as the LVGL API name so you know which helper
//    was used. The file/line reflect the location in *user code* that invoked the helper.
static inline lv_obj_t *lvlens_lv_obj_create(
    lv_obj_t *parent,
    const char *file,
    int line)
{
    // 4a) Call the real lv_obj_create:
    lv_obj_t *obj = lvlens_lv_obj_create_raw(parent);
    // 4b) Register metadata in your uthash table:
    lvlens_register(obj, file, line, "lv_obj_create");
    return obj;
}

static inline lv_obj_t *lvlens_lv_button_create(
    lv_obj_t *parent,
    const char *file,
    int line)
{
    lv_obj_t *obj = lvlens_lv_button_create_raw(parent);
    lvlens_register(obj, file, line, "lv_button_create");
    return obj;
}

static inline lv_obj_t *lvlens_lv_label_create(
    lv_obj_t *parent,
    const char *file,
    int line)
{
    lv_obj_t *obj = lvlens_lv_label_create_raw(parent);
    lvlens_register(obj, file, line, "lv_label_create");
    return obj;
}

// 5) Redefine the public LVGL calls so that any `lv_obj_create(parent)` in user code
//    expands to `lvlens_lv_obj_create(parent, __FILE__, __LINE__)`, etc.

#define lv_obj_create(parent) \
    lvlens_lv_obj_create((parent), __FILE__, __LINE__)

#define lv_button_create(parent) \
    lvlens_lv_button_create((parent), __FILE__, __LINE__)

#define lv_label_create(parent) \
    lvlens_lv_label_create((parent), __FILE__, __LINE__)
