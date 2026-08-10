// C wrapper to export Rust functions with __stdcall calling convention
#include <stdint.h>

// Declare Rust functions (extern "system" = __stdcall on Windows)
extern void __stdcall GiveFnptrsToDll(void* engfuncs, void* globals);
extern int __stdcall Meta_Query(const char* ifvers, void** plugin_info, void* meta_util_functions);
extern int __stdcall Meta_Attach(int now, void* meta_functions, void* meta_globals, void* gamedll_funcs);
extern int __stdcall Meta_Detach(int now, int reason);

// Re-export with explicit names (no decoration)
__declspec(dllexport) void __stdcall GiveFnptrsToDll(void* engfuncs, void* globals) {
    GiveFnptrsToDll(engfuncs, globals);
}

__declspec(dllexport) int __stdcall Meta_Query(const char* ifvers, void** plugin_info, void* meta_util_functions) {
    return Meta_Query(ifvers, plugin_info, meta_util_functions);
}

__declspec(dllexport) int __stdcall Meta_Attach(int now, void* meta_functions, void* meta_globals, void* gamedll_funcs) {
    return Meta_Attach(now, meta_functions, meta_globals, gamedll_funcs);
}

__declspec(dllexport) int __stdcall Meta_Detach(int now, int reason) {
    return Meta_Detach(now, reason);
}
