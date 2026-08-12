//! GoldSrc.rs Standalone Backend — proxy GameDLL loader.
//!
//! Loads the real game DLL (`mp.dll` / `cs.so`) and forwards all standard
//! GameDLL exports to it, while inserting our hooks around the key callbacks.

use std::path::PathBuf;
use std::sync::OnceLock;

use goldsrc_sys::{enginefuncs_t, globalvars_t, DLL_FUNCTIONS};

/// Name of the real game DLL to proxy.
#[cfg(target_os = "windows")]
const GAME_DLL_NAME: &str = "mp.dll";

#[cfg(target_os = "linux")]
const GAME_DLL_NAME: &str = "cs.so";

/// Holds the loaded real game DLL and its resolved function tables.
pub struct GameDllProxy {
    /// Libloading handle — keeps the DLL alive.
    _lib: libloading::Library,
    /// DLL function table populated by the real game DLL.
    pub dll_funcs: DLL_FUNCTIONS,
    /// Whether the real DLL was successfully loaded.
    #[allow(dead_code)]
    pub loaded: bool,
}

static PROXY: OnceLock<std::sync::Mutex<GameDllProxy>> = OnceLock::new();

/// Load the real game DLL and populate our proxy tables.
///
/// # Safety
/// `engfuncs` and `globals` must be valid pointers.
pub unsafe fn load(engfuncs: *mut enginefuncs_t, globals: *mut globalvars_t) -> bool {
    // Search for the game DLL relative to the executable directory.
    let dll_path = resolve_game_dll_path();

    let result = unsafe { try_load_game_dll(&dll_path, engfuncs, globals) };

    match result {
        Ok(proxy) => {
            let _ = PROXY.set(std::sync::Mutex::new(proxy));
            true
        }
        Err(e) => {
            // Log to stderr as server_print is not yet available.
            eprintln!(
                "[GoldSrc.rs Standalone] WARNING: Failed to load real game DLL '{}': {}",
                dll_path.display(),
                e
            );
            eprintln!("[GoldSrc.rs Standalone] Running in host-only mode (no game logic).");
            false
        }
    }
}

/// Resolve the path to the real game DLL.
fn resolve_game_dll_path() -> PathBuf {
    // Try relative to the executable directory (standard server layout).
    if let Ok(exe_dir) = std::env::current_exe().map(|p| p.parent().map(|d| d.to_path_buf())) {
        if let Some(base) = exe_dir {
            // Try cstrike/dlls/mp.dll (standard CS 1.6 layout).
            let candidate = base.join("cstrike").join("dlls").join(GAME_DLL_NAME);
            if candidate.exists() {
                return candidate;
            }
            // Try dlls/mp.dll (generic mod layout).
            let candidate2 = base.join("dlls").join(GAME_DLL_NAME);
            if candidate2.exists() {
                return candidate2;
            }
        }
    }
    // Fallback: just the DLL name and let the OS resolve it.
    PathBuf::from(GAME_DLL_NAME)
}

/// Load the game DLL and call `GiveFnptrsToDll` on it.
///
/// # Safety
/// `engfuncs` and `globals` must be valid.
unsafe fn try_load_game_dll(
    path: &PathBuf,
    engfuncs: *mut enginefuncs_t,
    globals: *mut globalvars_t,
) -> Result<GameDllProxy, Box<dyn std::error::Error>> {
    unsafe {
        // SAFETY: Loading a DLL from a trusted path.
        let lib = libloading::Library::new(path)?;

        // Call GiveFnptrsToDll on the real game DLL so it receives engine functions.
        let give_fns: libloading::Symbol<
            unsafe extern "system" fn(*mut enginefuncs_t, *mut globalvars_t),
        > = lib.get(b"GiveFnptrsToDll\0")?;
        give_fns(engfuncs, globals);

        // Retrieve the DLL_FUNCTIONS table from the real game DLL.
        let mut dll_funcs: DLL_FUNCTIONS = std::mem::zeroed();
        let mut iface_ver: i32 = 140;

        let get_entity_api: Result<
            libloading::Symbol<unsafe extern "C" fn(*mut DLL_FUNCTIONS, *mut i32) -> i32>,
            _,
        > = lib.get(b"GetEntityAPI2\0");

        if let Ok(f) = get_entity_api {
            f(&mut dll_funcs, &mut iface_ver);
        }

        Ok(GameDllProxy {
            _lib: lib,
            dll_funcs,
            loaded: true,
        })
    }
}

/// Forward a spawn call to the real game DLL if loaded.
pub fn forward_spawn(edict: *mut goldsrc_sys::edict_t) -> i32 {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            if let Some(f) = proxy.dll_funcs.pfnSpawn {
                return unsafe { f(edict) };
            }
        }
    }
    0
}

/// Forward a client connect call to the real game DLL if loaded.
pub fn forward_client_connect(
    edict: *mut goldsrc_sys::edict_t,
    name: *const std::os::raw::c_char,
    address: *const std::os::raw::c_char,
    reject_reason: *mut std::os::raw::c_char,
) -> i32 {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            if let Some(f) = proxy.dll_funcs.pfnClientConnect {
                return unsafe { f(edict, name, address, reject_reason) };
            }
        }
    }
    1 // Allow by default if no real DLL
}

/// Forward a client disconnect call to the real game DLL if loaded.
pub fn forward_client_disconnect(edict: *mut goldsrc_sys::edict_t) {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            if let Some(f) = proxy.dll_funcs.pfnClientDisconnect {
                unsafe { f(edict) };
            }
        }
    }
}

/// Forward a client command call to the real game DLL if loaded.
pub fn forward_client_command(edict: *mut goldsrc_sys::edict_t) {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            if let Some(f) = proxy.dll_funcs.pfnClientCommand {
                unsafe { f(edict) };
            }
        }
    }
}

/// Forward a start frame call to the real game DLL if loaded.
pub fn forward_start_frame() {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            if let Some(f) = proxy.dll_funcs.pfnStartFrame {
                unsafe { f() };
            }
        }
    }
}
