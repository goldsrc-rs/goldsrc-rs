//! Client classifications, connection lifecycles, life states, and team slots.

/// Where a message printed to a player is rendered.
///
/// Wire values match the engine's `PRINT_TYPE` enum consumed by
/// `pfnClientPrintf` (`print_console = 0`, ...). Note these differ from the
/// AMX Mod X `print_*` numbering — never pass AMXX constants here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum PrintTarget {
    /// Player's game console. No color codes supported in any mod.
    Console = 0,
    /// Center-screen notice. Plain text only.
    Center = 1,
    /// Chat area via the `SayText` user message.
    Chat = 2,
    /// Top-left developer notification area (print_notify = 3).
    Notify = 3,
    /// Chat area with color escapes: `^1` default, `^3` team, `^4` green.
    /// Colors render only in mods whose client parses SayText markup
    /// (CS 1.6 / CZ); elsewhere codes appear as literal text.
    #[default]
    ColoredChat = 4,
}

/// Client classification kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientKind {
    /// Human player client.
    Player,
    /// Fake client / AI Bot (`FL_FAKECLIENT`).
    Bot,
    /// HLTV spectator proxy (`FL_PROXY`).
    HLTV,
}

/// Network connection lifecycle of a client slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Slot is vacant.
    Disconnected,
    /// Client is connecting and negotiating resources.
    Connecting,
    /// Client is fully connected and active in the game world.
    Connected,
    /// Client is disconnecting.
    Disconnecting,
}

/// In-game life state of a player entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeState {
    /// Player is alive and participating in the round.
    Alive,
    /// Player is dead and awaiting respawn.
    Dead,
    /// Player is in free-look or spectator camera mode.
    Spectating,
}

impl LifeState {
    /// List of all possible player life states.
    pub const ALL: &'static [LifeState] =
        &[LifeState::Alive, LifeState::Dead, LifeState::Spectating];

    /// Returns the static string representation of this life state.
    pub const fn as_str(&self) -> &'static str {
        match self {
            LifeState::Alive => "alive",
            LifeState::Dead => "dead",
            LifeState::Spectating => "spectating",
        }
    }
}

impl std::fmt::Display for LifeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Game-agnostic team identifier (transparent wrapper over integer team slot).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct Team(pub i32);

impl Team {
    /// Unassigned / choosing team.
    pub const UNASSIGNED: Team = Team(0);
    /// Spectator team slot (common GoldSrc convention: 3).
    pub const SPECTATOR: Team = Team(3);

    /// Creates a new team identifier from a raw integer ID.
    pub const fn new(id: i32) -> Self {
        Self(id)
    }

    /// Returns the raw integer value of the team.
    pub const fn raw(&self) -> i32 {
        self.0
    }

    /// Returns `true` if this is the unassigned team slot (0).
    pub const fn is_unassigned(&self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if this is the spectator team slot (3).
    pub const fn is_spectator(&self) -> bool {
        self.0 == 3
    }
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Team({})", self.0)
    }
}

impl From<i32> for Team {
    fn from(val: i32) -> Self {
        Team(val)
    }
}

impl From<Team> for i32 {
    fn from(team: Team) -> Self {
        team.0
    }
}

use std::borrow::Cow;

/// Trait for types that can be resolved into a language code identifier (e.g. `"ru"`, `"en"`).
pub trait AsLangCode {
    /// Returns the active language code reference for translation lookups.
    fn as_lang_code(&self) -> Cow<'_, str>;
}

impl AsLangCode for str {
    fn as_lang_code(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl AsLangCode for &str {
    fn as_lang_code(&self) -> Cow<'_, str> {
        Cow::Borrowed(*self)
    }
}

impl AsLangCode for String {
    fn as_lang_code(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_str())
    }
}

impl AsLangCode for &String {
    fn as_lang_code(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.as_str())
    }
}

impl AsLangCode for crate::client::Player {
    fn as_lang_code(&self) -> Cow<'_, str> {
        Cow::Owned(self.lang())
    }
}

impl AsLangCode for &crate::client::Player {
    fn as_lang_code(&self) -> Cow<'_, str> {
        Cow::Owned(self.lang())
    }
}

impl<T: AsLangCode> AsLangCode for crate::client::Alive<T> {
    fn as_lang_code(&self) -> Cow<'_, str> {
        self.0.as_lang_code()
    }
}

impl<T: AsLangCode> AsLangCode for crate::client::Dead<T> {
    fn as_lang_code(&self) -> Cow<'_, str> {
        self.0.as_lang_code()
    }
}

impl AsLangCode for crate::client::Spectator {
    fn as_lang_code(&self) -> Cow<'_, str> {
        self.0.as_lang_code()
    }
}

impl AsLangCode for crate::client::Bot {
    fn as_lang_code(&self) -> Cow<'_, str> {
        self.0.as_lang_code()
    }
}

impl AsLangCode for crate::client::HLTV {
    fn as_lang_code(&self) -> Cow<'_, str> {
        self.0.as_lang_code()
    }
}
