//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract interface that plugin developers use.
//! It has no dependency on any specific backend (Metamod or Standalone).

/// Engine interface — provides access to engine functions.
pub trait Engine {
    /// Spawn an entity by classname.
    fn spawn_entity(&self, classname: &str) -> Option<Entity>;

    /// Get a player by index (1-based).
    fn get_player(&self, index: i32) -> Option<Player>;

    /// Print a message to the server console.
    fn server_print(&self, message: &str);

    /// Execute a server command.
    fn server_command(&self, command: &str);

    /// Get a cvar value as float.
    fn cvar_get_float(&self, name: &str) -> f32;

    /// Set a cvar value.
    fn cvar_set_float(&self, name: &str, value: f32);
}

#[cfg(target_arch = "wasm32")]
mod host_imports {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn host_entity_is_valid(index: i32) -> i32;
        pub fn host_entity_classname(index: i32, out_ptr: *mut u8, out_len: i32) -> i32;
        pub fn host_entity_origin(index: i32, out_x: *mut f32, out_y: *mut f32, out_z: *mut f32);
        pub fn host_entity_health(index: i32) -> f32;

        pub fn host_player_name(index: i32, out_ptr: *mut u8, out_len: i32) -> i32;
        pub fn host_player_origin(index: i32, out_x: *mut f32, out_y: *mut f32, out_z: *mut f32);
        pub fn host_player_set_origin(index: i32, x: f32, y: f32, z: f32);
        pub fn host_player_velocity(index: i32, out_x: *mut f32, out_y: *mut f32, out_z: *mut f32);
        pub fn host_player_set_velocity(index: i32, x: f32, y: f32, z: f32);
        pub fn host_player_health(index: i32) -> f32;
        pub fn host_player_set_health(index: i32, health: f32);
        pub fn host_player_armorvalue(index: i32) -> f32;
        pub fn host_player_set_armorvalue(index: i32, armor: f32);
    }
}

/// Safe wrapper around `edict_t` (entity dictionary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    edict: *mut goldsrc_sys::edict_t,
}

impl Entity {
    /// Creates an Entity from a raw index and edict_t pointer.
    ///
    /// # Safety
    /// The caller must ensure that `edict` is a valid pointer to an entity in the engine.
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        Self { index, edict }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            edict: std::ptr::null_mut(),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn is_valid(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_entity_is_valid(self.index) != 0 }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            !self.edict.is_null()
        }
    }

    pub fn classname(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut buf = [0u8; 64];
            let len = unsafe {
                host_imports::host_entity_classname(self.index, buf.as_mut_ptr(), buf.len() as i32)
            };
            if len > 0 {
                if let Ok(s) = std::str::from_utf8(&buf[..len as usize]) {
                    return Some(s.to_string());
                }
            }
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            let classname = (*self.edict).v.classname;
            if classname == 0 {
                return None;
            }
            let cstr = std::ffi::CStr::from_ptr(classname as *const i8);
            Some(cstr.to_string_lossy().into_owned())
        }
    }

    pub fn origin(&self) -> [f32; 3] {
        #[cfg(target_arch = "wasm32")]
        {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut z = 0.0;
            unsafe { host_imports::host_entity_origin(self.index, &mut x, &mut y, &mut z) };
            [x, y, z]
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.origin
        }
    }

    pub fn health(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_entity_health(self.index) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.health
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn as_ptr(&self) -> *mut goldsrc_sys::edict_t {
        self.edict
    }
}

/// 3D Vector type for GoldSrc positions and velocities.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<[f32; 3]> for Vector3 {
    fn from(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vector3> for [f32; 3] {
    fn from(v: Vector3) -> Self {
        [v.x, v.y, v.z]
    }
}

/// Safe wrapper around a player entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Player {
    pub index: i32,
    #[cfg(not(target_arch = "wasm32"))]
    edict: *mut goldsrc_sys::edict_t,
}

impl Player {
    /// Creates a Player from a raw index and edict_t pointer.
    ///
    /// # Safety
    /// The caller must ensure that `edict` is a valid pointer to a player entity in the engine.
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        Self { index, edict }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(index: i32) -> Self {
        Self {
            index,
            edict: std::ptr::null_mut(),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn is_valid(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_entity_is_valid(self.index) != 0 }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            !self.edict.is_null()
        }
    }

    pub fn name(&self) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            let mut buf = [0u8; 32];
            let len = unsafe {
                host_imports::host_player_name(self.index, buf.as_mut_ptr(), buf.len() as i32)
            };
            if len > 0 {
                if let Ok(s) = std::str::from_utf8(&buf[..len as usize]) {
                    return Some(s.to_string());
                }
            }
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            let netname = (*self.edict).v.netname;
            if netname == 0 {
                return None;
            }
            let cstr = std::ffi::CStr::from_ptr(netname as *const i8);
            Some(cstr.to_string_lossy().into_owned())
        }
    }

    pub fn origin(&self) -> Vector3 {
        #[cfg(target_arch = "wasm32")]
        {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut z = 0.0;
            unsafe { host_imports::host_player_origin(self.index, &mut x, &mut y, &mut z) };
            Vector3 { x, y, z }
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.origin.into()
        }
    }

    pub fn set_origin(&mut self, pos: Vector3) {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_player_set_origin(self.index, pos.x, pos.y, pos.z) };
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.origin = pos.into();
        }
    }

    pub fn velocity(&self) -> Vector3 {
        #[cfg(target_arch = "wasm32")]
        {
            let mut x = 0.0;
            let mut y = 0.0;
            let mut z = 0.0;
            unsafe { host_imports::host_player_velocity(self.index, &mut x, &mut y, &mut z) };
            Vector3 { x, y, z }
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.velocity.into()
        }
    }

    pub fn set_velocity(&mut self, vel: Vector3) {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_player_set_velocity(self.index, vel.x, vel.y, vel.z) };
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.velocity = vel.into();
        }
    }

    pub fn health(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_player_health(self.index) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.health
        }
    }

    pub fn set_health(&mut self, health: f32) {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_player_set_health(self.index, health) };
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.health = health;
        }
    }

    pub fn armorvalue(&self) -> f32 {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_player_armorvalue(self.index) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.armorvalue
        }
    }

    pub fn set_armorvalue(&mut self, armor: f32) {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_imports::host_player_set_armorvalue(self.index, armor) };
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            (*self.edict).v.armorvalue = armor;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn as_ptr(&self) -> *mut goldsrc_sys::edict_t {
        self.edict
    }
}

impl From<Player> for Entity {
    fn from(player: Player) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Entity {
                index: player.index,
                edict: player.edict,
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Entity {
                index: player.index,
            }
        }
    }
}

/// Plugin trait — implement this for your plugin.
pub trait Plugin: Send + Sync {
    /// Called when the plugin is loaded.
    fn on_load(&mut self) {}

    /// Called when the plugin is unloaded.
    fn on_unload(&self) {}

    /// Called when a player connects.
    fn on_client_connect(&self, _player: &Player) {}

    /// Called when a player disconnects.
    fn on_client_disconnect(&self, _player: &Player) {}

    /// Called when a player spawns.
    fn on_client_spawn(&self, _player: &Player) {}

    /// Called when a player dies.
    fn on_client_killed(&self, _victim: &Player, _killer: &Player) {}

    /// Called every server frame.
    fn on_server_frame(&self) {}
}

// SAFETY: Entity and Player are just wrappers around raw pointers.
// The caller must ensure the pointer is valid when used.
unsafe impl Send for Entity {}
unsafe impl Sync for Entity {}
unsafe impl Send for Player {}
unsafe impl Sync for Player {}
