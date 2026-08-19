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
        let mut queue = match self.0.lock() {
            Ok(q) => q,
            Err(e) => e.into_inner(),
        };
        queue.push_back(message.to_string());
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
#[derive(Clone, Copy)]
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
            if let Some(alloc_string) = funcs.pfnAllocString {
                (*edict).v.classname = alloc_string(cname.as_ptr()) as u32;
            }
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

impl goldsrc_api::EnginePrecache for EngineBackend {
    fn precache_model(&self, path: &str) -> i32 {
        unsafe {
            let cpath = std::ffi::CString::new(path).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnPrecacheModel, cpath.as_ptr())
        }
    }

    fn precache_sound(&self, path: &str) -> i32 {
        unsafe {
            let cpath = std::ffi::CString::new(path).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnPrecacheSound, cpath.as_ptr())
        }
    }

    fn precache_generic(&self, path: &str) -> i32 {
        unsafe {
            let cpath = std::ffi::CString::new(path).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnPrecacheGeneric, cpath.as_ptr())
        }
    }
}

impl goldsrc_api::EngineMessages for EngineBackend {
    fn message_begin(
        &self,
        msg_dest: i32,
        msg_type: i32,
        origin: Option<[f32; 3]>,
        edict_index: Option<i32>,
    ) {
        unsafe {
            let porigin = match origin {
                Some(ref pos) => pos.as_ptr(),
                None => std::ptr::null(),
            };
            let pedict = match edict_index {
                Some(idx) => (self.engfuncs)()
                    .pfnPEntityOfEntIndex
                    .map(|f| f(idx))
                    .unwrap_or(std::ptr::null_mut()),
                None => std::ptr::null_mut(),
            };
            call_engfunc!(
                (self.engfuncs)().pfnMessageBegin,
                msg_dest,
                msg_type,
                porigin,
                pedict
            );
        }
    }

    fn message_end(&self) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnMessageEnd);
        }
    }

    fn write_byte(&self, val: i32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteByte, val);
        }
    }

    fn write_char(&self, val: i32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteChar, val);
        }
    }

    fn write_short(&self, val: i32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteShort, val);
        }
    }

    fn write_long(&self, val: i32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteLong, val);
        }
    }

    fn write_angle(&self, val: f32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteAngle, val);
        }
    }

    fn write_coord(&self, val: f32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteCoord, val);
        }
    }

    fn write_string(&self, val: &str) {
        unsafe {
            let cstr = std::ffi::CString::new(val).unwrap_or_default();
            call_engfunc!((self.engfuncs)().pfnWriteString, cstr.as_ptr());
        }
    }

    fn write_entity(&self, val: i32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteEntity, val);
        }
    }
}

impl goldsrc_api::EngineEntities for EngineBackend {
    fn entity_is_valid(&self, index: i32) -> bool {
        self.get_player(index).is_some_and(|p| p.is_valid())
    }

    fn entity_classname(&self, index: i32) -> Option<String> {
        self.get_player(index).and_then(|e| e.classname())
    }

    fn entity_health(&self, index: i32) -> f32 {
        self.get_player(index).map(|e| e.health()).unwrap_or(0.0)
    }

    fn entity_set_health(&self, index: i32, health: f32) {
        if let Some(mut e) = self.get_player(index) {
            e.set_health(health);
        }
    }

    fn entity_origin(&self, index: i32) -> [f32; 3] {
        self.get_player(index)
            .map(|e| e.origin().into())
            .unwrap_or([0.0; 3])
    }

    fn entity_set_origin(&self, index: i32, pos: [f32; 3]) {
        if let Some(mut e) = self.get_player(index) {
            e.set_origin(pos.into());
        }
    }

    fn entity_velocity(&self, index: i32) -> [f32; 3] {
        self.get_player(index)
            .map(|e| e.velocity().into())
            .unwrap_or([0.0; 3])
    }

    fn entity_set_velocity(&self, index: i32, vel: [f32; 3]) {
        if let Some(mut e) = self.get_player(index) {
            e.set_velocity(vel.into());
        }
    }

    fn entity_angles(&self, index: i32) -> [f32; 3] {
        self.get_player(index)
            .map(|e| e.angles().into())
            .unwrap_or([0.0; 3])
    }

    fn entity_set_angles(&self, index: i32, angles: [f32; 3]) {
        if let Some(mut e) = self.get_player(index) {
            e.set_angles(angles.into());
        }
    }

    fn player_name(&self, index: i32) -> Option<String> {
        self.get_player(index).and_then(|p| p.name())
    }

    fn player_armorvalue(&self, index: i32) -> f32 {
        self.get_player(index)
            .map(|p| p.armorvalue())
            .unwrap_or(0.0)
    }

    fn player_set_armorvalue(&self, index: i32, armor: f32) {
        if let Some(mut p) = self.get_player(index) {
            p.set_armorvalue(armor);
        }
    }

    fn create_named_entity(&self, classname: &str) -> Option<i32> {
        unsafe {
            let funcs = (self.engfuncs)();
            let cstr = std::ffi::CString::new(classname).unwrap_or_default();
            let str_id = funcs.pfnAllocString.map(|f| f(cstr.as_ptr())).unwrap_or(0);
            let pent = funcs.pfnCreateNamedEntity.map(|f| f(str_id))?;
            if pent.is_null() {
                return None;
            }
            funcs.pfnIndexOfEdict.map(|f| f(pent))
        }
    }

    fn remove_entity(&self, index: i32) {
        unsafe {
            let funcs = (self.engfuncs)();
            let pent = funcs.pfnPEntityOfEntIndex.and_then(|f| {
                let p = f(index);
                if p.is_null() {
                    None
                } else {
                    Some(p)
                }
            });
            if let Some(p) = pent {
                call_engfunc!(funcs.pfnRemoveEntity, p);
            }
        }
    }

    fn drop_to_floor(&self, index: i32) -> i32 {
        unsafe {
            let funcs = (self.engfuncs)();
            let pent = funcs.pfnPEntityOfEntIndex.and_then(|f| {
                let p = f(index);
                if p.is_null() {
                    None
                } else {
                    Some(p)
                }
            });
            if let Some(p) = pent {
                call_engfunc_ret!(funcs.pfnDropToFloor, p)
            } else {
                0
            }
        }
    }
}

impl goldsrc_api::EngineCvars for EngineBackend {
    fn cvar_get_float(&self, name: &str) -> f32 {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnCVarGetFloat, cname.as_ptr())
        }
    }

    fn cvar_set_float(&self, name: &str, val: f32) {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            call_engfunc!((self.engfuncs)().pfnCVarSetFloat, cname.as_ptr(), val);
        }
    }

    fn cvar_get_string(&self, name: &str) -> Option<String> {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            let ptr = call_engfunc_ret!((self.engfuncs)().pfnCVarGetString, cname.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    fn cvar_set_string(&self, name: &str, val: &str) {
        unsafe {
            let cname = std::ffi::CString::new(name).unwrap_or_default();
            let cval = std::ffi::CString::new(val).unwrap_or_default();
            call_engfunc!(
                (self.engfuncs)().pfnCVarSetString,
                cname.as_ptr(),
                cval.as_ptr()
            );
        }
    }
}

impl goldsrc_api::EnginePhysics for EngineBackend {
    fn point_contents(&self, point: [f32; 3]) -> i32 {
        unsafe { call_engfunc_ret!((self.engfuncs)().pfnPointContents, point.as_ptr()) }
    }

    fn trace_line(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        flags: i32,
        ignore_ent: i32,
    ) -> goldsrc_api::TraceResult {
        unsafe {
            let funcs = (self.engfuncs)();
            let mut raw_trace = std::mem::zeroed::<goldsrc_sys::TraceResult>();
            let pedict = funcs
                .pfnPEntityOfEntIndex
                .map(|f| f(ignore_ent))
                .unwrap_or(std::ptr::null_mut());

            call_engfunc!(
                funcs.pfnTraceLine,
                start.as_ptr(),
                end.as_ptr(),
                flags,
                pedict,
                &mut raw_trace as *mut _
            );

            let hit_id = if raw_trace.pHit.is_null() {
                -1
            } else {
                funcs
                    .pfnIndexOfEdict
                    .map(|f| f(raw_trace.pHit))
                    .unwrap_or(-1)
            };

            goldsrc_api::TraceResult {
                all_solid: raw_trace.fAllSolid != 0,
                start_solid: raw_trace.fStartSolid != 0,
                in_open: raw_trace.fInOpen != 0,
                in_water: raw_trace.fInWater != 0,
                fraction: raw_trace.flFraction,
                end_pos: raw_trace.vecEndPos,
                plane_normal: raw_trace.vecPlaneNormal,
                hit_entity: hit_id,
            }
        }
    }

    fn trace_hull(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        flags: i32,
        hull_number: i32,
        ignore_ent: i32,
    ) -> goldsrc_api::TraceResult {
        unsafe {
            let funcs = (self.engfuncs)();
            let mut raw_trace = std::mem::zeroed::<goldsrc_sys::TraceResult>();
            let pedict = funcs
                .pfnPEntityOfEntIndex
                .map(|f| f(ignore_ent))
                .unwrap_or(std::ptr::null_mut());

            call_engfunc!(
                funcs.pfnTraceHull,
                start.as_ptr(),
                end.as_ptr(),
                flags,
                hull_number,
                pedict,
                &mut raw_trace as *mut _
            );

            let hit_id = if raw_trace.pHit.is_null() {
                -1
            } else {
                funcs
                    .pfnIndexOfEdict
                    .map(|f| f(raw_trace.pHit))
                    .unwrap_or(-1)
            };

            goldsrc_api::TraceResult {
                all_solid: raw_trace.fAllSolid != 0,
                start_solid: raw_trace.fStartSolid != 0,
                in_open: raw_trace.fInOpen != 0,
                in_water: raw_trace.fInWater != 0,
                fraction: raw_trace.flFraction,
                end_pos: raw_trace.vecEndPos,
                plane_normal: raw_trace.vecPlaneNormal,
                hit_entity: hit_id,
            }
        }
    }
}

impl goldsrc_api::EngineSound for EngineBackend {
    fn emit_sound(
        &self,
        entity: i32,
        channel: i32,
        sample: &str,
        volume: f32,
        attenuation: f32,
        flags: i32,
        pitch: i32,
    ) {
        unsafe {
            let funcs = (self.engfuncs)();
            let pedict = funcs
                .pfnPEntityOfEntIndex
                .map(|f| f(entity))
                .unwrap_or(std::ptr::null_mut());
            let cstr = std::ffi::CString::new(sample).unwrap_or_default();
            call_engfunc!(
                funcs.pfnEmitSound,
                pedict,
                channel,
                cstr.as_ptr(),
                volume,
                attenuation,
                flags,
                pitch
            );
        }
    }

    fn emit_ambient_sound(
        &self,
        entity: i32,
        pos: [f32; 3],
        sample: &str,
        volume: f32,
        attenuation: f32,
        flags: i32,
        pitch: i32,
    ) {
        unsafe {
            let funcs = (self.engfuncs)();
            let pedict = funcs
                .pfnPEntityOfEntIndex
                .map(|f| f(entity))
                .unwrap_or(std::ptr::null_mut());
            let mut pos_copy = pos;
            let cstr = std::ffi::CString::new(sample).unwrap_or_default();
            call_engfunc!(
                funcs.pfnEmitAmbientSound,
                pedict,
                pos_copy.as_mut_ptr(),
                cstr.as_ptr(),
                volume,
                attenuation,
                flags,
                pitch
            );
        }
    }
}
