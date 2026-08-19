//! GoldSrc.rs Standalone Backend — proxy GameDLL loader.
//!
//! Loads the real game DLL (`mp.dll` / `cs.so`) and forwards all standard
//! GameDLL exports to it, while inserting our hooks around the key callbacks.

use goldsrc::log;
use goldsrc_sys::{enginefuncs_t, globalvars_t, DLL_FUNCTIONS, NEW_DLL_FUNCTIONS};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Name of the real game DLL to proxy.
#[cfg(target_os = "windows")]
const GAME_DLL_NAMES: &[&str] = &[
    "mp.dll",
    "server.dll",
    "hl.dll",
    "cs.so",
    "svencoop.dll",
    "dod.dll",
    "tfc.dll",
];

#[cfg(target_os = "linux")]
const GAME_DLL_NAMES: &[&str] = &["cs.so", "server.so", "hl.so", "dod.so", "tfc.so"];

/// Holds the loaded real game DLL and its resolved function tables.
pub struct GameDllProxy {
    /// Libloading handle — keeps the DLL alive.
    _lib: libloading::Library,
    /// DLL function table populated by the real game DLL.
    pub dll_funcs: DLL_FUNCTIONS,
    /// Optional NEW_DLL_FUNCTIONS table (ReGameDLL, Sven Co-op).
    pub new_dll_funcs: NEW_DLL_FUNCTIONS,
    pub has_new_dll_funcs: bool,
    /// Whether the real DLL was successfully loaded.
    #[allow(dead_code)]
    pub loaded: bool,
}

static PROXY: OnceLock<std::sync::Mutex<GameDllProxy>> = OnceLock::new();

pub fn dbg_log(msg: &str) {
    use std::io::Write;
    let _ = std::fs::create_dir_all("cstrike/goldsrc");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("cstrike/goldsrc/debug.log")
    {
        let _ = writeln!(f, "[Standalone] {msg}");
        let _ = f.flush();
    }
}

/// Ensure the real game DLL is loaded and its function tables are populated.
pub fn ensure_loaded() -> bool {
    if PROXY.get().is_some() {
        return true;
    }
    let dll_path = resolve_game_dll_path();
    dbg_log(&format!(
        "Attempting to load real GameDLL from path: {dll_path:?}"
    ));
    let norm_path = goldsrc::paths::PathResolver::normalize(&dll_path);
    log::info!(
        target: "proxy",
        "Attempting to load real GameDLL from path: \"{}\"",
        norm_path
    );

    let result = unsafe { try_load_game_dll(&dll_path) };

    match result {
        Ok(proxy) => {
            dbg_log(&format!(
                "Successfully loaded real GameDLL from \"{norm_path}\""
            ));
            log::info!(
                target: "proxy",
                "Successfully loaded real GameDLL from \"{}\"",
                norm_path
            );
            let _ = PROXY.set(std::sync::Mutex::new(proxy));
            true
        }
        Err(e) => {
            dbg_log(&format!(
                "ERROR: Failed to load real GameDLL from \"{norm_path}\": {e}"
            ));
            log::error!(
                target: "proxy",
                "ERROR: Failed to load real GameDLL from \"{}\": {}",
                norm_path,
                e
            );
            eprintln!(
                "[GoldSrc.rs Standalone] WARNING: Failed to load real game DLL \"{}\": {}",
                norm_path, e
            );
            false
        }
    }
}

/// Forward GiveFnptrsToDll to the real game DLL.
///
/// # Safety
/// `engfuncs` and `globals` must be valid engine pointers.
pub unsafe fn forward_give_fnptrs_to_dll(engfuncs: *mut enginefuncs_t, globals: *mut globalvars_t) {
    ensure_loaded();
    if let Some(proxy_lock) = PROXY.get() {
        let guard = proxy_lock.lock().unwrap_or_else(|e| e.into_inner());
        let give_fns: Result<
            libloading::Symbol<unsafe extern "system" fn(*mut enginefuncs_t, *mut globalvars_t)>,
            _,
        > = guard._lib.get(b"GiveFnptrsToDll\0");
        match give_fns {
            Ok(f) => {
                dbg_log("Forwarding GiveFnptrsToDll to real GameDLL...");
                f(engfuncs, globals);
                dbg_log("Real GameDLL GiveFnptrsToDll returned successfully");
            }
            Err(e) => {
                dbg_log(&format!(
                    "ERROR: GiveFnptrsToDll not found in real GameDLL: {e}"
                ));
            }
        }
    }
}

/// Helper to resolve the path of the original game DLL to proxy.
fn resolve_game_dll_path() -> PathBuf {
    // 1. Try reading original gamedll from liblist.gam
    let liblist_paths = ["cstrike/liblist.gam", "liblist.gam"];
    for liblist_path in &liblist_paths {
        if let Ok(content) = std::fs::read_to_string(liblist_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                // Find commented out original gamedll or active gamedll that isn't us
                if (trimmed.starts_with("gamedll") || trimmed.starts_with("; gamedll"))
                    && !trimmed.contains("goldsrc")
                {
                    // Clean up comments and extract value
                    let clean = trimmed.trim_start_matches(';').trim();
                    if clean.starts_with("gamedll") {
                        // Extract quoted path
                        if let Some(start) = clean.find('"') {
                            if let Some(end) = clean[start + 1..].find('"') {
                                let orig_path = &clean[start + 1..start + 1 + end];
                                let target_path = PathBuf::from(orig_path);
                                if target_path.exists() {
                                    return target_path;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Search common mod directories
    let mod_dirs = [
        "cstrike", "svencoop", "valve", "czero", "dod", "tfc", "gearbox", ".",
    ];
    for mod_dir in &mod_dirs {
        for dll_name in GAME_DLL_NAMES {
            let candidate = PathBuf::from(mod_dir).join("dlls").join(dll_name);
            if candidate.exists() {
                log::info!(
                    target: "proxy",
                    "Resolved GameDLL via mod search: \"{}\"",
                    goldsrc::paths::PathResolver::normalize(&candidate)
                );
                return candidate;
            }
        }
    }

    // 3. Fallback: try executable directory
    if let Ok(Some(base)) = std::env::current_exe().map(|p| p.parent().map(|d| d.to_path_buf())) {
        for mod_dir in &mod_dirs {
            for dll_name in GAME_DLL_NAMES {
                let candidate = base.join(mod_dir).join("dlls").join(dll_name);
                if candidate.exists() {
                    log::info!(
                        target: "proxy",
                        "Resolved GameDLL via exe base search: \"{}\"",
                        goldsrc::paths::PathResolver::normalize(&candidate)
                    );
                    return candidate;
                }
            }
        }
    }

    PathBuf::from("cstrike")
        .join("dlls")
        .join(GAME_DLL_NAMES[0])
}

/// Load the game DLL and populate its function tables.
unsafe fn try_load_game_dll(path: &PathBuf) -> Result<GameDllProxy, Box<dyn std::error::Error>> {
    unsafe {
        // SAFETY: Loading a DLL from a trusted path.
        let lib = libloading::Library::new(path)?;

        // Retrieve the DLL_FUNCTIONS table from the real game DLL.
        let mut dll_funcs: DLL_FUNCTIONS = std::mem::zeroed();
        let mut iface_ver: i32 = 140;

        let get_entity_api2: Result<
            libloading::Symbol<unsafe extern "C" fn(*mut DLL_FUNCTIONS, *mut i32) -> i32>,
            _,
        > = lib.get(b"GetEntityAPI2\0");

        let mut loaded_api = false;
        if let Ok(f) = get_entity_api2 {
            let ret = f(&mut dll_funcs, &mut iface_ver);
            if ret != 0 {
                log::info!(
                    target: "proxy",
                    "Successfully populated DLL_FUNCTIONS via GetEntityAPI2"
                );
                loaded_api = true;
            }
        }

        if !loaded_api {
            // Fallback: GetEntityAPI(DLL_FUNCTIONS*, int)
            let get_entity_api: Result<
                libloading::Symbol<unsafe extern "C" fn(*mut DLL_FUNCTIONS, i32) -> i32>,
                _,
            > = lib.get(b"GetEntityAPI\0");
            if let Ok(f) = get_entity_api {
                let ret = f(&mut dll_funcs, 140);
                if ret != 0 {
                    log::info!(
                        target: "proxy",
                        "Successfully populated DLL_FUNCTIONS via GetEntityAPI"
                    );
                    loaded_api = true;
                }
            }
        }

        if !loaded_api {
            log::warn!(
                target: "proxy",
                "WARNING: Neither GetEntityAPI2 nor GetEntityAPI returned 1 for real GameDLL!"
            );
        }

        // Retrieve NEW_DLL_FUNCTIONS if available
        let mut new_dll_funcs: NEW_DLL_FUNCTIONS = std::mem::zeroed();
        let mut new_iface_ver: i32 = 1;
        let get_new_dll_fns: Result<
            libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *mut i32) -> i32>,
            _,
        > = lib.get(b"GetNewDLLFunctions\0");

        let has_new_dll_funcs = if let Ok(f) = get_new_dll_fns {
            let ret = f(
                &mut new_dll_funcs as *mut _ as *mut std::ffi::c_void,
                &mut new_iface_ver,
            );
            if ret != 0 {
                log::info!(
                    target: "proxy",
                    "Successfully populated NEW_DLL_FUNCTIONS via GetNewDLLFunctions"
                );
                true
            } else {
                false
            }
        } else {
            false
        };

        Ok(GameDllProxy {
            _lib: lib,
            dll_funcs,
            new_dll_funcs,
            has_new_dll_funcs,
            loaded: true,
        })
    }
}

/// Copy the real game DLL's function table into the provided table pointer.
pub fn populate_dll_table(dll_table: *mut DLL_FUNCTIONS) {
    ensure_loaded();
    if dll_table.is_null() {
        return;
    }
    if let Some(lock) = PROXY.get() {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        if guard.loaded {
            unsafe {
                std::ptr::copy_nonoverlapping(&guard.dll_funcs, dll_table, 1);
            }
            dbg_log("Successfully copied real DLL_FUNCTIONS to engine table!");
        } else {
            dbg_log("ERROR: Real GameDLL not loaded when calling populate_dll_table!");
        }
    }
}

/// Copy NEW_DLL_FUNCTIONS from real GameDLL if present.
pub fn populate_new_dll_table(new_dll_table: *mut std::ffi::c_void) -> bool {
    ensure_loaded();
    if new_dll_table.is_null() {
        return false;
    }
    if let Some(lock) = PROXY.get() {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        if guard.loaded && guard.has_new_dll_funcs {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &guard.new_dll_funcs,
                    new_dll_table as *mut NEW_DLL_FUNCTIONS,
                    1,
                );
            }
            dbg_log("Successfully copied real NEW_DLL_FUNCTIONS to engine table!");
            return true;
        }
    }
    false
}

/// Forward a spawn call to the real game DLL if loaded.
pub fn forward_spawn(edict: *mut goldsrc_sys::edict_t) -> i32 {
    let func = PROXY.get().and_then(|lock| {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        guard.dll_funcs.pfnSpawn
    });
    if let Some(f) = func {
        unsafe { f(edict) }
    } else {
        0
    }
}

/// Forward a client connect call to the real game DLL if loaded.
pub fn forward_client_connect(
    edict: *mut goldsrc_sys::edict_t,
    name: *const std::os::raw::c_char,
    address: *const std::os::raw::c_char,
    reject_reason: *mut std::os::raw::c_char,
) -> i32 {
    let func = PROXY.get().and_then(|lock| {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        guard.dll_funcs.pfnClientConnect
    });
    if let Some(f) = func {
        unsafe { f(edict, name, address, reject_reason) }
    } else {
        1 // Allow by default if no real DLL
    }
}

/// Forward a client disconnect call to the real game DLL if loaded.
pub fn forward_client_disconnect(edict: *mut goldsrc_sys::edict_t) {
    let func = PROXY.get().and_then(|lock| {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        guard.dll_funcs.pfnClientDisconnect
    });
    if let Some(f) = func {
        unsafe { f(edict) };
    }
}

/// Forward a client command call to the real game DLL if loaded.
pub fn forward_client_command(edict: *mut goldsrc_sys::edict_t) {
    let func = PROXY.get().and_then(|lock| {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        guard.dll_funcs.pfnClientCommand
    });
    if let Some(f) = func {
        unsafe { f(edict) };
    }
}

/// Forward a game init call to the real game DLL if loaded.
pub fn forward_game_init() {
    let func = PROXY.get().and_then(|lock| {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        guard.dll_funcs.pfnGameInit
    });
    if let Some(f) = func {
        unsafe { f() };
    }
}

/// Forward a start frame call to the real game DLL if loaded.
pub fn forward_start_frame() {
    let func = PROXY.get().and_then(|lock| {
        let guard = lock.lock().unwrap_or_else(|e| e.into_inner());
        guard.dll_funcs.pfnStartFrame
    });
    if let Some(f) = func {
        unsafe { f() };
    }
}

/// Forward Server_GetBlendingInterface call to the real game DLL.
///
/// # Safety
/// Pointers must be valid or null as accepted by HLSDK.
pub unsafe fn forward_server_get_blending_interface(
    version: i32,
    ppinterface: *mut *mut std::ffi::c_void,
    pstudio: *mut std::ffi::c_void,
    rotationmatrix: *mut std::ffi::c_void,
    bonetransform: *mut std::ffi::c_void,
) -> i32 {
    ensure_loaded();
    if let Some(proxy_lock) = PROXY.get() {
        let guard = proxy_lock.lock().unwrap_or_else(|e| e.into_inner());
        let func: Result<
            libloading::Symbol<
                unsafe extern "C" fn(
                    i32,
                    *mut *mut std::ffi::c_void,
                    *mut std::ffi::c_void,
                    *mut std::ffi::c_void,
                    *mut std::ffi::c_void,
                ) -> i32,
            >,
            _,
        > = guard._lib.get(b"Server_GetBlendingInterface\0");
        if let Ok(f) = func {
            return f(version, ppinterface, pstudio, rotationmatrix, bonetransform);
        }
    }
    0
}

/// Forward an entity factory function call (e.g. worldspawn, info_player_start) to the real game DLL.
///
/// # Safety
/// `pev` is a valid pointer to an `entvars_t` allocated by the engine.
pub unsafe fn forward_entity(name: &str, pev: *mut goldsrc_sys::entvars_t) {
    ensure_loaded();
    if let Some(proxy_lock) = PROXY.get() {
        let guard = proxy_lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut name_bytes = name.as_bytes().to_vec();
        name_bytes.push(0);
        let func: Result<libloading::Symbol<unsafe extern "C" fn(*mut goldsrc_sys::entvars_t)>, _> =
            guard._lib.get(&name_bytes[..]);
        if let Ok(f) = func {
            f(pev);
        }
    }
}

/// Forward CreateInterface call to the real game DLL if available.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
pub unsafe fn forward_create_interface(
    name: *const std::os::raw::c_char,
    return_code: *mut i32,
) -> *mut std::ffi::c_void {
    ensure_loaded();
    if let Some(proxy_lock) = PROXY.get() {
        let guard = proxy_lock.lock().unwrap_or_else(|e| e.into_inner());
        let func: Result<
            libloading::Symbol<
                unsafe extern "C" fn(
                    *const std::os::raw::c_char,
                    *mut i32,
                ) -> *mut std::ffi::c_void,
            >,
            _,
        > = guard._lib.get(b"CreateInterface\0");
        if let Ok(f) = func {
            return f(name, return_code);
        }
    }
    std::ptr::null_mut()
}
