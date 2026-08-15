//! Engine hook callbacks for the Metamod backend.

use goldsrc_sys::ffi::catch_ffi_panic;

use crate::{call_engfunc, call_engfunc_ret, engfuncs, PRINT_QUEUE};

/// Hook for DispatchSpawn - called when an entity spawns.
///
/// # Safety
/// `edict` must be a valid pointer to an edict_t.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_spawn(_edict: *mut goldsrc_sys::edict_t) -> i32 {
    // SAFETY: trivial hook; no Rust state touched. catch_unwind guards the ABI boundary.
    catch_ffi_panic("hook_spawn", 0, || 0)
}

/// Post-hook for DispatchSpawn.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_spawn_post(_edict: *mut goldsrc_sys::edict_t) -> i32 {
    catch_ffi_panic("hook_spawn_post", 0, || 0)
}

/// Post-hook for StartFrame.
pub unsafe extern "C" fn hook_start_frame_post() {
    // =========================================================================================
    // WARNING! VERY IMPORTANT COSTELL AND HISTORICAL REFERENCE FOR DESCENDANTS
    // =========================================================================================
    // If you're reading this code, you're probably wondering: "What the fuck is going on here?"
    //
    // Some genius (may their code compile with errors!) decided to update the console logger
    // in ReHLDS / HLDS to use the C++ `fmtlib` library (`std::format`),
    // BUT FORGOT THAT STRINGS FROM PLUGINS CANNOT BE PASSED DIRECTLY AS A FORMAT!
    //
    // The result is that if the plugin outputs JSON or any string with curly braces `{` or `}`,
    // `fmtlib` considers them format specifiers, throws an UNCAUGHT exception
    // `std::format_error("invalid format string")` and THE ENTIRE SERVER CRASHES TO HELL!
    //
    // I spent half me life blaming StartFrame, mutexes, poor Wasmi, memory alignment and Windows,
    // but the fault lay with a damn genius who didn't know how to use `fmt::print("{}")`!
    //
    // Hence:
    // 1. PRINT_QUEUE saves from CRT buffer overflow when spamming logs for 1 frame.
    // 2. All `{` and `}` are ESCAPED as `{{` and `}}` — so `fmt` turns them back into braces without a crash.
    // 3. `%` is escaped as `%%`.
    // =========================================================================================

    for message in PRINT_QUEUE.drain() {
        if let Ok(msg) = std::ffi::CString::new(message) {
            call_engfunc!(engfuncs().pfnServerPrint, msg.as_ptr());
        }
    }
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

/// Post-hook for ClientConnect.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_client_connect_post(
    _entity: *mut goldsrc_sys::edict_t,
    _name: *const std::os::raw::c_char,
    _address: *const std::os::raw::c_char,
    _reject_reason: *mut std::os::raw::c_char,
) -> goldsrc_sys::qboolean {
    catch_ffi_panic("hook_client_connect_post", 0, || 0)
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
pub unsafe extern "C" fn hook_client_disconnect_post(_entity: *mut goldsrc_sys::edict_t) {
    catch_ffi_panic("hook_client_disconnect_post", (), || ());
}

/// Hook for ClientCommand - called when a player issues a command.
///
/// # Safety
/// `_entity` must be a valid pointer to an edict_t.
#[allow(dead_code)]
pub unsafe extern "C" fn hook_client_command(_entity: *mut goldsrc_sys::edict_t) {
    // SAFETY: catch_unwind guards the ABI boundary; engine calls are safe at this point.
    catch_ffi_panic("hook_client_command", (), || {
        let argc = call_engfunc_ret!(engfuncs().pfnCmd_Argc);
        if argc == 0 {
            return;
        }
        let cmd_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Argv, 0);
        let args_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Args);
        if !cmd_ptr.is_null() {
            if let Ok(cmd_str) = std::ffi::CStr::from_ptr(cmd_ptr).to_str() {
                let args_str = if !args_ptr.is_null() {
                    std::ffi::CStr::from_ptr(args_ptr)
                        .to_str()
                        .unwrap_or_default()
                } else {
                    ""
                };
                goldsrc::host::HostRuntime::with_manager(|m| {
                    if let Some(manager) = m {
                        manager.dispatch_command(cmd_str, args_str);
                    }
                });
            }
        }
    });
}

/// Hook for StartFrame - called every server frame.
pub unsafe extern "C" fn hook_start_frame() {
    // SAFETY: catch_unwind guards the ABI boundary; engine calls are safe at this point.
    catch_ffi_panic("hook_start_frame", (), || {
        goldsrc::host::HostRuntime::with_manager(|m| {
            if let Some(manager) = m {
                manager.on_server_frame();
            }
        });
    });
}
