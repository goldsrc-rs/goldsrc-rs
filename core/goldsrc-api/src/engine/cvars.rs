//! Engine console variables (cvar) operations.

/// Console variable operations.
pub trait EngineCvars: Send + Sync {
    /// Read a cvar value as a floating-point number.
    fn cvar_get_float(&self, name: &str) -> f32;

    /// Set a cvar value as a floating-point number.
    fn cvar_set_float(&self, name: &str, val: f32);

    /// Read a cvar value as a string.
    fn cvar_get_string(&self, name: &str) -> Option<String>;

    /// Set a cvar value as a string.
    fn cvar_set_string(&self, name: &str, val: &str);
}
