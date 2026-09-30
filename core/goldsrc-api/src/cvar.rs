//! Declarative CVar abstraction, builder, runtime bindings, and configuration models.
//!
//! Provides typed access (`i32`, `f32`, `String`), default values, description,
//! synchronization flags (archive, server, protected), and the [`ConfigModel`]
//! trait for bidirectional synchronization between engine `cvar_t` and disk TOML.

pub use goldsrc_spi::cvar::{CvarEngine, CvarFlags};
use std::sync::{Arc, Mutex};

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
    fn register_cvars(&self, engine: &dyn CvarEngine);

    /// Synchronizes local fields from the current engine CVAR values.
    fn sync_from_cvars(&mut self, engine: &dyn CvarEngine);

    /// Writes local field values into the engine's CVARs.
    fn sync_to_cvars(&self, engine: &dyn CvarEngine);
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

#[cfg(target_arch = "wasm32")]
use crate::bindings::goldsrc::engine::api as host_api;

/// Read float console variable via host WASM or fallback mock.
pub fn cvar_get_float(name: &str) -> f32 {
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_cvar_get_float(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
        800.0
    }
}

/// Set float console variable via host WASM or fallback mock.
pub fn cvar_set_float(name: &str, val: f32) {
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_cvar_set_float(name, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, val);
    }
}

/// Read string console variable via host WASM or fallback mock.
pub fn cvar_get_string(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_cvar_get_string(name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
        None
    }
}

/// Set string console variable via host WASM or fallback mock.
pub fn cvar_set_string(name: &str, val: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        host_api::host_cvar_set_string(name, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (name, val);
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
        cvar_get_float(self.name) as i32
    }

    /// Sets the integer value in the engine.
    pub fn set(&self, val: i32) {
        let old = self.get();
        cvar_set_float(self.name, val as f32);
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
        cvar_get_float(self.name)
    }

    /// Sets the float value in the engine.
    pub fn set(&self, val: f32) {
        let old = self.get();
        cvar_set_float(self.name, val);
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
        cvar_get_string(self.name).unwrap_or_else(|| self.default_value.clone())
    }

    /// Sets the string value in the engine.
    pub fn set(&self, val: &str) {
        let old = self.get();
        cvar_set_string(self.name, val);
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
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self);

    /// Writes `self` into the engine cvar.
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str);
}

impl FromCvarEngine for i32 {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as i32;
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for u32 {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as u32;
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for usize {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as usize;
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for f32 {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name);
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
        engine.cvar_set_float(name, *self);
    }
}

impl FromCvarEngine for f64 {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) as f64;
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
        engine.cvar_set_float(name, *self as f32);
    }
}

impl FromCvarEngine for bool {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        *current = engine.cvar_get_float(name) > 0.0;
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
        engine.cvar_set_float(name, if *self { 1.0 } else { 0.0 });
    }
}

impl FromCvarEngine for String {
    fn read_cvar(engine: &dyn CvarEngine, name: &str, current: &mut Self) {
        if let Some(val) = engine.cvar_get_string(name) {
            *current = val;
        }
    }
    fn write_cvar(&self, engine: &dyn CvarEngine, name: &str) {
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
