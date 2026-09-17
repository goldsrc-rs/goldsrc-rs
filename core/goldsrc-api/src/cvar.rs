//! Declarative CVar abstraction, builder, runtime bindings, and configuration models.
//!
//! Provides typed access (`i32`, `f32`, `String`), default values, description,
//! synchronization flags (archive, server, protected), and the [`ConfigModel`]
//! trait for bidirectional synchronization between engine `cvar_t` and disk TOML.

use std::sync::{Arc, Mutex};

/// Console variable behavior and persistence flags corresponding to GoldSrc `FCVAR_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CvarFlags(pub u32);

impl CvarFlags {
    /// Empty flags.
    pub const NONE: Self = Self(0);
    /// Save to config file (e.g. `vars.rc` or `archive`).
    pub const ARCHIVE: Self = Self(1 << 0);
    /// Changes the client's info string.
    pub const USERINFO: Self = Self(1 << 1);
    /// Server cvar, notifies players when changed.
    pub const SERVER: Self = Self(1 << 2);
    /// Backward-compatible alias for [`SERVER`](Self::SERVER).
    pub const NOTIFY: Self = Self(1 << 2);
    /// Defined by external DLL plugin (`FCVAR_EXTDLL`).
    pub const EXT_DLL: Self = Self(1 << 3);
    /// Defined by the client DLL (`FCVAR_CLIENTDLL`).
    pub const CLIENT_DLL: Self = Self(1 << 4);
    /// Protected cvar (e.g. password, doesn't broadcast data to clients).
    pub const PROTECTED: Self = Self(1 << 5);
    /// Singleplayer only cvar (cannot be changed by clients in multiplayer).
    pub const SP_ONLY: Self = Self(1 << 6);
    /// Read-only variable, cannot be changed by clients.
    pub const READ_ONLY: Self = Self(1 << 6);
    /// Printable characters only (e.g. player names).
    pub const PRINTABLE_ONLY: Self = Self(1 << 7);
    /// Unlogged server cvar (don't log changes to console/log).
    pub const UNLOGGED: Self = Self(1 << 8);
    /// Strip leading and trailing whitespace from value.
    pub const NO_EXTRA_WHITESPACE: Self = Self(1 << 9);

    /// Combines two flag sets.
    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Checks if a flag is contained.
    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns the raw bitmask value.
    #[inline(always)]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for CvarFlags {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CvarFlags {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for CvarFlags {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CvarFlags {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// Metadata descriptor for a configuration field bound to an engine CVAR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CvarField {
    /// Engine console variable name (e.g. `vip_bonus_hp`).
    pub name: &'static str,
    /// TOML key name in configuration file (e.g. `bonus_hp`).
    pub toml_key: &'static str,
    /// Default string value representation.
    pub default_str: &'static str,
    /// Human-readable documentation comment.
    pub description: &'static str,
    /// Behavior flags.
    pub flags: CvarFlags,
}

/// Domain configuration model capable of bidirectional synchronization with
/// engine CVARs and TOML disk formats.
pub trait ConfigModel: Send + Sync {
    /// Serializes configuration fields into documented TOML with section comments.
    fn to_toml(&self) -> String;

    /// Serializes configuration fields into GoldSrc console variable `.cfg` script.
    fn to_cvars(&self) -> String;

    /// Registers all associated CVARs in the given engine interface.
    fn register_cvars(&self, engine: &dyn crate::engine::Engine);

    /// Synchronizes local fields from the current engine CVAR values.
    fn sync_from_cvars(&mut self, engine: &dyn crate::engine::Engine);

    /// Writes local field values into the engine's CVARs.
    fn sync_to_cvars(&self, engine: &dyn crate::engine::Engine);
}

/// Type alias for an observer callback invoked when a [`Cvar`] value changes.
pub type CvarObserver<T> = Arc<Mutex<Box<dyn Fn(&T, &T) + Send + 'static>>>;

/// A handle to a typed console variable with cached name, default value, and observers.
#[derive(Clone)]
pub struct Cvar<T> {
    name: &'static str,
    default_value: T,
    flags: CvarFlags,
    description: &'static str,
    on_change: Option<CvarObserver<T>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Cvar<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cvar")
            .field("name", &self.name)
            .field("default_value", &self.default_value)
            .field("flags", &self.flags)
            .field("description", &self.description)
            .finish()
    }
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

    /// Attaches a change listener callback called whenever `set` is invoked.
    pub fn on_change<F>(mut self, observer: F) -> Self
    where
        F: Fn(&T, &T) + Send + 'static,
    {
        self.on_change = Some(Arc::new(Mutex::new(Box::new(observer))));
        self
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
            on_change: None,
        }
    }

    /// Reads current integer value from the engine.
    pub fn get(&self) -> i32 {
        crate::engine::api::cvar_get_float(self.name) as i32
    }

    /// Sets the integer value in the engine.
    pub fn set(&self, val: i32) {
        let old = self.get();
        crate::engine::api::cvar_set_float(self.name, val as f32);
        if let Some(obs) = &self.on_change
            && let Ok(cb) = obs.lock()
        {
            cb(&old, &val);
        }
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
            on_change: None,
        }
    }

    /// Reads current float value from the engine.
    pub fn get(&self) -> f32 {
        crate::engine::api::cvar_get_float(self.name)
    }

    /// Sets the float value in the engine.
    pub fn set(&self, val: f32) {
        let old = self.get();
        crate::engine::api::cvar_set_float(self.name, val);
        if let Some(obs) = &self.on_change
            && let Ok(cb) = obs.lock()
        {
            cb(&old, &val);
        }
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
            on_change: None,
        }
    }

    /// Reads current string value from the engine.
    pub fn get(&self) -> String {
        crate::engine::api::cvar_get_string(self.name).unwrap_or_else(|| self.default_value.clone())
    }

    /// Sets the string value in the engine.
    pub fn set(&self, val: &str) {
        let old = self.get();
        crate::engine::api::cvar_set_string(self.name, val);
        if let Some(obs) = &self.on_change
            && let Ok(cb) = obs.lock()
        {
            let new_str = val.to_string();
            cb(&old, &new_str);
        }
    }
}

/// Helper trait for formatting a configuration value into a TOML-compatible literal string.
pub trait ToTomlVal {
    /// Formats `self` as a TOML value literal.
    fn to_toml_val(&self) -> String;
}

impl ToTomlVal for String {
    fn to_toml_val(&self) -> String {
        format!("\"{}\"", self.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

impl ToTomlVal for &str {
    fn to_toml_val(&self) -> String {
        format!("\"{}\"", self.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

impl ToTomlVal for bool {
    fn to_toml_val(&self) -> String {
        self.to_string()
    }
}

impl ToTomlVal for i32 {
    fn to_toml_val(&self) -> String {
        self.to_string()
    }
}

impl ToTomlVal for u32 {
    fn to_toml_val(&self) -> String {
        self.to_string()
    }
}

impl ToTomlVal for usize {
    fn to_toml_val(&self) -> String {
        self.to_string()
    }
}

impl ToTomlVal for f32 {
    fn to_toml_val(&self) -> String {
        format!("{:.2}", self)
    }
}

impl ToTomlVal for f64 {
    fn to_toml_val(&self) -> String {
        format!("{:.2}", self)
    }
}

/// Helper trait for reading and writing typed configuration values to/from the GoldSrc engine.
pub trait FromCvarEngine {
    /// Reads current cvar value from the engine and updates `current`.
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self);

    /// Writes `self` into the engine cvar.
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str);
}

impl FromCvarEngine for i32 {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as i32;
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for u32 {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as u32;
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for usize {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as usize;
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for f32 {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name);
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_float(name, *self);
    }
}

impl FromCvarEngine for f64 {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as f64;
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for bool {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) > 0.0;
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_float(name, if *self { 1.0 } else { 0.0 });
    }
}

impl FromCvarEngine for String {
    fn read_cvar(engine: &dyn crate::engine::Engine, name: &str, current: &mut Self) {
        if let Some(val) = engine.cvar_get_string(name) {
            *current = val;
        }
    }
    fn write_cvar(&self, engine: &dyn crate::engine::Engine, name: &str) {
        engine.cvar_set_string(name, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cvar_flags_bitwise() {
        let f1 = CvarFlags::ARCHIVE;
        let f2 = CvarFlags::SERVER;
        let combined = f1 | f2;

        assert!(combined.contains(CvarFlags::ARCHIVE));
        assert!(combined.contains(CvarFlags::SERVER));
        assert!(!combined.contains(CvarFlags::PROTECTED));
        assert_eq!(combined.bits(), (1 << 0) | (1 << 2));
    }
}
