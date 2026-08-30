//! Engine function table bridge implementing `goldsrc_api::Engine`.

use crate::backend::print_queue::{PrintQueue, escape_server_print, sanitize_client_print};
use crate::{call_engfunc, call_engfunc_ret};
use goldsrc_api::EngineCvars;
use goldsrc_sys::enginefuncs_t;

/// Standard `Engine` implementation parameterized by the engfunc source.
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

    /// Creates a player handle from an index if valid.
    ///
    /// For player slots (1..=32) the edict must have `FL_CLIENT` (1 << 3) set;
    /// engine slots without a connected client will not have this flag even when
    /// `edict.free == 0`, so we reject them here before handing off to
    /// `EDict::is_valid()` which only checks the serial number.
    pub fn get_player(&self, index: i32) -> Option<goldsrc_api::Player> {
        unsafe {
            let funcs = (self.engfuncs)();
            let edict = (funcs.pfnPEntityOfEntIndex).and_then(|f| f(index).as_mut())?;
            if edict.free != 0 {
                return None;
            }
            if (1..=32).contains(&index) && edict.v.flags & goldsrc_api::consts::FL_CLIENT == 0 {
                return None;
            }
            Some(goldsrc_api::Player::from_raw(index, edict))
        }
    }

    /// Counts connected active human and bot players from engine edicts.
    pub fn count_active_players(&self) -> usize {
        let auth_count = goldsrc_api::auth::Auth::total_players();
        if auth_count > 0 {
            return auth_count;
        }
        let mut count = 0;
        for i in 1..=(goldsrc_api::consts::MAX_PLAYERS as i32) {
            if <Self as goldsrc_api::EngineEntities>::player_name(self, i).is_some() {
                count += 1;
            }
        }
        count
    }

    /// Spawns an entity by classname.
    pub fn spawn_entity(&self, classname: &str) -> Option<goldsrc_api::Entity> {
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
            let index = crate::api_registry::edict_index(edict);
            Some(goldsrc_api::Entity::from_raw(index, edict))
        }
    }

    /// Prints a message to the server console.
    pub fn server_print(&self, message: &str) {
        <Self as goldsrc_api::EngineConsole>::server_print(self, message);
    }

    /// Executes a server command string.
    pub fn server_command(&self, command: &str) {
        <Self as goldsrc_api::EngineConsole>::server_command(self, command);
    }

    /// Drains the deferred server-print queue to the engine console with
    /// fmtlib-safe escaping (ReHLDS routes `ServerPrint` through fmtlib).
    pub fn drain_prints(&self) {
        for message in self.print_queue.drain() {
            if let Ok(cstr) = std::ffi::CString::new(message) {
                unsafe {
                    call_engfunc!((self.engfuncs)().pfnServerPrint, cstr.as_ptr());
                }
            }
        }
    }
}

static PRECACHE_SOUNDS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));
static PRECACHE_MODELS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));
static PRECACHE_GENERICS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

impl EngineBackend {
    /// Precaches all pending/registered resources during map spawn phase.
    pub fn precache_pending_resources(&self) {
        let default_sounds = [
            "events/tutor_msg.wav",
            "items/suitchargeno1.wav",
            "buttons/bell1.wav",
            "buttons/button1.wav",
            "buttons/blip1.wav",
            "weapons/c4_beep1.wav",
            "common/wpn_denyselect.wav",
        ];
        if let Ok(mut set) = PRECACHE_SOUNDS.lock() {
            for sound in default_sounds {
                set.insert(sound.to_string());
            }
        }

        unsafe {
            if let Ok(sounds) = PRECACHE_SOUNDS.lock() {
                for sound in sounds.iter() {
                    if let Ok(cpath) = std::ffi::CString::new(sound.as_str()) {
                        call_engfunc!((self.engfuncs)().pfnPrecacheSound, cpath.as_ptr());
                    }
                }
            }
            if let Ok(models) = PRECACHE_MODELS.lock() {
                for model in models.iter() {
                    if let Ok(cpath) = std::ffi::CString::new(model.as_str()) {
                        call_engfunc!((self.engfuncs)().pfnPrecacheModel, cpath.as_ptr());
                    }
                }
            }
            if let Ok(generics) = PRECACHE_GENERICS.lock() {
                for generic in generics.iter() {
                    if let Ok(cpath) = std::ffi::CString::new(generic.as_str()) {
                        call_engfunc!((self.engfuncs)().pfnPrecacheGeneric, cpath.as_ptr());
                    }
                }
            }
        }
    }
}

impl goldsrc_api::EnginePrecache for EngineBackend {
    fn precache_model(&self, path: &str) -> i32 {
        if let Ok(mut set) = PRECACHE_MODELS.lock() {
            set.insert(path.to_string());
        }
        unsafe {
            let cpath = std::ffi::CString::new(path).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnPrecacheModel, cpath.as_ptr())
        }
    }

    fn precache_sound(&self, path: &str) -> i32 {
        if let Ok(mut set) = PRECACHE_SOUNDS.lock() {
            set.insert(path.to_string());
        }
        unsafe {
            let cpath = std::ffi::CString::new(path).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnPrecacheSound, cpath.as_ptr())
        }
    }

    fn precache_generic(&self, path: &str) -> i32 {
        if let Ok(mut set) = PRECACHE_GENERICS.lock() {
            set.insert(path.to_string());
        }
        unsafe {
            let cpath = std::ffi::CString::new(path).unwrap_or_default();
            call_engfunc_ret!((self.engfuncs)().pfnPrecacheGeneric, cpath.as_ptr())
        }
    }
}

static USER_MSG_REGISTRY: std::sync::LazyLock<
    std::sync::RwLock<std::collections::HashMap<String, i32>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

pub type UserMsgResolverFn = fn(&str) -> i32;
pub type MapNameResolverFn = fn() -> Option<String>;

static USER_MSG_RESOLVER_FN: std::sync::OnceLock<UserMsgResolverFn> = std::sync::OnceLock::new();
static MAP_NAME_RESOLVER_FN: std::sync::OnceLock<MapNameResolverFn> = std::sync::OnceLock::new();

/// Sets a backend-specific resolver for querying the active map name.
pub fn set_map_name_resolver(resolver: MapNameResolverFn) {
    let _ = MAP_NAME_RESOLVER_FN.set(resolver);
}

pub type GamedllSpawnFn = unsafe extern "C" fn(*mut goldsrc_sys::edict_t) -> i32;
pub type GamedllTouchFn =
    unsafe extern "C" fn(*mut goldsrc_sys::edict_t, *mut goldsrc_sys::edict_t);

static GAME_DLL_SPAWN: std::sync::OnceLock<GamedllSpawnFn> = std::sync::OnceLock::new();
static GAME_DLL_TOUCH: std::sync::OnceLock<GamedllTouchFn> = std::sync::OnceLock::new();

/// Registers the real GameDLL `DispatchSpawn` (call once after DLL load).
pub fn set_game_dll_spawn(f: GamedllSpawnFn) {
    let _ = GAME_DLL_SPAWN.set(f);
}

/// Registers the real GameDLL `Touch` (call once after DLL load).
pub fn set_game_dll_touch(f: GamedllTouchFn) {
    let _ = GAME_DLL_TOUCH.set(f);
}

/// Sets a backend-specific resolver for finding user message IDs.
pub fn set_user_msg_resolver(resolver: UserMsgResolverFn) {
    let _ = USER_MSG_RESOLVER_FN.set(resolver);
}

/// Registers a known user message ID into the runtime registry.
pub fn register_user_msg_id(name: &str, id: i32) {
    if id > 0
        && id != 255
        && let Ok(mut map) = USER_MSG_REGISTRY.write()
    {
        map.insert(name.to_string(), id);
    }
}

impl goldsrc_api::EngineMessages for EngineBackend {
    fn reg_user_msg(&self, name: &str, size: i32) -> i32 {
        if let Ok(map) = USER_MSG_REGISTRY.read()
            && let Some(&id) = map.get(name)
            && id > 0
            && id != 255
        {
            return id;
        }

        if let Some(resolver) = USER_MSG_RESOLVER_FN.get() {
            let id = resolver(name);
            if id > 0 && id != 255 {
                register_user_msg_id(name, id);
                return id;
            }
        }

        let engine_id = unsafe {
            if let Ok(cname) = std::ffi::CString::new(name) {
                call_engfunc_ret!((self.engfuncs)().pfnRegUserMsg, cname.as_ptr(), size)
            } else {
                0
            }
        };

        if engine_id > 0 && engine_id != 255 {
            register_user_msg_id(name, engine_id);
            return engine_id;
        }

        0
    }

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
            let clean = val.replace('\0', "");
            let safe = if clean.len() > 500 {
                let mut end = 500;
                while end > 0 && !clean.is_char_boundary(end) {
                    end -= 1;
                }
                &clean[..end]
            } else {
                &clean
            };
            if let Ok(cstr) = std::ffi::CString::new(safe) {
                call_engfunc!((self.engfuncs)().pfnWriteString, cstr.as_ptr());
            }
        }
    }

    fn write_entity(&self, val: i32) {
        unsafe {
            call_engfunc!((self.engfuncs)().pfnWriteEntity, val);
        }
    }
}

impl goldsrc_api::EngineConsole for EngineBackend {
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

impl goldsrc_api::EngineEntities for EngineBackend {
    fn entity_is_valid(&self, index: i32) -> bool {
        unsafe {
            let funcs = (self.engfuncs)();
            let Some(pedict) = (funcs.pfnPEntityOfEntIndex).and_then(|f| f(index).as_mut()) else {
                return false;
            };
            if pedict.free != 0 {
                return false;
            }
            if (1..=32).contains(&index) {
                // GoldSrc engine: pev->flags & FL_CLIENT (1 << 3 = 8).
                // Edict is a connected client only if FL_CLIENT is set.
                if pedict.v.flags & goldsrc_api::consts::FL_CLIENT == 0 {
                    return false;
                }
                if pedict.v.netname != 0 {
                    return true;
                }
                if let Some(get_infokey) = funcs.pfnGetInfoKeyBuffer
                    && let Some(infokey_val) = funcs.pfnInfoKeyValue
                {
                    let buffer = get_infokey(pedict);
                    let key = std::ffi::CString::new("name").unwrap_or_default();
                    let val_ptr = infokey_val(buffer, key.as_ptr());
                    if !val_ptr.is_null()
                        && let Ok(name_str) = std::ffi::CStr::from_ptr(val_ptr).to_str()
                    {
                        return !name_str.trim().is_empty();
                    }
                }
                return true;
            }
            true
        }
    }

    fn entity_classname(&self, index: i32) -> Option<String> {
        unsafe {
            let funcs = (self.engfuncs)();
            let pedict = (funcs.pfnPEntityOfEntIndex)?(index);
            if pedict.is_null() {
                return None;
            }
            let classname_offset = (*pedict).v.classname;
            if classname_offset != 0
                && let Some(sz_from_idx) = funcs.pfnSzFromIndex
            {
                let str_ptr = sz_from_idx(classname_offset as i32);
                if !str_ptr.is_null()
                    && let Ok(s) = std::ffi::CStr::from_ptr(str_ptr).to_str()
                {
                    return Some(s.to_string());
                }
            }
            None
        }
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

    fn player_handle(&self, index: i32) -> Option<goldsrc_api::Player> {
        self.get_player(index)
    }

    fn player_name(&self, index: i32) -> Option<String> {
        unsafe {
            let funcs = (self.engfuncs)();
            let pedict = (funcs.pfnPEntityOfEntIndex)?(index);
            if pedict.is_null() {
                return None;
            }
            if let Some(get_infokey) = funcs.pfnGetInfoKeyBuffer
                && let Some(infokey_val) = funcs.pfnInfoKeyValue
            {
                let buffer = get_infokey(pedict);
                let key = std::ffi::CString::new("name").unwrap_or_default();
                let val_ptr = infokey_val(buffer, key.as_ptr());
                if !val_ptr.is_null()
                    && let Ok(name_str) = std::ffi::CStr::from_ptr(val_ptr).to_str()
                    && !name_str.is_empty()
                {
                    return Some(name_str.to_string());
                }
            }
            let netname_offset = (*pedict).v.netname;
            if netname_offset != 0
                && let Some(sz_from_idx) = funcs.pfnSzFromIndex
            {
                let str_ptr = sz_from_idx(netname_offset as i32);
                if !str_ptr.is_null()
                    && let Ok(name_str) = std::ffi::CStr::from_ptr(str_ptr).to_str()
                    && !name_str.is_empty()
                {
                    return Some(name_str.to_string());
                }
            }
            None
        }
    }

    fn player_lang(&self, index: i32) -> Option<String> {
        unsafe {
            let funcs = (self.engfuncs)();
            let pedict = (funcs.pfnPEntityOfEntIndex)?(index);
            if pedict.is_null() {
                return None;
            }
            if let Some(get_infokey) = funcs.pfnGetInfoKeyBuffer
                && let Some(infokey_val) = funcs.pfnInfoKeyValue
            {
                let buffer = get_infokey(pedict);
                for key_name in ["_lang", "_cl_lang", "lang", "cl_lang"] {
                    let key = std::ffi::CString::new(key_name).unwrap_or_default();
                    let val_ptr = infokey_val(buffer, key.as_ptr());
                    if !val_ptr.is_null()
                        && let Ok(lang_str) = std::ffi::CStr::from_ptr(val_ptr).to_str()
                        && !lang_str.trim().is_empty()
                    {
                        return Some(lang_str.trim().to_lowercase());
                    }
                }
            }
            self.cvar_get_string("server_language")
        }
    }

    fn player_team(&self, index: i32) -> i32 {
        unsafe {
            let funcs = (self.engfuncs)();
            let pedict = match funcs.pfnPEntityOfEntIndex {
                Some(f) => f(index),
                None => return 0,
            };
            if pedict.is_null() {
                return 0;
            }

            // 1. Direct edict team offset (CS 1.6 / CZ team slot: 1=T, 2=CT, 3=Spectator)
            let edict_team = (*pedict).v.team;
            if (1..=3).contains(&edict_team) {
                return edict_team;
            }

            // 2. Query infobuffer for "model" (e.g. "terror", "leet", "urban", "gign", "gsg9", "sas")
            if let Some(get_infokey) = funcs.pfnGetInfoKeyBuffer
                && let Some(infokey_val) = funcs.pfnInfoKeyValue
            {
                let buffer = get_infokey(pedict);
                let model_key = std::ffi::CString::new("model").unwrap_or_default();
                let val_ptr = infokey_val(buffer, model_key.as_ptr());
                if !val_ptr.is_null()
                    && let Ok(model_str) = std::ffi::CStr::from_ptr(val_ptr).to_str()
                {
                    let m = model_str.to_ascii_lowercase();
                    if m == "terror" || m == "leet" || m == "arctic" || m == "guerilla" {
                        return 1; // Terrorist
                    } else if m == "urban" || m == "gsg9" || m == "gign" || m == "sas" || m == "vip"
                    {
                        return 2; // Counter-Terrorist
                    }
                }
            }

            0 // Unassigned
        }
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
            let cstr = std::ffi::CString::new(classname).ok()?;
            let str_id = funcs.pfnAllocString.map(|f| f(cstr.as_ptr())).unwrap_or(0);
            if str_id == 0 {
                return None;
            }
            let pent = funcs.pfnCreateNamedEntity.map(|f| f(str_id))?;
            if pent.is_null() {
                return None;
            }
            (*pent).v.spawnflags |= 1 << 30;

            let idx = crate::api_registry::edict_index(pent);
            if idx > 0 { Some(idx) } else { None }
        }
    }

    fn remove_entity(&self, index: i32) {
        unsafe {
            let funcs = (self.engfuncs)();
            let pent = funcs.pfnPEntityOfEntIndex.and_then(|f| {
                let p = f(index);
                if p.is_null() { None } else { Some(p) }
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
                if p.is_null() { None } else { Some(p) }
            });
            if let Some(p) = pent {
                call_engfunc_ret!(funcs.pfnDropToFloor, p)
            } else {
                0
            }
        }
    }

    fn dispatch_spawn(&self, index: i32) -> i32 {
        unsafe {
            let funcs = (self.engfuncs)();
            let pent = funcs.pfnPEntityOfEntIndex.and_then(|f| {
                let p = f(index);
                if p.is_null() { None } else { Some(p) }
            });
            match (pent, GAME_DLL_SPAWN.get()) {
                (Some(p), Some(f)) => f(p),
                _ => {
                    log::debug!(target: "core", "dispatch_spawn({index}): no GameDLL bridge");
                    0
                }
            }
        }
    }

    fn dispatch_touch(&self, touched: i32, other: i32) {
        unsafe {
            let funcs = (self.engfuncs)();
            let resolve = |idx: i32| {
                funcs.pfnPEntityOfEntIndex.and_then(|f| {
                    let p = f(idx);
                    if p.is_null() { None } else { Some(p) }
                })
            };
            match (resolve(touched), resolve(other), GAME_DLL_TOUCH.get()) {
                (Some(a), Some(b), Some(f)) => f(a, b),
                _ => {
                    log::debug!(target: "core", "dispatch_touch({touched},{other}): no GameDLL bridge");
                }
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
            let funcs = (self.engfuncs)();
            if let Some(pfn) = funcs.pfnCVarGetString {
                let ptr = pfn(cname.as_ptr());
                if !ptr.is_null() {
                    let val = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
                    if !val.is_empty() {
                        return Some(val);
                    }
                }
            }
            if let Some(pfn_ptr) = funcs.pfnCVarGetPointer {
                let cvar_ptr = pfn_ptr(cname.as_ptr());
                if !cvar_ptr.is_null() && !(*cvar_ptr).string.is_null() {
                    let val = std::ffi::CStr::from_ptr((*cvar_ptr).string)
                        .to_string_lossy()
                        .into_owned();
                    if !val.is_empty() {
                        return Some(val);
                    }
                }
            }
            if name == "mapname"
                && let Some(resolver) = MAP_NAME_RESOLVER_FN.get()
                && let Some(m) = resolver()
                && !m.is_empty()
            {
                return Some(m);
            }
            None
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
                crate::api_registry::edict_index(raw_trace.pHit)
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
                crate::api_registry::edict_index(raw_trace.pHit)
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
