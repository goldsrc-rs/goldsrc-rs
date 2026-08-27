//! Declarative CVar abstraction, builder, and runtime bindings.
//!
//! Provides typed access (`i32`, `f32`, `String`), default values, description,
//! and synchronization flags (archive, server, protected).

/// Console variable behavior and persistence flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CvarFlags(pub u32);

impl CvarFlags {
    /// Empty flags.
    pub const NONE: Self = Self(0);
    /// Save to config file (e.g. `archive`).
    pub const ARCHIVE: Self = Self(1 << 0);
    /// Notify clients when the value changes (`notify`).
    pub const NOTIFY: Self = Self(1 << 1);
    /// Sensitive password/key cvar (`server`).
    pub const SERVER: Self = Self(1 << 2);
    /// Read-only variable, cannot be changed by players.
    pub const READ_ONLY: Self = Self(1 << 3);

    /// Combines two flag sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Checks if a flag is contained.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// A handle to a typed console variable with cached name.
#[derive(Debug, Clone)]
pub struct Cvar<T> {
    name: &'static str,
    default_value: T,
    flags: CvarFlags,
    description: &'static str,
}

impl<T> Cvar<T> {
    /// Name of the CVar.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Flags assigned to this CVar.
    pub const fn flags(&self) -> CvarFlags {
        self.flags
    }

    /// Human-readable description.
    pub const fn description(&self) -> &'static str {
        self.description
    }
}

impl Cvar<i32> {
    /// Creates a new integer CVar definition.
    pub const fn new_int(
        name: &'static str,
        default: i32,
        flags: CvarFlags,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            default_value: default,
            flags,
            description,
        }
    }

    /// Reads current integer value from the engine.
    pub fn get(&self) -> i32 {
        crate::engine::api::cvar_get_float(self.name) as i32
    }

    /// Sets the integer value in the engine.
    pub fn set(&self, val: i32) {
        crate::engine::api::cvar_set_float(self.name, val as f32);
    }
}

impl Cvar<f32> {
    /// Creates a new floating-point CVar definition.
    pub const fn new_float(
        name: &'static str,
        default: f32,
        flags: CvarFlags,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            default_value: default,
            flags,
            description,
        }
    }

    /// Reads current float value from the engine.
    pub fn get(&self) -> f32 {
        crate::engine::api::cvar_get_float(self.name)
    }

    /// Sets the float value in the engine.
    pub fn set(&self, val: f32) {
        crate::engine::api::cvar_set_float(self.name, val);
    }
}

impl Cvar<String> {
    /// Creates a new string CVar definition.
    pub fn new_string(
        name: &'static str,
        default: &'static str,
        flags: CvarFlags,
        description: &'static str,
    ) -> Self {
        Self {
            name,
            default_value: default.to_string(),
            flags,
            description,
        }
    }

    /// Reads current string value from the engine.
    pub fn get(&self) -> String {
        crate::engine::api::cvar_get_string(self.name).unwrap_or_else(|| self.default_value.clone())
    }

    /// Sets the string value in the engine.
    pub fn set(&self, val: &str) {
        crate::engine::api::cvar_set_string(self.name, val);
    }
}
