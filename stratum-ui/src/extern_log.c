#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include "extern_log.h"

#ifdef _WIN32
int vasprintf(char **strp, const char *fmt, va_list args)
{
    if (!strp || !fmt)
        return -1; // Prevent null pointer issues

    // Get required buffer size (+1 for null terminator)
    int size = _vscprintf(fmt, args) + 1;
    if (size <= 0)
    {
        *strp = NULL;
        return -1;
    }

    // Allocate the required buffer
    char *buffer = (char *)malloc(size);
    if (!buffer)
    {
        *strp = NULL;
        return -1;
    }

    // Format the string into the allocated buffer
    int written = vsnprintf(buffer, size, fmt, args);
    if (written < 0 || written >= size)
    {
        free(buffer);
        *strp = NULL;
        return -1;
    }

    *strp = buffer;
    return written; //  Return actual string length (excluding null terminator)
}
#endif

void ui_logf(LogLevel log_level, const char *fmt, ...)
{
    char *log_message;
    va_list args;

    va_start(args, fmt);
    int result = vasprintf(&log_message, fmt, args);
    va_end(args);

    if (result == -1)
    {
        return; // Allocation failed
    }

    ui_log(log_level, log_message);
    free(log_message);
}