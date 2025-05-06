#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include "extern_log.h"

void ui_logf(LogLevel log_level, const char *fmt, ...)
{
    va_list args;
    va_start(args, fmt);

    // First, determine the size needed
    va_list args_copy;
    va_copy(args_copy, args);
    int needed = vsnprintf(NULL, 0, fmt, args_copy);
    va_end(args_copy);

    if (needed < 0)
    {
        va_end(args);
        return;
    }

    // Allocate space (+1 for null terminator)
    char *log_message = (char *)malloc(needed + 1);
    if (!log_message)
    {
        va_end(args);
        return;
    }

    // Format the actual string
    vsnprintf(log_message, needed + 1, fmt, args);
    va_end(args);

    // Log and free
    ui_log(log_level, log_message);
    free(log_message);
}