//! GoldSrc.rs Standalone Backend
//!
//! A proxy GameDLL that loads via `liblist.gam` → `gamedll` key.
//! Receives engine function pointers directly from `hlds.exe`/`hlds_linux`,
//! loads and proxies the real `mp.dll`/`cs.so`, and runs WASM plugins.
//!
//! # Server Setup
//! In `cstrike/liblist.gam`, change:
//! ```text
//! gamedll "dlls/mp.dll"
//! ```
//! to:
//! ```text
//! gamedll "addons/goldsrc/bin/goldsrc_standalone.dll"
//! ```

mod commands;
mod engine_api;
mod entities;
mod proxy;

use goldsrc::backend::EngineBackend;
use goldsrc::log;
use goldsrc_sys::ffi::catch_ffi_panic;
use goldsrc_sys::{DLL_FUNCTIONS, enginefuncs_t, globalvars_t};

// ============================================================================
// Backend (Engine trait implementation)
// ============================================================================

/// Standalone backend: the shared `EngineBackend` fed by this crate's engfunc
/// accessor and print queue. The backend is a thin adapter.
pub type StandaloneBackend = EngineBackend;

static PRINT_QUEUE: goldsrc::backend::PrintQueue = goldsrc::backend::PrintQueue::new();

static BACKEND: StandaloneBackend = EngineBackend::new(engine_api::engfuncs, &PRINT_QUEUE);

pub fn backend() -> &'static StandaloneBackend {
    &BACKEND
}

pub use goldsrc::call_engfunc;
pub use goldsrc::call_engfunc_ret;

// ============================================================================
// WASM Host initialization
// ============================================================================

fn init_wasm_host() {
    goldsrc::backend::set_map_name_resolver(|| {
        if let Some(globals) = engine_api::try_globals() {
            let mapname_str_offset = globals.mapname;
            if mapname_str_offset != 0 {
                // 1. Direct memory resolution via pStringBase (standard HLSDK STRING() macro)
                if !globals.pStringBase.is_null()
                    && (mapname_str_offset as usize) <= goldsrc_sys::ffi::STRING_POOL_MASK
                {
                    let ptr = unsafe {
                        (globals.pStringBase as *const u8).add(mapname_str_offset as usize)
                            as *const std::os::raw::c_char
                    };
                    if let Some(name) = unsafe { goldsrc_sys::ffi::cstr_to_string_bounded(ptr, 64) }
                    {
                        return Some(name);
                    }
                }
                // 2. Engine string table resolver via pfnSzFromIndex
                if let Some(sz_fn) = engine_api::try_engfuncs().and_then(|ef| ef.pfnSzFromIndex) {
                    let ptr = unsafe { sz_fn(mapname_str_offset as i32) };
                    if let Some(name) = unsafe { goldsrc_sys::ffi::cstr_to_string_bounded(ptr, 64) }
                    {
                        return Some(name);
                    }
                }
            }
        }
        None
    });

    let engine: std::sync::Arc<dyn goldsrc_api::Engine> = std::sync::Arc::new(*backend());
    if let Err(e) = goldsrc::host::HostRuntime::init(
        goldsrc_api::consts::BackendType::Standalone,
        |msg| {
            backend().server_print(msg);
        },
        engine,
    ) {
        log::error!(target: "core", "{e}");
    }
}

// ============================================================================
// Hook strategy (backend behavior for each DLL_FUNCTIONS slot)
// ============================================================================

/// Standalone strategy: forward every call to the real GameDLL via
/// [`proxy`], then run framework business logic around it.
pub struct StandaloneHooks;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl goldsrc::api_registry::EntityHooks for StandaloneHooks {
    fn game_init(&self) {
        // 1. Forward GameDLLInit to real GameDLL
        proxy::forward_game_init();
        // 2. Initialize WASM host and plugins
        init_wasm_host();
        // 3. Register CLI commands after engine command system is initialized
        commands::register_cli_commands();
        log::info!(target: "core", "hook_game_init: WASM host & commands initialized successfully");
    }

    fn spawn(&self, edict: *mut goldsrc_sys::edict_t) -> i32 {
        crate::backend().precache_pending_resources();
        let ret = proxy::forward_spawn(edict);
        let index = unsafe { goldsrc::api_registry::edict_index(edict) };
        if index >= 0 {
            let _ = goldsrc::hooks::entity_hooks().read().map(|reg| {
                reg.dispatch_generic(
                    goldsrc_api::gamedata::VTableFunc::Spawn,
                    index,
                    goldsrc::hooks::HookTiming::Post,
                )
            });
        }
        ret
    }

    fn server_activate(
        &self,
        edict_list: *mut goldsrc_sys::edict_t,
        edict_count: i32,
        client_max: i32,
    ) {
        proxy::forward_server_activate(edict_list, edict_count, client_max);
        goldsrc::hooks::on_server_activate();
    }

    fn server_deactivate(&self) {
        proxy::forward_server_deactivate();
        goldsrc::hooks::on_server_deactivate();
    }

    fn client_connect(
        &self,
        edict: *mut goldsrc_sys::edict_t,
        index: i32,
        name: *const std::ffi::c_char,
        address: *const std::ffi::c_char,
        reject_reason: *mut std::ffi::c_char,
    ) -> i32 {
        let result = proxy::forward_client_connect(edict, name, address, reject_reason);
        if result != 0 {
            goldsrc::hooks::emit_player_event("client_connect", index);
        }
        result as i32
    }

    fn client_disconnect(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_client_disconnect(edict);
        goldsrc::hooks::emit_player_event("client_disconnect", index);
    }

    fn client_command(
        &self,
        edict: *mut goldsrc_sys::edict_t,
        index: i32,
        cmd: &str,
        args: &str,
    ) -> bool {
        let handled = goldsrc::hooks::dispatch_client_command(index, cmd, args);
        if !handled {
            proxy::forward_client_command(edict);
        }
        handled
    }

    fn start_frame(&self) {
        proxy::forward_start_frame();
        goldsrc::hooks::on_server_frame();
        crate::backend().drain_prints();
    }

    fn player_pre_think(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_player_pre_think(edict);
        goldsrc::hooks::emit_player_event("player_pre_think", index);
    }

    fn player_post_think(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_player_post_think(edict);
        goldsrc::hooks::emit_player_event("player_post_think", index);
    }

    fn cmd_start(
        &self,
        player: *const goldsrc_sys::edict_t,
        index: i32,
        cmd: *const goldsrc_sys::usercmd_s,
        random_seed: u32,
    ) {
        proxy::forward_cmd_start(player, cmd, random_seed);
        if !cmd.is_null() {
            let buttons = unsafe { (*cmd).buttons };
            let mut payload = [0u8; 8];
            payload[0..4].copy_from_slice(&index.to_le_bytes());
            payload[4..6].copy_from_slice(&buttons.to_le_bytes());
            goldsrc::hooks::emit_event("cmd_start", &payload);
        } else {
            goldsrc::hooks::emit_player_event("cmd_start", index);
        }
    }

    fn cmd_end(&self, player: *const goldsrc_sys::edict_t, index: i32) {
        proxy::forward_cmd_end(player);
        goldsrc::hooks::emit_player_event("cmd_end", index);
    }

    fn add_to_full_pack(
        &self,
        state: *mut goldsrc_sys::entity_state_s,
        e: i32,
        ent: *mut goldsrc_sys::edict_t,
        host: *mut goldsrc_sys::edict_t,
        hostflags: i32,
        player: i32,
        pset: *mut u8,
    ) -> i32 {
        proxy::forward_add_to_full_pack(state, e, ent, host, hostflags, player, pset)
    }

    fn client_kill(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_client_kill(edict);
        goldsrc::hooks::emit_player_event("client_kill", index);
    }

    fn touch(
        &self,
        touched: *mut goldsrc_sys::edict_t,
        touched_idx: i32,
        other: *mut goldsrc_sys::edict_t,
        other_idx: i32,
    ) {
        proxy::forward_touch(touched, other);
        goldsrc::hooks::emit_event(
            "entity_touch",
            &goldsrc::api_registry::pack_two_i32(touched_idx, other_idx),
        );
    }

    fn entity_use(
        &self,
        used: *mut goldsrc_sys::edict_t,
        used_idx: i32,
        other: *mut goldsrc_sys::edict_t,
        other_idx: i32,
    ) {
        proxy::forward_use(used, other);
        goldsrc::hooks::emit_event(
            "entity_use",
            &goldsrc::api_registry::pack_two_i32(used_idx, other_idx),
        );
    }

    fn client_put_in_server(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_client_put_in_server(edict);
        goldsrc::hooks::emit_player_event("client_put_in_server", index);
    }

    fn client_user_info_changed(
        &self,
        edict: *mut goldsrc_sys::edict_t,
        _index: i32,
        infobuffer: *mut std::ffi::c_char,
    ) {
        proxy::forward_client_user_info_changed(edict, infobuffer);
    }

    fn think(&self, edict: *mut goldsrc_sys::edict_t, _index: i32) {
        proxy::forward_think(edict);
    }

    fn blocked(
        &self,
        blocked: *mut goldsrc_sys::edict_t,
        _b_idx: i32,
        other: *mut goldsrc_sys::edict_t,
        _o_idx: i32,
    ) {
        proxy::forward_blocked(blocked, other);
    }

    fn key_value(
        &self,
        edict: *mut goldsrc_sys::edict_t,
        _index: i32,
        pkvd: *mut goldsrc_sys::KeyValueData,
    ) {
        proxy::forward_key_value(edict, pkvd);
    }

    fn set_abs_box(&self, edict: *mut goldsrc_sys::edict_t, _index: i32) {
        proxy::forward_set_abs_box(edict);
    }

    fn update_client_data(
        &self,
        edict: *const goldsrc_sys::edict_t,
        _index: i32,
        sendweapons: i32,
        cd: *mut goldsrc_sys::clientdata_s,
    ) {
        proxy::forward_update_client_data(edict, sendweapons, cd);
    }

    fn spectator_connect(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_spectator_connect(edict);
        goldsrc::hooks::emit_player_event("spectator_connect", index);
    }

    fn spectator_disconnect(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_spectator_disconnect(edict);
        goldsrc::hooks::emit_player_event("spectator_disconnect", index);
    }

    fn spectator_think(&self, edict: *mut goldsrc_sys::edict_t, index: i32) {
        proxy::forward_spectator_think(edict);
        goldsrc::hooks::emit_player_event("spectator_think", index);
    }

    fn sys_error(&self, error_string: &str) {
        if let Ok(c_str) = std::ffi::CString::new(error_string) {
            proxy::forward_sys_error(c_str.as_ptr());
        }
    }

    fn pm_move(&self, ppmove: *mut goldsrc_sys::playermove_s, server: goldsrc_sys::qboolean) {
        proxy::forward_pm_move(ppmove, server);
    }

    fn setup_visibility(
        &self,
        view_ent: *mut goldsrc_sys::edict_t,
        client: *mut goldsrc_sys::edict_t,
        pvs: *mut *mut u8,
        pas: *mut *mut u8,
    ) {
        proxy::forward_setup_visibility(view_ent, client, pvs, pas);
    }

    fn inconsistent_file(
        &self,
        player: *const goldsrc_sys::edict_t,
        _index: i32,
        filename: &str,
        disconnect_message: *mut std::ffi::c_char,
    ) -> i32 {
        if let Ok(c_filename) = std::ffi::CString::new(filename) {
            proxy::forward_inconsistent_file(player, c_filename.as_ptr(), disconnect_message)
        } else {
            0
        }
    }

    fn allow_lag_compensation(&self) -> i32 {
        proxy::forward_allow_lag_compensation()
    }
}

/// Static hook instance handed to the registry in `GiveFnptrsToDll`.
pub static HOOKS: StandaloneHooks = StandaloneHooks;

// ============================================================================
// GameDLL Entry Points (loaded via liblist.gam `gamedll` key)
// ============================================================================

/// Called by the engine immediately after loading the DLL.
/// Provides engine function pointers and global variables.
///
/// # Safety
/// Pointers are provided by `hlds.exe` / `hlds_linux` and are always valid at this point.
/// Any Rust panic is caught — an unhandled panic here would crash HLDS.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "system" fn GiveFnptrsToDll(
    engfuncs: *mut enginefuncs_t,
    globals: *mut globalvars_t,
) {
    // SAFETY: engfuncs and globals are engine-provided; valid for the server lifetime.
    catch_ffi_panic("GiveFnptrsToDll", (), || unsafe {
        goldsrc_sys::guard::install_crash_guard();
        engine_api::init(engfuncs, globals);
        // Register the unified hook strategy before the engine queries our tables.
        goldsrc::api_registry::register(goldsrc::api_registry::Registry {
            hooks: &HOOKS,
            engfuncs: engine_api::engfuncs,
        });
        proxy::forward_give_fnptrs_to_dll(engfuncs, globals);
        // Expose real GameDLL entry points for direct calls (give_item etc.).
        if let Some(f) = proxy::real_dispatch_spawn() {
            goldsrc::backend::set_game_dll_spawn(f);
        }
        if let Some(f) = proxy::real_touch() {
            goldsrc::backend::set_game_dll_touch(f);
        }
    });
}

/// Called by the engine to retrieve our `DLL_FUNCTIONS` hook table.
///
/// # Safety
/// `dll_table` is a valid pointer provided by the engine.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI2(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: *mut i32,
) -> i32 {
    // SAFETY: dll_table and interface_version are engine-provided; valid at call time.
    catch_ffi_panic("GetEntityAPI2", 0, || {
        if dll_table.is_null() {
            return 0;
        }
        // Standalone overlays its own hook table on top of the real GameDLL's,
        // so a version mismatch is reported but does not abort registration.
        // SAFETY: interface_version is a valid engine-provided pointer.
        if !(unsafe {
            goldsrc::api_registry::negotiate_interface_version(interface_version, false)
        }) {
            return 0;
        }
        unsafe {
            // 1. Populate with real GameDLL (mp.dll / cs.so) callbacks.
            proxy::populate_dll_table(dll_table);

            // 2. Overlay our hooks — single registration point.
            goldsrc::api_registry::install_dll_api2(dll_table);
            (*dll_table).pfnGameInit = Some(goldsrc::api_registry::api_game_init);
            (*dll_table).pfnServerActivate = Some(goldsrc::api_registry::api_server_activate);
            (*dll_table).pfnClientConnect = Some(goldsrc::api_registry::api_client_connect);
            (*dll_table).pfnClientDisconnect = Some(goldsrc::api_registry::api_client_disconnect);
            (*dll_table).pfnSpawn = Some(goldsrc::api_registry::api_spawn);
        }
        1
    })
}

/// Old-style entity API (required for compatibility with some engine versions).
///
/// # Safety
/// `dll_table` must be a valid pointer.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetEntityAPI(
    dll_table: *mut DLL_FUNCTIONS,
    interface_version: i32,
) -> i32 {
    catch_ffi_panic("GetEntityAPI", 0, || {
        if dll_table.is_null() {
            return 0;
        }
        let mut ver = interface_version;
        // SAFETY: dll_table is valid, verified above.
        unsafe { GetEntityAPI2(dll_table, &mut ver) }
    })
}

/// Called by engine for NEW_DLL_FUNCTIONS interface (ReGameDLL, Sven Co-op, HLSDK 2.x).
///
/// # Safety
/// Pointers must be valid.
/// Any Rust panic is caught — the C caller receives 0 instead of UB.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn GetNewDLLFunctions(
    new_dll_table: *mut std::ffi::c_void,
    interface_version: *mut i32,
) -> i32 {
    catch_ffi_panic("GetNewDLLFunctions", 0, || {
        if new_dll_table.is_null() {
            return 0;
        }
        unsafe {
            if !interface_version.is_null() {
                *interface_version = goldsrc_api::consts::NEW_DLL_INTERFACE_VERSION;
            }
        }
        if proxy::populate_new_dll_table(new_dll_table) {
            1
        } else {
            0
        }
    })
}

/// Called by engine to query the studio model animation blending interface.
///
/// # Safety
/// Pointers must be valid or null as accepted by HLSDK.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn Server_GetBlendingInterface(
    version: i32,
    ppinterface: *mut *mut std::ffi::c_void,
    pstudio: *mut std::ffi::c_void,
    rotationmatrix: *mut std::ffi::c_void,
    bonetransform: *mut std::ffi::c_void,
) -> i32 {
    catch_ffi_panic("Server_GetBlendingInterface", 0, || unsafe {
        proxy::forward_server_get_blending_interface(
            version,
            ppinterface,
            pstudio,
            rotationmatrix,
            bonetransform,
        )
    })
}

/// Called by engine/modules to query abstract interfaces (e.g. VServerAdmin).
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[unsafe(no_mangle)]
#[inline(never)]
pub unsafe extern "C" fn CreateInterface(
    name: *const std::os::raw::c_char,
    return_code: *mut i32,
) -> *mut std::ffi::c_void {
    catch_ffi_panic("CreateInterface", std::ptr::null_mut(), || unsafe {
        proxy::forward_create_interface(name, return_code)
    })
}
