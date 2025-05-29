#pragma once

#include "ui_export_marker.h"

typedef enum
{
    LOG_TRACE = 0,
    LOG_DEBUG = 1,
    LOG_INFO = 2,
    LOG_WARN = 3,
    LOG_ERROR = 4
} LogLevel;

// Callback registration for UI logging
typedef void (*ui_log_cb_t)(void *user_data, LogLevel level, const char *msg);
UI_EXPORT void register_ui_log_callback(ui_log_cb_t cb, void *user_data);

void ui_logf(LogLevel log_level, const char *fmt, ...);
