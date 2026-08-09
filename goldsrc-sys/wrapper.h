// Wrapper header for bindgen.
// We avoid including extdll.h directly because it pulls in <windows.h>,
// which is problematic for bindgen. Instead, we define the minimal
// types needed and include the C-compatible headers individually.

// --- Minimal type definitions (normally from extdll.h) ---
typedef unsigned int string_t;
typedef float vec_t;
#define vec3_t vec_t[3]

// --- HLSDK headers ---
#include "const.h"
#include "progdefs.h"
#include "edict.h"
#include "eiface.h"

// --- Metamod headers ---
#include "meta_api.h"
#include "dllapi.h"
#include "mutil.h"
