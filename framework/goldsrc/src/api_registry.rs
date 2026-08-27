//! Unified FFI registration point for engine-facing function tables.
//!
//! This module is the single place where `DLL_FUNCTIONS` hook tables are
//! populated, replacing per-backend hand-written table filling. It separates
//! three concerns:
//!
//! 1. **FFI boundary** — `extern "C"` trampolines + panic guards (here).
//! 2. **Engine interaction** — edict→index resolution, client command reading.
//! 3. **Backend strategy** — [`EntityHooks`] impl per backend (forward to real
//!    GameDLL, metamod MRES handling, event emission, ...).
//!
//! Backends implement [`EntityHooks`] once, register it via [`register`],
//! and let [`install_dll_api2`] / [`install_dll_api2_post`] fill the tables.

use goldsrc_sys::{DLL_FUNCTIONS, edict_t, enginefuncs_t, qboolean};
use std::ffi::c_char;
use std::os::raw::c_int;
use std::sync::OnceLock;

use goldsrc_sys::ffi::catch_ffi_panic;

/// Backend-specific behavior for each hooked `DLL_FUNCTIONS` slot.
///
/// All methods default to no-ops; backends override only what they need.
/// Business logic (WASM events / command dispatch) is invoked explicitly by
/// each impl through [`crate::hooks`], keeping this trait free of hidden magic.
///
/// Raw pointers are forwarded as-is because backends may need to pass them to
/// the real GameDLL or metamod; precomputed indexes/strings save impls from
/// repeating engine reads.
#[allow(unused_variables)]
pub trait EntityHooks: Send + Sync {
    // --- DLL_FUNCTIONS (pre-hooks / direct GameDLL replacement) ---

    /// `pfnGameInit` — server startup.
    fn game_init(&self) {}
    /// `pfnSpawn` — entity spawn. Returns the value passed back to the engine.
    fn spawn(&self, edict: *mut edict_t) -> i32 {
        0
    }
    /// `pfnServerActivate` — map loaded and activated.
    fn server_activate(&self, edict_list: *mut edict_t, edict_count: i32, client_max: i32) {}
    /// `pfnServerDeactivate` — map ending / shutdown.
    fn server_deactivate(&self) {}
    /// `pfnClientConnect`. Returns the connect verdict (0 = reject).
    fn client_connect(
        &self,
        edict: *mut edict_t,
        index: i32,
        name: *const c_char,
        address: *const c_char,
        reject_reason: *mut c_char,
    ) -> i32 {
        0
    }
    /// `pfnClientDisconnect`.
    fn client_disconnect(&self, edict: *mut edict_t, index: i32) {}
    /// `pfnClientCommand`. Returns `true` if a plugin consumed the command
    /// (backend decides how to suppress the GameDLL, e.g. proxy skip or MRES).
    fn client_command(&self, edict: *mut edict_t, index: i32, cmd: &str, args: &str) -> bool {
        false
    }
    /// `pfnStartFrame` — every server frame.
    fn start_frame(&self) {}
    /// `pfnPlayerPostThink`.
    fn player_post_think(&self, edict: *mut edict_t, index: i32) {}
    /// `pfnClientKill` — suicide command.
    fn client_kill(&self, edict: *mut edict_t, index: i32) {}
    /// `pfnTouch` — two entities collided.
    fn touch(&self, touched: *mut edict_t, touched_idx: i32, other: *mut edict_t, other_idx: i32) {}
    /// `pfnUse` — entity activated/triggered.
    fn entity_use(&self, used: *mut edict_t, used_idx: i32, other: *mut edict_t, other_idx: i32) {}

    // --- DLL_FUNCTIONS (post-hooks; used by metamod chaining) ---

    /// Post-`pfnSpawn`.
    fn spawn_post(&self, edict: *mut edict_t) {}
    /// Post-`pfnClientConnect`.
    fn client_connect_post(&self, index: i32) {}
    /// Post-`pfnClientDisconnect`.
    fn client_disconnect_post(&self, index: i32) {}
    /// Post-`pfnStartFrame`.
    fn start_frame_post(&self) {}
}

/// Everything a backend provides so trampolines can resolve engine state.
pub struct Registry {
    pub hooks: &'static dyn EntityHooks,
    /// Engine func-table accessor (same one handed to `EngineBackend::new`).
    pub engfuncs: fn() -> &'static enginefuncs_t,
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();
static WARNED_UNREGISTERED: OnceLock<()> = OnceLock::new();

/// Registers the backend hook implementation. Idempotent: first call wins.
pub fn register(config: Registry) {
    let _ = REGISTRY.set(config);
}

fn hooks() -> Option<&'static dyn EntityHooks> {
    match REGISTRY.get() {
        Some(r) => Some(r.hooks),
        // Silent no-op here would make an unregistered backend look like a
        // plugin bug ("events don't fire") — surface the real cause once.
        None => {
            if WARNED_UNREGISTERED.set(()).is_ok() {
                log::warn!(target: "core",
                    "api_registry: hook fired before register() — event dropped");
            }
            None
        }
    }
}

fn engfuncs() -> Option<&'static enginefuncs_t> {
    REGISTRY.get().map(|r| (r.engfuncs)())
}

/// HLSDK interface-version negotiation for `GetEntityAPI2`-style entry points.
///
/// Writes our supported version back so the engine can report the mismatch.
/// With `strict = true` (HLSDK-correct policy, used by metamod) a mismatch
/// also aborts registration; with `strict = false` (standalone proxy, which
/// overlays its own table anyway) only the pointer write is performed.
///
/// # Safety
/// `interface_version` must be null or point to a valid `i32` provided by the
/// engine for version exchange.
pub unsafe fn negotiate_interface_version(interface_version: *mut i32, strict: bool) -> bool {
    if interface_version.is_null() {
        return !strict;
    }
    // SAFETY: non-null pointer provided by the engine for version exchange.
    unsafe {
        if *interface_version != goldsrc_api::consts::ENGINE_INTERFACE_VERSION {
            *interface_version = goldsrc_api::consts::ENGINE_INTERFACE_VERSION;
            return !strict;
        }
    }
    true
}

static EDICT_BASE: std::sync::atomic::AtomicPtr<edict_t> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());
static EDICT_COUNT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Updates the known active edict array boundaries from `server_activate`.
pub fn update_edict_bounds(base: *mut edict_t, count: i32) {
    EDICT_BASE.store(base, std::sync::atomic::Ordering::Relaxed);
    EDICT_COUNT.store(count, std::sync::atomic::Ordering::Relaxed);
}

/// Resolves an edict pointer to its entity index safely (0 if unresolvable or out-of-bounds).
///
/// # Safety
/// `edict` must be null or a valid edict pointer provided by the engine.
pub unsafe fn edict_index(edict: *mut edict_t) -> i32 {
    if edict.is_null() {
        return 0;
    }
    let base = EDICT_BASE.load(std::sync::atomic::Ordering::Relaxed);
    let count = EDICT_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    if !base.is_null() && count > 0 {
        let diff = (edict as usize).wrapping_sub(base as usize);
        let edict_size = std::mem::size_of::<edict_t>();
        if edict_size > 0 && diff.is_multiple_of(edict_size) {
            let idx = (diff / edict_size) as i32;
            if idx >= 0 && idx < count {
                return idx;
            }
        }
        // Pointer is outside active sv.edicts bounds; return 0 safely without triggering Host_Error
        return 0;
    }

    // If bounds are not yet populated (before server_activate), return 0 safely.
    // Calling engine pfnIndexOfEdict directly triggers Host_Error("IndexOfEdict: bad entity")
    // in ReHLDS if called on non-edict entity pointers during worldspawn/initialization.
    0
}

/// Reads the currently dispatched client command as `(cmd, args)`.
pub fn read_client_command() -> (String, String) {
    let Some(funcs) = engfuncs() else {
        return (String::new(), String::new());
    };
    // SAFETY: engine-provided C strings, valid during ClientCommand dispatch.
    unsafe {
        let cmd = crate::backend::cstr_to_string(
            funcs.pfnCmd_Argv.map(|f| f(0)).unwrap_or(std::ptr::null()),
        );
        let args = crate::backend::cstr_to_string(
            funcs.pfnCmd_Args.map(|f| f()).unwrap_or(std::ptr::null()),
        );
        (cmd, args)
    }
}

/// Packs two entity indexes into an 8-byte little-endian event payload.
pub fn pack_two_i32(a: i32, b: i32) -> [u8; 8] {
    let mut payload = [0u8; 8];
    payload[0..4].copy_from_slice(&a.to_le_bytes());
    payload[4..8].copy_from_slice(&b.to_le_bytes());
    payload
}

// ============================================================================
// Trampolines — the only `extern "C"` hook bodies in the codebase.
// ============================================================================

/// # Safety
/// Engine callback; invoked only through the installed function tables.
pub unsafe extern "C" fn api_game_init() {
    catch_ffi_panic("game_init", (), || {
        if let Some(h) = hooks() {
            h.game_init();
        }
    });
}

/// # Safety
/// Engine callback; `pent` must be a valid edict pointer or null.
pub unsafe extern "C" fn api_spawn(pent: *mut edict_t) -> c_int {
    catch_ffi_panic("spawn", 0, || hooks().map_or(0, |h| h.spawn(pent)))
}

/// # Safety
/// Engine callback; `p_edict_list` must be a valid edict array base.
pub unsafe extern "C" fn api_server_activate(
    p_edict_list: *mut edict_t,
    edict_count: c_int,
    client_max: c_int,
) {
    catch_ffi_panic("server_activate", (), || {
        update_edict_bounds(p_edict_list, edict_count);
        if let Some(h) = hooks() {
            h.server_activate(p_edict_list, edict_count, client_max);
        }
    });
}

/// # Safety
/// Engine callback; invoked only through the installed function tables.
pub unsafe extern "C" fn api_server_deactivate() {
    catch_ffi_panic("server_deactivate", (), || {
        if let Some(h) = hooks() {
            h.server_deactivate();
        }
    });
}

/// # Safety
/// Engine callback; string pointers must be valid C strings per HLSDK contract.
pub unsafe extern "C" fn api_client_connect(
    p_entity: *mut edict_t,
    psz_name: *const c_char,
    psz_address: *const c_char,
    sz_reject_reason: *mut c_char,
) -> qboolean {
    catch_ffi_panic("client_connect", 0, || {
        let index = unsafe { edict_index(p_entity) };
        hooks().map_or(0, |h| {
            h.client_connect(p_entity, index, psz_name, psz_address, sz_reject_reason)
        })
    })
}

/// # Safety
/// Engine callback; `p_entity` must be a valid player edict pointer or null.
pub unsafe extern "C" fn api_client_disconnect(p_entity: *mut edict_t) {
    catch_ffi_panic("client_disconnect", (), || {
        if let Some(h) = hooks() {
            let index = unsafe { edict_index(p_entity) };
            h.client_disconnect(p_entity, index);
        }
    });
}

/// # Safety
/// Engine callback; `p_entity` must be a valid edict pointer; command buffer
/// (`pfnCmd_Argv`/`pfnCmd_Args`) must be active.
pub unsafe extern "C" fn api_client_command(p_entity: *mut edict_t) {
    catch_ffi_panic("client_command", (), || {
        if let Some(h) = hooks() {
            let index = unsafe { edict_index(p_entity) };
            let (cmd, args) = read_client_command();
            h.client_command(p_entity, index, &cmd, &args);
        }
    });
}

/// # Safety
/// Engine callback; invoked only through the installed function tables.
pub unsafe extern "C" fn api_start_frame() {
    catch_ffi_panic("start_frame", (), || {
        if let Some(h) = hooks() {
            h.start_frame();
        }
    });
}

/// # Safety
/// Engine callback; `p_entity` must be a valid player edict pointer.
pub unsafe extern "C" fn api_player_post_think(p_entity: *mut edict_t) {
    catch_ffi_panic("player_post_think", (), || {
        if let Some(h) = hooks() {
            let index = unsafe { edict_index(p_entity) };
            h.player_post_think(p_entity, index);
        }
    });
}

/// # Safety
/// Engine callback; `p_entity` must be a valid player edict pointer.
pub unsafe extern "C" fn api_client_kill(p_entity: *mut edict_t) {
    catch_ffi_panic("client_kill", (), || {
        if let Some(h) = hooks() {
            let index = unsafe { edict_index(p_entity) };
            h.client_kill(p_entity, index);
        }
    });
}

/// # Safety
/// Engine callback; both pointers must be valid edicts or null.
pub unsafe extern "C" fn api_touch(pent_touched: *mut edict_t, pent_other: *mut edict_t) {
    catch_ffi_panic("touch", (), || {
        if let Some(h) = hooks() {
            let touched_idx = unsafe { edict_index(pent_touched) };
            let other_idx = unsafe { edict_index(pent_other) };
            h.touch(pent_touched, touched_idx, pent_other, other_idx);
        }
    });
}

/// # Safety
/// Engine callback; both pointers must be valid edicts or null.
pub unsafe extern "C" fn api_use(pent_used: *mut edict_t, pent_other: *mut edict_t) {
    catch_ffi_panic("use", (), || {
        if let Some(h) = hooks() {
            let used_idx = unsafe { edict_index(pent_used) };
            let other_idx = unsafe { edict_index(pent_other) };
            h.entity_use(pent_used, used_idx, pent_other, other_idx);
        }
    });
}

// --- Post trampolines (metamod chaining) ---

/// # Safety
/// Metamod post-hook callback; `pent` must be a valid edict pointer.
pub unsafe extern "C" fn api_spawn_post(pent: *mut edict_t) -> c_int {
    catch_ffi_panic("spawn_post", 0, || {
        if let Some(h) = hooks() {
            h.spawn_post(pent);
        }
        0
    })
}

/// # Safety
/// Metamod post-hook callback; `p_entity` must be a valid edict pointer.
pub unsafe extern "C" fn api_client_connect_post(
    p_entity: *mut edict_t,
    _psz_name: *const c_char,
    _psz_address: *const c_char,
    _sz_reject_reason: *mut c_char,
) -> qboolean {
    catch_ffi_panic("client_connect_post", 1, || {
        if let Some(h) = hooks() {
            let index = unsafe { edict_index(p_entity) };
            h.client_connect_post(index);
        }
        1
    })
}

/// # Safety
/// Metamod post-hook callback; `p_entity` must be a valid edict pointer.
pub unsafe extern "C" fn api_client_disconnect_post(p_entity: *mut edict_t) {
    catch_ffi_panic("client_disconnect_post", (), || {
        if let Some(h) = hooks() {
            let index = unsafe { edict_index(p_entity) };
            h.client_disconnect_post(index);
        }
    });
}

/// # Safety
/// Metamod post-hook callback; invoked only through the installed tables.
pub unsafe extern "C" fn api_start_frame_post() {
    catch_ffi_panic("start_frame_post", (), || {
        if let Some(h) = hooks() {
            h.start_frame_post();
        }
    });
}

// ============================================================================
// Table installers & Unified ApiRegistry Facade.
// ============================================================================

/// Hook phase defining which table slots (pre-hooks or post-hooks) to populate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPhase {
    /// Pre-hook / GameDLL replacement table (GetEntityAPI / GetEntityAPI2).
    Pre,
    /// Post-hook / Metamod chaining table (GetEntityAPI_Post / GetEntityAPI2_Post).
    Post,
}

/// Unified API registration facade for backends.
pub struct ApiRegistry;

impl ApiRegistry {
    /// Registers the backend hook implementation. Idempotent: first call wins.
    pub fn register(hooks: &'static dyn EntityHooks, engfuncs: fn() -> &'static enginefuncs_t) {
        register(Registry { hooks, engfuncs });
    }

    /// Negotiates the interface version and installs the corresponding hook trampolines.
    ///
    /// # Safety
    /// `table` and `interface_version` must be valid non-null pointers provided by the engine/metamod.
    pub unsafe fn install(
        table: *mut DLL_FUNCTIONS,
        interface_version: *mut i32,
        phase: HookPhase,
        strict: bool,
    ) -> bool {
        if table.is_null() {
            return false;
        }
        // SAFETY: caller guarantees interface_version validity.
        if !unsafe { negotiate_interface_version(interface_version, strict) } {
            return false;
        }
        // SAFETY: caller guarantees table validity.
        unsafe {
            match phase {
                HookPhase::Pre => install_dll_api2(table),
                HookPhase::Post => install_dll_api2_post(table),
            }
        }
        true
    }
}

/// Fills a `DLL_FUNCTIONS` table with all pre-hook trampolines.
///
/// # Safety
/// `table` must point to a writable `DLL_FUNCTIONS` owned by the caller.
pub unsafe fn install_dll_api2(table: *mut DLL_FUNCTIONS) {
    // SAFETY: caller guarantees table validity.
    unsafe {
        let t = &mut *table;
        t.pfnServerDeactivate = Some(api_server_deactivate);
        t.pfnClientCommand = Some(api_client_command);
        t.pfnStartFrame = Some(api_start_frame);
        t.pfnPlayerPostThink = Some(api_player_post_think);
        t.pfnClientKill = Some(api_client_kill);
    }
}

/// Fills a `DLL_FUNCTIONS` table with all post-hook trampolines.
///
/// # Safety
/// `table` must point to a writable `DLL_FUNCTIONS` owned by the caller.
pub unsafe fn install_dll_api2_post(table: *mut DLL_FUNCTIONS) {
    // SAFETY: caller guarantees table validity.
    unsafe {
        let t = &mut *table;
        t.pfnServerActivate = Some(api_server_activate);
        t.pfnClientConnect = Some(api_client_connect_post);
        t.pfnClientDisconnect = Some(api_client_disconnect_post);
        t.pfnStartFrame = Some(api_start_frame_post);
    }
}
