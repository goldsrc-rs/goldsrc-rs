//! Server console command handling for `meta-rs` / `mrs`.

use goldsrc_api::Engine;

use crate::{backend, call_engfunc, call_engfunc_ret, engfuncs};

pub fn register_cli_commands() {
    goldsrc::cli::init_host_cli(goldsrc::cli::HostCliBackend {
        argc: || unsafe { call_engfunc_ret!(engfuncs().pfnCmd_Argc) },
        argv: |i| unsafe { call_engfunc_ret!(engfuncs().pfnCmd_Argv, i) },
        print: |msg| backend().server_print(msg),
        version: (
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            env!("BUILD_TARGET"),
        ),
    });
    goldsrc::cli::register_host_commands(|name, handler| {
        let cname = std::ffi::CString::new(name).unwrap();
        unsafe {
            call_engfunc!(
                engfuncs().pfnAddServerCommand,
                cname.as_ptr(),
                Some(handler)
            );
        }
    });
}
