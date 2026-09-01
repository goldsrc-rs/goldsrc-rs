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

/// Game team identifiers (compatible with Counter-Strike 1.6 team slots).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Team {
    /// Unassigned / choosing team.
    Unassigned = 0,
    /// Terrorist team (T).
    Terrorist = 1,
    /// Counter-Terrorist team (CT).
    CounterTerrorist = 2,
    /// Spectator team (SPEC).
    Spectator = 3,
}

impl Team {
    /// List of all possible teams.
    pub const ALL: &'static [Team] = &[
        Team::Unassigned,
        Team::Terrorist,
        Team::CounterTerrorist,
        Team::Spectator,
    ];

    /// Returns the canonical lowercase string identifier of this team.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Team::Unassigned => "unassigned",
            Team::Terrorist => "terrorist",
            Team::CounterTerrorist => "ct",
            Team::Spectator => "spectator",
        }
    }
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<i32> for Team {
    fn from(val: i32) -> Self {
        match val {
            1 => Team::Terrorist,
            2 => Team::CounterTerrorist,
            3 => Team::Spectator,
            _ => Team::Unassigned,
        }
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

impl AsLangCode for crate::client::Terrorist {
    fn as_lang_code(&self) -> Cow<'_, str> {
        self.0.as_lang_code()
    }
}

impl AsLangCode for crate::client::CounterTerrorist {
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
