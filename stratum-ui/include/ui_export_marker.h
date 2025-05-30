#pragma once

#ifdef _WIN32
#define UI_EXPORT __declspec(dllexport) // Windows DLL Export
#else
#define UI_EXPORT
#endif