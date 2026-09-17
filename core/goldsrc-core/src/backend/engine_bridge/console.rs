//! Engine console print and command execution operations.

use super::EngineBackend;
use crate::backend::print_queue::{escape_server_print, sanitize_client_print};
use crate::call_engfunc;
use goldsrc_spi::engine::EngineConsole;

impl EngineConsole for EngineBackend {
    fn server_print(&self, message: &str) {
        unsafe {
            let funcs = (self.engfuncs)();
            if let Some(f) = funcs.pfnServerPrint {
                for buffered in self.print_queue.drain() {
                    for line in buffered.lines() {
                        let safe = escape_server_print(line);
                        if let Ok(cstr) = std::ffi::CString::new(safe) {
                            f(cstr.as_ptr());
                        }
                    }
                }
                for line in message.lines() {
                    let safe = escape_server_print(line);
                    if let Ok(cstr) = std::ffi::CString::new(safe) {
                        f(cstr.as_ptr());
                    }
                }
            } else {
                self.print_queue.push(message);
            }
        }
    }

    fn client_print(&self, client_index: i32, print_type: i32, message: &str) {
        unsafe {
            let funcs = (self.engfuncs)();
            if let Some(pfn_p_entity_of_ent_index) = funcs.pfnPEntityOfEntIndex {
                let pedict = pfn_p_entity_of_ent_index(client_index);
                if !pedict.is_null() {
                    let safe_bytes = sanitize_client_print(message);
                    call_engfunc!(
                        funcs.pfnClientPrintf,
                        pedict,
                        print_type as _,
                        safe_bytes.as_ptr() as *const std::ffi::c_char
                    );
                }
            }
        }
    }

    fn server_command(&self, command: &str) {
        unsafe {
            let cmd = std::ffi::CString::new(command).unwrap_or_default();
            call_engfunc!((self.engfuncs)().pfnServerCommand, cmd.as_ptr());
        }
    }
}
