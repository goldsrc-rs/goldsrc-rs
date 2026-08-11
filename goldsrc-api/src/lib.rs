//! Pure Rust traits (interfaces) for GoldSrc engine interaction.
//!
//! This crate defines the abstract interface that plugin developers use.
//! It has no dependency on any specific backend (Metamod or Standalone).

use std::ffi::CStr;

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

/// Safe wrapper around `edict_t` (entity dictionary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub index: i32,
    edict: *mut goldsrc_sys::edict_t,
}

impl Entity {
    /// Create a new Entity from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `edict_t` pointer.
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        Self { index, edict }
    }

    /// Get the entity index (1-based).
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Check if the entity is valid (not null).
    pub fn is_valid(&self) -> bool {
        !self.edict.is_null()
    }

    /// Get the entity's classname.
    pub fn classname(&self) -> Option<String> {
        // SAFETY: edict is valid, v is always present
        unsafe {
            let classname = (*self.edict).v.classname;
            if classname == 0 {
                return None;
            }
            let cstr = CStr::from_ptr(classname as *const i8);
            Some(cstr.to_string_lossy().into_owned())
        }
    }

    /// Get the entity's origin (position).
    pub fn origin(&self) -> [f32; 3] {
        // SAFETY: edict is valid, v is always present
        unsafe { (*self.edict).v.origin }
    }

    /// Get the entity's health.
    pub fn health(&self) -> f32 {
        // SAFETY: edict is valid, v is always present
        unsafe { (*self.edict).v.health }
    }

    /// Get the raw edict pointer.
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
    edict: *mut goldsrc_sys::edict_t,
}

impl Player {
    /// Create a new Player from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `edict_t` pointer.
    pub unsafe fn from_raw(index: i32, edict: *mut goldsrc_sys::edict_t) -> Self {
        Self { index, edict }
    }

    /// Get the player index (1-based).
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Check if the player is valid (not null).
    pub fn is_valid(&self) -> bool {
        !self.edict.is_null()
    }

    /// Get the player's name.
    pub fn name(&self) -> Option<String> {
        // SAFETY: edict is valid, v is always present
        unsafe {
            let netname = (*self.edict).v.netname;
            if netname == 0 {
                return None;
            }
            let cstr = CStr::from_ptr(netname as *const i8);
            Some(cstr.to_string_lossy().into_owned())
        }
    }

    /// Get the player's origin (position).
    pub fn origin(&self) -> Vector3 {
        // SAFETY: edict is valid, v is always present
        unsafe { (*self.edict).v.origin.into() }
    }

    /// Set the player's origin.
    pub fn set_origin(&mut self, pos: Vector3) {
        unsafe {
            (*self.edict).v.origin = pos.into();
        }
    }

    /// Get the player's velocity.
    pub fn velocity(&self) -> Vector3 {
        unsafe { (*self.edict).v.velocity.into() }
    }

    /// Set the player's velocity.
    pub fn set_velocity(&mut self, vel: Vector3) {
        unsafe {
            (*self.edict).v.velocity = vel.into();
        }
    }

    /// Get the player's health.
    pub fn health(&self) -> f32 {
        // SAFETY: edict is valid, v is always present
        unsafe { (*self.edict).v.health }
    }

    /// Set the player's health.
    pub fn set_health(&mut self, health: f32) {
        unsafe {
            (*self.edict).v.health = health;
        }
    }

    /// Get the player's armor value.
    pub fn armorvalue(&self) -> f32 {
        unsafe { (*self.edict).v.armorvalue }
    }

    /// Set the player's armor value.
    pub fn set_armorvalue(&mut self, armor: f32) {
        unsafe {
            (*self.edict).v.armorvalue = armor;
        }
    }

    /// Check if player is alive (health > 0).
    pub fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    /// Get the raw edict pointer.
    pub fn as_ptr(&self) -> *mut goldsrc_sys::edict_t {
        self.edict
    }
}

impl From<Player> for Entity {
    fn from(player: Player) -> Self {
        Entity {
            index: player.index,
            edict: player.edict,
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
