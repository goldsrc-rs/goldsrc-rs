//! Server console command handling for `meta-rs` / `mrs`.

use goldsrc_api::Engine;

use crate::{backend, call_engfunc, call_engfunc_ret, engine_api, wasm_manager};

pub fn register_cli_commands() {
    goldsrc::cli::init_host_cli(goldsrc::cli::HostCliBackend {
        argc: || unsafe { call_engfunc_ret!(engine_api::engfuncs().pfnCmd_Argc) },
        argv: |i| unsafe { call_engfunc_ret!(engine_api::engfuncs().pfnCmd_Argv, i) },
        manager: wasm_manager,
        print: |msg| backend().server_print(msg),
        version: (
            env!("CARGO_PKG_VERSION"),
            "dev",
            "standalone",
        ),
    });
    goldsrc::cli::register_host_commands(|name, handler| {
        let cname = std::ffi::CString::new(name).unwrap();
        unsafe {
            call_engfunc!(
                engine_api::engfuncs().pfnAddServerCommand,
                cname.as_ptr(),
                Some(handler)
            );
        }
    });
}