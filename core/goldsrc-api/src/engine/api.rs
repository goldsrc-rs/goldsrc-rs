//! Global engine operations for plugins (WASM guest & native host mock).

pub use crate::Vector3;

/// Precache a model file (e.g. "models/player.mdl").
pub fn precache_model(path: &str) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_precache_model(path)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = path;
        1
    }
}

/// Precache a sound file (e.g. "events/tutor_msg.wav").
pub fn precache_sound(path: &str) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_precache_sound(path)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = path;
        1
    }
}

/// Precache a generic resource file.
pub fn precache_generic(path: &str) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_precache_generic(path)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = path;
        1
    }
}

/// Emit dynamic sound attached to an entity.
pub fn emit_sound(
    entity: i32,
    channel: i32,
    sample: &str,
    volume: f32,
    attenuation: f32,
    flags: i32,
    pitch: i32,
) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_emit_sound(
            entity,
            channel,
            sample,
            volume,
            attenuation,
            flags,
            pitch,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (entity, channel, sample, volume, attenuation, flags, pitch);
    }
}

/// Read float console variable.
pub fn cvar_get_float(name: &str) -> f32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_cvar_get_float(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
        800.0
    }
}

/// Set float console variable.
pub fn cvar_set_float(name: &str, val: f32) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_cvar_set_float(name, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, val);
    }
}

/// Read string console variable.
pub fn cvar_get_string(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_cvar_get_string(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
        None
    }
}

/// Set string console variable.
pub fn cvar_set_string(name: &str, val: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_cvar_set_string(name, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, val);
    }
}

/// Spawn a named entity (e.g. "info_target", "armoury_entity").
pub fn create_named_entity(classname: &str) -> Option<i32> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_create_named_entity(classname)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = classname;
        None
    }
}

/// Remove an entity from the world.
pub fn remove_entity(entity: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_remove_entity(entity);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = entity;
    }
}

/// Drop an entity to the floor geometry.
pub fn drop_to_floor(entity: i32) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_drop_to_floor(entity)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = entity;
        1
    }
}
