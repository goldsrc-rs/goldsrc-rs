//! Console variable flags and provider interface.

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

/// Abstract cvar operations required for engine synchronization.
pub trait CvarEngine: Send + Sync {
    /// Read a cvar value as a floating-point number.
    fn cvar_get_float(&self, name: &str) -> f32;

    /// Set a cvar value as a floating-point number.
    fn cvar_set_float(&self, name: &str, val: f32);

    /// Read a cvar value as a string.
    fn cvar_get_string(&self, name: &str) -> Option<String>;

    /// Set a cvar value as a string.
    fn cvar_set_string(&self, name: &str, val: &str);

    /// Registers an engine console variable with the given name, default string value, and behavior flags.
    fn cvar_register(&self, name: &str, default_value: &str, flags: CvarFlags) -> bool;
}
