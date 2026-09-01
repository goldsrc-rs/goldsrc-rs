use crate::{backend, call_engfunc, call_engfunc_ret, engfuncs};

pub fn register_cli_commands() {
    goldsrc_core::cli::init_host_cli(goldsrc_core::cli::HostCliBackend {
        argc: || unsafe { call_engfunc_ret!(engfuncs().pfnCmd_Argc) },
        argv: |i| unsafe { call_engfunc_ret!(engfuncs().pfnCmd_Argv, i) },
        print: |msg| backend().server_print(msg),
        version: (
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            env!("BUILD_TARGET"),
        ),
    });
    goldsrc_core::cli::register_host_commands(|name, handler| {
        if let Ok(cname) = std::ffi::CString::new(name) {
            unsafe {
                call_engfunc!(
                    engfuncs().pfnAddServerCommand,
                    cname.into_raw(),
                    Some(handler)
                );
            }
        }
    });
    goldsrc_core::cli::register_plugin_server_commands(|name, handler| {
        if let Ok(cname) = std::ffi::CString::new(name) {
            unsafe {
                call_engfunc!(
                    engfuncs().pfnAddServerCommand,
                    cname.into_raw(),
                    Some(handler)
                );
            }
        }
    });
}
