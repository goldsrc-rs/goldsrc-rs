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

/// Entity handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub index: i32,
    pub edict: *mut goldsrc_sys::edict_t,
}

/// Player handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Player {
    pub index: i32,
    pub edict: *mut goldsrc_sys::edict_t,
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
