#include "extern_log.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

// Static storage for the callback and its user data
static ui_log_cb_t g_log_cb = NULL;
static void *g_log_user_data = NULL;

// Register the Rust logging callback. Call this before lvgl_setup().
AMNIO_API void register_ui_log_callback(ui_log_cb_t cb, void *user_data)
{
    g_log_cb = cb;
    g_log_user_data = user_data;
}

void ui_logf(LogLevel log_level, const char *fmt, ...)
{
    va_list args;
    va_start(args, fmt);

    // Figure out how big the formatted message will be
    va_list args_copy;
    va_copy(args_copy, args);
    int needed = vsnprintf(NULL, 0, fmt, args_copy);
    va_end(args_copy);

    if (needed < 0)
    {
        va_end(args);
        return;
    }

    // Allocate a buffer for the message (+1 for '\0')
    char *log_message = (char *)malloc(needed + 1);
    if (!log_message)
    {
        va_end(args);
        return;
    }

    // Actually format into the buffer
    vsnprintf(log_message, needed + 1, fmt, args);
    va_end(args);

    // Dispatch to the registered callback, or fallback to stderr
    if (g_log_cb)
    {
        g_log_cb(g_log_user_data, log_level, log_message);
    }
    else
    {
        fprintf(stderr, "[%d] %s\n", (int)log_level, log_message);
    }

    free(log_message);
}
