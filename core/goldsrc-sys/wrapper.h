// Wrapper header for bindgen.
// Only C-compatible HLSDK headers are included here.
// Metamod types are defined separately in the metamod backend crate.

// --- Minimal type definitions (normally from extdll.h) ---
typedef unsigned int string_t;
typedef float vec_t;
typedef vec_t vec3_t[3];

// --- HLSDK headers ---
#include "const.h"
#include "progdefs.h"
#include "edict.h"
#include "usercmd.h"
#include "eiface.h"
