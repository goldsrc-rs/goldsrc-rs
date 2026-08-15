//! GoldSrc.rs Standalone Backend — proxy GameDLL loader.
//!
//! Loads the real game DLL (`mp.dll` / `cs.so`) and forwards all standard
//! GameDLL exports to it, while inserting our hooks around the key callbacks.

use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use goldsrc_sys::{enginefuncs_t, globalvars_t, DLL_FUNCTIONS};

pub fn file_log(msg: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(goldsrc::paths::PathResolver::debug_log_path())
    {
        let _ = writeln!(file, "[Standalone] {}", msg);
    }
}

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
    /// Optional NEW_DLL_FUNCTIONS table (opaque raw buffer).
    pub new_dll_funcs_buf: [u8; 512],
    pub has_new_dll_funcs: bool,
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
    let dll_path = resolve_game_dll_path();
    file_log(&format!(
        "Attempting to load real GameDLL from path: {:?}",
        dll_path
    ));

    let result = unsafe { try_load_game_dll(&dll_path, engfuncs, globals) };

    match result {
        Ok(proxy) => {
            file_log(&format!(
                "Successfully loaded real GameDLL from {:?}",
                dll_path
            ));
            let _ = PROXY.set(std::sync::Mutex::new(proxy));
            true
        }
        Err(e) => {
            file_log(&format!(
                "ERROR: Failed to load real GameDLL from '{:?}': {}",
                dll_path, e
            ));
            eprintln!(
                "[GoldSrc.rs Standalone] WARNING: Failed to load real game DLL '{}': {}",
                dll_path.display(),
                e
            );
            false
        }
    }
}

/// Resolve the path to the real game DLL universally across any GoldSrc mod (CS 1.6, Sven Co-op, Half-Life, etc.).
fn resolve_game_dll_path() -> PathBuf {
    // 1. Try reading original gamedll path from liblist.gam if present
    for liblist in &[
        "cstrike/liblist.gam",
        "liblist.gam",
        "svencoop/liblist.gam",
        "valve/liblist.gam",
    ] {
        let p = PathBuf::from(liblist);
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    // Look for commented-out original line or gamedll line that is NOT our standalone binary
                    let clean = if trimmed.starts_with(';') {
                        trimmed.trim_start_matches(';').trim()
                    } else {
                        trimmed
                    };

                    if clean.starts_with("gamedll") || clean.starts_with("gamedll_linux") {
                        if clean.contains("goldsrc_standalone") {
                            continue;
                        }
                        // Extract quoted path
                        if let Some(start) = clean.find('"') {
                            if let Some(end) = clean[start + 1..].find('"') {
                                let orig_path = &clean[start + 1..start + 1 + end];
                                let target_path = PathBuf::from(orig_path);
                                if target_path.exists() {
                                    file_log(&format!(
                                        "Resolved original GameDLL from liblist.gam: {:?}",
                                        target_path
                                    ));
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
                file_log(&format!("Resolved GameDLL via mod search: {:?}", candidate));
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
                    file_log(&format!(
                        "Resolved GameDLL via exe base search: {:?}",
                        candidate
                    ));
                    return candidate;
                }
            }
        }
    }

    PathBuf::from("cstrike")
        .join("dlls")
        .join(GAME_DLL_NAMES[0])
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
            unsafe extern "C" fn(*mut enginefuncs_t, *mut globalvars_t),
        > = lib.get(b"GiveFnptrsToDll\0")?;
        give_fns(engfuncs, globals);

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
                file_log("Successfully populated DLL_FUNCTIONS via GetEntityAPI2");
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
                    file_log("Successfully populated DLL_FUNCTIONS via GetEntityAPI");
                    loaded_api = true;
                }
            }
        }

        if !loaded_api {
            file_log(
                "WARNING: Neither GetEntityAPI2 nor GetEntityAPI returned 1 for real GameDLL!",
            );
        }

        // Retrieve NEW_DLL_FUNCTIONS if available
        let mut new_dll_funcs_buf = [0u8; 512];
        let mut new_iface_ver: i32 = 1;
        let get_new_dll_fns: Result<
            libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *mut i32) -> i32>,
            _,
        > = lib.get(b"GetNewDLLFunctions\0");

        let has_new_dll_funcs = if let Ok(f) = get_new_dll_fns {
            let ret = f(
                new_dll_funcs_buf.as_mut_ptr() as *mut std::ffi::c_void,
                &mut new_iface_ver,
            );
            if ret != 0 {
                file_log("Successfully populated NEW_DLL_FUNCTIONS via GetNewDLLFunctions");
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
            new_dll_funcs_buf,
            has_new_dll_funcs,
            loaded: true,
        })
    }
}

/// Copy the real game DLL's function table into the provided table pointer.
pub fn populate_dll_table(dll_table: *mut DLL_FUNCTIONS) {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            unsafe {
                *dll_table = proxy.dll_funcs;
            }
        }
    }
}

/// Copy NEW_DLL_FUNCTIONS from real GameDLL if present.
pub fn populate_new_dll_table(new_dll_table: *mut std::ffi::c_void) -> bool {
    if let Some(proxy_lock) = PROXY.get() {
        if let Ok(proxy) = proxy_lock.lock() {
            if proxy.has_new_dll_funcs && !new_dll_table.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        proxy.new_dll_funcs_buf.as_ptr(),
                        new_dll_table as *mut u8,
                        512,
                    );
                }
                return true;
            }
        }
    }
    false
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
