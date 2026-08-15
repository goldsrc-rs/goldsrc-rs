//! Shared backend plumbing: engine access, deferred print queue and
//! engfunc-call macros. Both Metamod and Standalone backends are thin
//! adapters over this module (enabled by the `host` feature).

use goldsrc_api::Engine;
use goldsrc_sys::enginefuncs_t;

/// Invokes an optional engfunc pointer with no arguments.
#[macro_export]
macro_rules! call_engfunc {
    ($func:expr) => {
        if let Some(f) = $func {
            f();
        }
    };
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*);
        }
    };
}

/// Invokes an optional engfunc pointer and returns its result, or
/// `Default::default()` if the pointer is unset.
#[macro_export]
macro_rules! call_engfunc_ret {
    ($func:expr) => {
        if let Some(f) = $func {
            f()
        } else {
            Default::default()
        }
    };
    ($func:expr, $($arg:expr),*) => {
        if let Some(f) = $func {
            f($($arg),*)
        } else {
            Default::default()
        }
    };
}

/// Deferred server-print queue.
///
/// Printing is deferred to the post-start-frame hook because the engine is
/// unstable if a plugin prints mid-frame. Also escapes fmtlib-sensitive
/// characters (`%`, `{`, `}`) — ReHLDS routes `ServerPrint` through fmtlib
/// and unescaped braces would crash the server.
pub struct PrintQueue(std::sync::Mutex<std::collections::VecDeque<String>>);

impl Default for PrintQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PrintQueue {
    /// Create an empty print queue.
    pub const fn new() -> Self {
        Self(std::sync::Mutex::new(std::collections::VecDeque::new()))
    }
    /// Queue a message for later printing.
    pub fn push(&self, message: &str) {
        if let Ok(mut queue) = self.0.lock() {
            queue.push_back(message.to_string());
        }
    }

    /// Take all pending messages, escaping fmtlib-sensitive characters.
    ///
    /// `%` → `%%`, `{`/`}` → `{{`/`}}`, CR/LF stripped, lines trimmed to 400 chars.
    pub fn drain(&self) -> Vec<String> {
        let messages = {
            let mut queue = match self.0.lock() {
                Ok(q) => q,
                Err(e) => e.into_inner(),
            };
            if queue.is_empty() {
                return Vec::new();
            }
            std::mem::take(&mut *queue)
        };
        messages
            .into_iter()
            .map(|message| {
                let safe = message
                    .replace('%', "%%")
                    .replace('{', "{{")
                    .replace('}', "}}")
                    .replace('\r', "")
                    .replace('\n', " ");
                let mut end = safe.len().min(400);
                while end > 0 && !safe.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}\n", safe[..end].trim_end())
            })
            .collect()
    }
}

/// Standard `Engine` implementation parameterized by the engfunc source.
///
/// Both backends differ only in how they obtain the engine function table,
/// so the whole `Engine` trait implementation lives here once.
pub struct EngineBackend {
    engfuncs: fn() -> &'static enginefuncs_t,
    print_queue: &'static PrintQueue,
}

impl EngineBackend {
    /// Create a backend from an engfunc-table accessor and a print queue.
    pub const fn new(
        engfuncs: fn() -> &'static enginefuncs_t,
        print_queue: &'static PrintQueue,
    ) -> Self {
        Self {
            engfuncs,
            print_queue,
        }
    }
}

impl Engine for EngineBackend {
    fn spawn_entity(&self, classname: &str) -> Option<goldsrc_api::Entity> {
        unsafe {
            let funcs = (self.engfuncs)();
            let edict = (funcs.pfnCreateEntity)?();
            if edict.is_null() {
                return None;
            }
            let cname = std::ffi::CString::new(classname).unwrap_or_default();
            call_engfunc!(funcs.pfnSetModel, edict, cname.as_ptr());
            let index = (funcs.pfnIndexOfEdict)?(edict);
            Some(goldsrc_api::Entity::from_raw(index, edict))
        }
    }

    fn get_player(&self, index: i32) -> Option<goldsrc_api::Player> {
        unsafe {
            let funcs = (self.engfuncs)();
            let edict = (funcs.pfnPEntityOfEntIndex)?(index);
            if edict.is_null() {
                return None;
            }
            Some(goldsrc_api::Player::from_raw(index, edict))
        }
    }

    fn server_print(&self, message: &str) {
        self.print_queue.push(message);
    }

    fn server_command(&self, command: &str) {
        unsafe {
            let cmd = std::ffi::CString::new(command).unwrap_or_default();
            call_engfunc!((self.engfuncs)().pfnServerCommand, cmd.as_ptr());
        }
    }

    fn cvar_get_float(&self, name: &str) -> f32 {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnCVarGetFloat, cname.as_ptr())
        }
    }

    fn cvar_set_float(&self, name: &str, value: f32) {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            call_engfunc!((self.engfuncs)().pfnCVarSetFloat, cname.as_ptr(), value);
        }
    }
}
