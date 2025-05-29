#pragma once

#include <stdint.h>
#include <stdbool.h>

typedef struct
{
    uintptr_t ptr;
    uintptr_t parent_ptr;
    const char *class_name;
    int16_t x, y, w, h;
    bool hidden;
    void *debug_id;
} FlatNode;