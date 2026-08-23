//! Engine hook callbacks for the Metamod backend.

use goldsrc_sys::ffi::catch_ffi_panic;

use crate::{PRINT_QUEUE, call_engfunc, call_engfunc_ret, engfuncs};

/// Hook for DispatchSpawn - called when an entity spawns.
///
/// # Safety
/// `edict` must be a valid pointer to an edict_t.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_spawn(_edict: *mut goldsrc_sys::edict_t) -> i32 {
    catch_ffi_panic("hook_spawn", 0, || 0)
}

/// Post-hook for DispatchSpawn.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_spawn_post(_edict: *mut goldsrc_sys::edict_t) -> i32 {
    catch_ffi_panic("hook_spawn_post", 0, || 0)
}

/// Post-hook for StartFrame.
pub unsafe extern "C" fn hook_start_frame_post() {
    catch_ffi_panic("hook_start_frame_post", (), || {
        for message in PRINT_QUEUE.drain() {
            if let Ok(msg) = std::ffi::CString::new(message) {
                unsafe {
                    call_engfunc!(engfuncs().pfnServerPrint, msg.as_ptr());
                }
            }
        }
    });
}

/// Hook for ClientConnect - called when a player connects.
///
/// # Safety
/// Pointers must be valid C strings.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_client_connect(
    _entity: *mut goldsrc_sys::edict_t,
    _name: *const std::os::raw::c_char,
    _address: *const std::os::raw::c_char,
    _reject_reason: *mut std::os::raw::c_char,
) -> goldsrc_sys::qboolean {
    // SAFETY: catch_unwind guards the ABI boundary.
    catch_ffi_panic("hook_client_connect", 0, || 0)
}

/// Post-hook for ClientConnect - called when a player connects.
///
/// # Safety
/// Pointers must be valid C strings.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_client_connect_post(
    entity: *mut goldsrc_sys::edict_t,
    _name: *const std::os::raw::c_char,
    _address: *const std::os::raw::c_char,
    _reject_reason: *mut std::os::raw::c_char,
) -> goldsrc_sys::qboolean {
    // SAFETY: catch_unwind guards the ABI boundary.
    catch_ffi_panic("hook_client_connect_post", 0, || {
        // SAFETY: `entity` is a valid edict pointer provided by the engine.
        let index = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, entity) };
        goldsrc::hooks::emit_player_event("client_connect", index);
        0
    })
}

/// Hook for ClientDisconnect - called when a player disconnects.
///
/// # Safety
/// `entity` must be valid player edict pointer or null.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_client_disconnect(_entity: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_disconnect", (), || ());
}

/// Post-hook for ClientDisconnect.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_client_disconnect_post(entity: *mut goldsrc_sys::edict_t) {
    // SAFETY: catch_unwind guards the ABI boundary.
    catch_ffi_panic("hook_client_disconnect_post", (), || {
        // SAFETY: `entity` is a valid edict pointer provided by the engine.
        let index = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, entity) };
        goldsrc::hooks::emit_player_event("client_disconnect", index);
    });
}

/// Hook for ClientCommand - called when a player issues a command.
///
/// # Safety
/// `_entity` must be a valid pointer to an edict_t.
pub unsafe extern "C" fn hook_client_command(entity: *mut goldsrc_sys::edict_t) {
    // SAFETY: catch_unwind guards the ABI boundary; engine calls are safe at this point.
    catch_ffi_panic("hook_client_command", (), || {
        let index = if !entity.is_null() {
            call_engfunc_ret!(engfuncs().pfnIndexOfEdict, entity)
        } else {
            0
        };

        let argc = call_engfunc_ret!(engfuncs().pfnCmd_Argc);
        if argc == 0 {
            return;
        }
        let cmd_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Argv, 0);
        let args_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Args);
        if !cmd_ptr.is_null()
            && let Ok(cmd_str) = unsafe { std::ffi::CStr::from_ptr(cmd_ptr) }.to_str()
        {
            let args_str = if !args_ptr.is_null() {
                unsafe { std::ffi::CStr::from_ptr(args_ptr) }
                    .to_str()
                    .unwrap_or_default()
            } else {
                ""
            };
            let handled = goldsrc::hooks::dispatch_client_command(index, cmd_str, args_str);
            if handled {
                let mg = crate::meta_globals();
                mg.mres = crate::meta_types::MRES_SUPERCEDE;
            }
        }
    });
}

/// Hook for ServerActivate - called when a new map is loaded and activated.
pub unsafe extern "C" fn hook_server_activate(
    _pedict_list: *mut goldsrc_sys::edict_t,
    _edict_count: i32,
    _client_max: i32,
) {
    catch_ffi_panic("hook_server_activate", (), || {
        crate::backend().precache_pending_resources();
        goldsrc::hooks::on_server_activate();
    });
}

/// Hook for ServerDeactivate - called when the current map ends.
pub unsafe extern "C" fn hook_server_deactivate() {
    catch_ffi_panic("hook_server_deactivate", (), || {
        goldsrc::hooks::on_server_deactivate();
    });
}

/// Hook for StartFrame - called every server frame.
pub unsafe extern "C" fn hook_start_frame() {
    // SAFETY: catch_unwind guards the ABI boundary; engine calls are safe at this point.
    catch_ffi_panic("hook_start_frame", (), || {
        goldsrc::hooks::on_server_frame();
    });
}

/// Hook for PlayerPostThink - called after player physics and movement are updated.
pub unsafe extern "C" fn hook_player_post_think(entity: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_player_post_think", (), || {
        let index = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, entity) };
        goldsrc::hooks::emit_player_event("player_post_think", index);
    });
}

/// Hook for ClientKill - called when a player runs the `kill` suicide command.
pub unsafe extern "C" fn hook_client_kill(entity: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_kill", (), || {
        let index = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, entity) };
        goldsrc::hooks::emit_player_event("client_kill", index);
    });
}

/// Hook for Touch - called when two entities collide.
pub unsafe extern "C" fn hook_touch(
    pent_touched: *mut goldsrc_sys::edict_t,
    pent_other: *mut goldsrc_sys::edict_t,
) {
    catch_ffi_panic("hook_touch", (), || {
        let touched_idx = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, pent_touched) };
        let other_idx = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, pent_other) };
        let mut payload = [0u8; 8];
        payload[0..4].copy_from_slice(&touched_idx.to_le_bytes());
        payload[4..8].copy_from_slice(&other_idx.to_le_bytes());
        goldsrc::hooks::emit_event("entity_touch", &payload);
    });
}

/// Hook for Use - called when an entity is activated or triggered.
pub unsafe extern "C" fn hook_use(
    pent_used: *mut goldsrc_sys::edict_t,
    pent_other: *mut goldsrc_sys::edict_t,
) {
    catch_ffi_panic("hook_use", (), || {
        let used_idx = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, pent_used) };
        let other_idx = unsafe { call_engfunc_ret!(engfuncs().pfnIndexOfEdict, pent_other) };
        let mut payload = [0u8; 8];
        payload[0..4].copy_from_slice(&used_idx.to_le_bytes());
        payload[4..8].copy_from_slice(&other_idx.to_le_bytes());
        goldsrc::hooks::emit_event("entity_use", &payload);
    });
}
