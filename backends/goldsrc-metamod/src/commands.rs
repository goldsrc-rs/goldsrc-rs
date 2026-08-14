//! Server console command handling for `meta-rs` / `mrs`.

use goldsrc_api::Engine;

use crate::{backend, call_engfunc, call_engfunc_ret, engfuncs, wasm_manager};

/// Server command handler for `meta-rs` and `mrs` console commands.
pub unsafe extern "C" fn handle_mrs_command() {
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let argc = call_engfunc_ret!(engfuncs().pfnCmd_Argc);
        if argc == 0 {
            return;
        }

        let mut raw_args = Vec::new();
        for i in 0..argc {
            let arg_ptr = call_engfunc_ret!(engfuncs().pfnCmd_Argv, i);
            if !arg_ptr.is_null() {
                if let Ok(cstr) = std::ffi::CStr::from_ptr(arg_ptr).to_str() {
                    raw_args.push(std::ffi::OsString::from(cstr));
                }
            }
        }

        goldsrc::cli::dispatch_mrs_command(
            raw_args,
            wasm_manager(),
            (
                env!("CARGO_PKG_VERSION"),
                env!("GIT_HASH"),
                env!("BUILD_TARGET"),
            ),
            |msg| backend().server_print(msg),
        );
    }));
    if let Err(err) = res {
        let err_msg = if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        backend().server_print(&format!(
            "[GoldSrc.rs PANIC] Caught panic in CLI Command: {}\n",
            err_msg
        ));
    }
}

pub fn register_cli_commands() {
    let cmd_meta_rs = std::ffi::CString::new("meta-rs").unwrap();
    let cmd_mrs = std::ffi::CString::new("mrs").unwrap();
    unsafe {
        call_engfunc!(
            engfuncs().pfnAddServerCommand,
            cmd_meta_rs.as_ptr(),
            Some(handle_mrs_command)
        );
        call_engfunc!(
            engfuncs().pfnAddServerCommand,
            cmd_mrs.as_ptr(),
            Some(handle_mrs_command)
        );
    }
}
