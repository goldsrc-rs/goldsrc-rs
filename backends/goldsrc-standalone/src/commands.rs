//! Server console command handling for `meta-rs` / `mrs`.

use goldsrc_api::Engine;

use crate::{backend, call_engfunc, call_engfunc_ret, engine_api};

pub fn register_cli_commands() {
    goldsrc::cli::init_host_cli(goldsrc::cli::HostCliBackend {
        argc: || unsafe { call_engfunc_ret!(engine_api::engfuncs().pfnCmd_Argc) },
        argv: |i| unsafe { call_engfunc_ret!(engine_api::engfuncs().pfnCmd_Argv, i) },
        print: |msg| backend().server_print(msg),
        version: (
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            env!("BUILD_TARGET"),
        ),
    });
    goldsrc::cli::register_host_commands_with_names(&["goldsrc-rs", "grs"], |name, handler| {
        let cname = std::ffi::CString::new(name).unwrap().into_raw();
        unsafe {
            call_engfunc!(
                engine_api::engfuncs().pfnAddServerCommand,
                cname,
                Some(handler)
            );
        }
    });
}
